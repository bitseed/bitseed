use crate::inscription;

use {
    crate::{
        generator::{self, GeneratorLoader, InscribeSeed},
        operation::{DeployRecord, MintRecord, Operation},
        sft::{Content, SFT},
        wallet::Wallet,
        GENERATOR_TICK,
    },
    anyhow::{anyhow, bail, ensure, Result},
    bitcoin::{
        absolute::LockTime,
        address::NetworkUnchecked,
        blockdata::{opcodes, script},
        key::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
        policy::MAX_STANDARD_TX_WEIGHT,
        secp256k1::{self, constants::SCHNORR_SIGNATURE_SIZE, rand, Secp256k1, XOnlyPublicKey},
        sighash::{Prevouts, SighashCache, TapSighashType},
        taproot::{ControlBlock, LeafVersion, Signature, TapLeafHash, TaprootBuilder},
        Address, Amount, OutPoint, PrivateKey, Script, ScriptBuf, Sequence, Transaction, TxIn,
        TxOut, Txid, Witness,
    },
    bitcoincore_rpc::{
        bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
        RpcApi,
    },
    ciborium::Value,
    clap::Parser,
    ord::{FeeRate, Inscription, InscriptionId, Target, TransactionBuilder},
    ordinals::SatPoint,
    serde::{Deserialize, Serialize},
    std::{collections::BTreeMap, path::Path},
};

const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);

#[derive(Debug, Clone, Parser)]
pub struct InscribeOptions {
    #[arg(
        long,
        help = "Inscribe <SATPOINT>. This SatPoint will be used as mint seed."
    )]
    pub(crate) satpoint: Option<SatPoint>,
    #[arg(
        long,
        help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
    )]
    pub(crate) commit_fee_rate: Option<FeeRate>,
    #[arg(long, help = "Send inscription to <DESTINATION>.")]
    pub(crate) destination: Option<Address<NetworkUnchecked>>,
    #[arg(long, help = "Don't sign or broadcast transactions.")]
    pub(crate) dry_run: bool,
    #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
    pub(crate) fee_rate: FeeRate,
    #[arg(long, alias = "nobackup", help = "Do not back up recovery key.")]
    pub(crate) no_backup: bool,
    #[arg(
        long,
        alias = "nolimit",
        help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
    )]
    pub(crate) no_limit: bool,
    #[arg(
        long,
        help = "Amount of postage to include in the inscription. Default `10000sat`."
    )]
    pub(crate) postage: Option<Amount>,
}

impl InscribeOptions {
    pub fn postage(&self) -> Amount {
        self.postage.unwrap_or(TARGET_POSTAGE)
    }

    pub fn commit_fee_rate(&self) -> FeeRate {
        self.commit_fee_rate.unwrap_or(self.fee_rate)
    }

    pub fn reveal_fee_rate(&self) -> FeeRate {
        self.fee_rate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InscriptionOrId {
    Inscription(Inscription),
    Id(InscriptionId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscribeOutput {
    commit_tx: Txid,
    reveal_tx: Txid,
    total_fees: u64,
    inscription: InscriptionOrId,
}

pub struct Inscriber {
    wallet: Wallet,
    option: InscribeOptions,
    inscriptions: Vec<Inscription>,
    satpoint: SatPoint,
    destination: Address,
}

impl Inscriber {
    pub fn new(wallet: Wallet, option: InscribeOptions) -> Result<Self> {
        let destination = match option.destination.clone() {
            Some(destination) => destination.require_network(wallet.chain().network())?,
            None => wallet.get_change_address()?,
        };

        let satpoint = match option.satpoint.clone() {
            Some(satpoint) => {
                //TODO check the satpoint exists.
                satpoint
            }
            None => {
                let utxo = wallet.select_utxo(&destination)?;
                SatPoint {
                    outpoint: utxo,
                    offset: 0,
                }
            }
        };

        Ok(Self {
            wallet,
            option,
            inscriptions: Vec::new(),
            satpoint,
            destination,
        })
    }

    pub fn with_generator<P>(self, generator_name: String, generator_program: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let bytecode = std::fs::read(generator_program)?;
        let content = Content::new(generator::CONTENT_TYPE.to_string(), bytecode);
        let attributes = Value::Map(vec![(
            Value::Text("name".to_string()),
            Value::Text(generator_name.clone().into()),
        )]);
        let mint_record = MintRecord {
            sft: SFT {
                tick: GENERATOR_TICK.to_string(),
                amount: 1,
                attributes: Some(attributes),
                content: Some(content),
            },
        };
        self.with_operation(Operation::Mint(mint_record))
    }

    pub fn with_deploy(
        self,
        tick: String,
        amount: u64,
        generator: InscriptionId,
        repeat: u64,
        deploy_args: Vec<String>,
    ) -> Result<Self> {
        //TODO check the generator exists.
        let deploy_record = DeployRecord {
            tick,
            amount,
            generator: format!("/inscription/{}", generator),
            repeat,
            deploy_args,
        };
        self.with_operation(Operation::Deploy(deploy_record))
    }

    pub fn with_mint(
        self,
        deploy_inscription: InscriptionId,
        user_input: Option<String>,
    ) -> Result<Self> {
        let operation = self
            .wallet
            .get_operation_by_inscription_id(deploy_inscription)?;
        let deploy_record = match operation {
            Operation::Deploy(deploy_record) => deploy_record,
            _ => bail!("deploy transaction must have a deploy operation"),
        };
        let generator_loader = GeneratorLoader::new(self.wallet.clone());
        let generator = generator_loader.load(&deploy_record.generator)?;

        let seed_utxo = self.satpoint.outpoint;
        let btc_client = self.wallet.bitcoin_client()?;
        let seed_tx = btc_client.get_transaction(&seed_utxo.txid, Some(true))?;
        let seed = InscribeSeed::new(
            seed_tx
                .info
                .blockhash
                .ok_or_else(|| anyhow!("seed utxo has no blockhash"))?,
            seed_utxo,
        );

        let destination = self.destination.clone();

        let output =
            generator.inscribe_generate(deploy_record.deploy_args, &seed, destination, user_input);
        let sft = SFT {
            tick: deploy_record.tick,
            amount: output.amount,
            attributes: output.attributes,
            content: output.content,
        };
        let mint_record = MintRecord { sft };
        self.with_operation(Operation::Mint(mint_record))
    }

    pub fn with_split(mut self, asset_inscription_id: InscriptionId, amount: u64) -> Result<Self> {
        Ok(self)
    }

    fn with_operation(mut self, operation: Operation) -> Result<Self> {
        let inscription = operation.to_inscription();
        self.inscriptions.push(inscription);
        Ok(self)
    }

    pub fn inscribe(&self) -> Result<Vec<InscribeOutput>> {
        let outputs: Result<Vec<InscribeOutput>, _> = self
            .inscriptions
            .iter()
            .map(|inscription| self.inscribe_inner(inscription))
            .collect();

        outputs
    }

    fn inscribe_inner(&self, inscription: &Inscription) -> Result<InscribeOutput> {
        let mut utxos = self.wallet.get_unspent_outputs()?;
        let locked_utxos = self.wallet.get_locked_outputs()?;
        let runic_utxos = self.wallet.get_runic_outputs()?;
        let chain = self.wallet.chain();
        let commit_tx_change = [
            self.wallet.get_change_address()?,
            self.wallet.get_change_address()?,
        ];
        let wallet_inscriptions = self.wallet.get_inscriptions()?;

        let destination = self.destination.clone();
        let satpoint = self.satpoint;

        let secp256k1 = Secp256k1::new();
        let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
        let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

        let reveal_script = inscription
            .append_reveal_script_to_builder(
                ScriptBuf::builder()
                    .push_slice(public_key.serialize())
                    .push_opcode(opcodes::all::OP_CHECKSIG),
            )
            .into_script();

        let taproot_spend_info = TaprootBuilder::new()
            .add_leaf(0, reveal_script.clone())
            .expect("adding leaf should work")
            .finalize(&secp256k1, public_key)
            .expect("finalizing taproot builder should work");

        let control_block = taproot_spend_info
            .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
            .expect("should compute control block");

        let commit_tx_address =
            Address::p2tr_tweaked(taproot_spend_info.output_key(), chain.network());

        let total_postage = self.option.postage().to_sat();

        let mut reveal_inputs = Vec::new();
        let mut reveal_outputs = Vec::new();

        // if let Some(ParentInfo {
        //     location,
        //     id: _,
        //     destination,
        //     tx_out,
        // }) = self.parent_info.clone()
        // {
        //     reveal_inputs.push(location.outpoint);
        //     reveal_outputs.push(TxOut {
        //         script_pubkey: destination.script_pubkey(),
        //         value: tx_out.value,
        //     });
        // }

        // if self.mode == Mode::SatPoints {
        //     for (satpoint, _txout) in self.reveal_satpoints.iter() {
        //         reveal_inputs.push(satpoint.outpoint);
        //     }
        // }

        reveal_inputs.push(OutPoint::null());

        reveal_outputs.push(TxOut {
            script_pubkey: destination.script_pubkey(),
            value: self.option.postage().to_sat(),
        });

        let commit_input = 0;

        let (_, reveal_fee) = Self::build_reveal_transaction(
            &control_block,
            self.option.reveal_fee_rate(),
            reveal_inputs.clone(),
            commit_input,
            reveal_outputs.clone(),
            &reveal_script,
        );

        let target = Target::Value(reveal_fee + Amount::from_sat(total_postage));

        let commit_tx = TransactionBuilder::new(
            satpoint,
            wallet_inscriptions,
            utxos.clone(),
            locked_utxos.clone(),
            runic_utxos.clone(),
            commit_tx_address.clone(),
            commit_tx_change,
            self.option.commit_fee_rate(),
            target,
        )
        .build_transaction()?;

        let (vout, _commit_output) = commit_tx
            .output
            .iter()
            .enumerate()
            .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
            .expect("should find sat commit/inscription output");

        reveal_inputs[commit_input] = OutPoint {
            txid: commit_tx.txid(),
            vout: vout.try_into().unwrap(),
        };

        let (mut reveal_tx, _fee) = Self::build_reveal_transaction(
            &control_block,
            self.option.reveal_fee_rate(),
            reveal_inputs,
            commit_input,
            reveal_outputs.clone(),
            &reveal_script,
        );

        for output in reveal_tx.output.iter() {
            ensure!(
                output.value >= output.script_pubkey.dust_value().to_sat(),
                "commit transaction output would be dust"
            )
        }

        let mut prevouts = Vec::new();

        // if let Some(parent_info) = self.parent_info.clone() {
        //     prevouts.push(parent_info.tx_out);
        // }

        // if self.mode == Mode::SatPoints {
        //     for (_satpoint, txout) in self.reveal_satpoints.iter() {
        //         prevouts.push(txout.clone());
        //     }
        // }

        prevouts.push(commit_tx.output[vout].clone());

        let mut sighash_cache = SighashCache::new(&mut reveal_tx);

        let sighash = sighash_cache
            .taproot_script_spend_signature_hash(
                commit_input,
                &Prevouts::All(&prevouts),
                TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
                TapSighashType::Default,
            )
            .expect("signature hash should compute");

        let sig = secp256k1.sign_schnorr(
            &secp256k1::Message::from_slice(sighash.as_ref())
                .expect("should be cryptographically secure hash"),
            &key_pair,
        );

        let witness = sighash_cache
            .witness_mut(commit_input)
            .expect("getting mutable witness reference should work");

        witness.push(
            Signature {
                sig,
                hash_ty: TapSighashType::Default,
            }
            .to_vec(),
        );

        witness.push(reveal_script);
        witness.push(&control_block.serialize());

        let recovery_key_pair = key_pair.tap_tweak(&secp256k1, taproot_spend_info.merkle_root());

        let (x_only_pub_key, _parity) = recovery_key_pair.to_inner().x_only_public_key();
        assert_eq!(
            Address::p2tr_tweaked(
                TweakedPublicKey::dangerous_assume_tweaked(x_only_pub_key),
                chain.network(),
            ),
            commit_tx_address
        );

        let reveal_weight = reveal_tx.weight();

        if !self.option.no_limit
            && reveal_weight > bitcoin::Weight::from_wu(MAX_STANDARD_TX_WEIGHT.into())
        {
            bail!(
            "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
          );
        }

        utxos.insert(
            reveal_tx.input[commit_input].previous_output,
            commit_tx.output[reveal_tx.input[commit_input].previous_output.vout as usize].clone(),
        );

        let total_fees =
            Self::calculate_fee(&commit_tx, &utxos) + Self::calculate_fee(&reveal_tx, &utxos);

        if self.option.dry_run {
            return Ok(InscribeOutput {
                commit_tx: commit_tx.txid(),
                reveal_tx: reveal_tx.txid(),
                total_fees: total_fees,
                inscription: InscriptionOrId::Inscription(inscription.clone()),
            });
        }

        let bitcoin_client = self.wallet.bitcoin_client()?;

        let signed_commit_tx = bitcoin_client
            .sign_raw_transaction_with_wallet(&commit_tx, None, None)?
            .hex;

        let result = bitcoin_client.sign_raw_transaction_with_wallet(
            &reveal_tx,
            Some(
                &commit_tx
                    .output
                    .iter()
                    .enumerate()
                    .map(|(vout, output)| SignRawTransactionInput {
                        txid: commit_tx.txid(),
                        vout: vout.try_into().unwrap(),
                        script_pub_key: output.script_pubkey.clone(),
                        redeem_script: None,
                        amount: Some(Amount::from_sat(output.value)),
                    })
                    .collect::<Vec<SignRawTransactionInput>>(),
            ),
            None,
        )?;

        ensure!(
            result.complete,
            format!("Failed to sign reveal transaction: {:?}", result.errors)
        );

        let signed_reveal_tx = result.hex;

        if !self.option.no_backup {
            Self::backup_recovery_key(&self.wallet, recovery_key_pair)?;
        }

        let commit = bitcoin_client.send_raw_transaction(&signed_commit_tx)?;

        let reveal = match bitcoin_client.send_raw_transaction(&signed_reveal_tx) {
            Ok(txid) => txid,
            Err(err) => {
              return Err(anyhow!(
              "Failed to send reveal transaction: {err}\nCommit tx {commit} will be recovered once mined"
            ))
            }
          };
        let inscription_id = InscriptionId {
            txid: reveal,
            index: 0,
        };
        Ok(InscribeOutput {
            commit_tx: commit,
            reveal_tx: reveal,
            total_fees: total_fees,
            inscription: InscriptionOrId::Id(inscription_id),
        })
    }

    fn backup_recovery_key(wallet: &Wallet, recovery_key_pair: TweakedKeyPair) -> Result<()> {
        let recovery_private_key = PrivateKey::new(
            recovery_key_pair.to_inner().secret_key(),
            wallet.chain().network(),
        );

        let bitcoin_client = wallet.bitcoin_client()?;

        let info = bitcoin_client
            .get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

        let response = bitcoin_client.import_descriptors(vec![ImportDescriptors {
            descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
            timestamp: Timestamp::Now,
            active: Some(false),
            range: None,
            next_index: None,
            internal: Some(false),
            label: Some("commit tx recovery key".to_string()),
        }])?;

        for result in response {
            if !result.success {
                return Err(anyhow!("commit tx recovery key import failed"));
            }
        }

        Ok(())
    }

    fn build_reveal_transaction(
        control_block: &ControlBlock,
        fee_rate: FeeRate,
        reveal_inputs: Vec<OutPoint>,
        commit_input_index: usize,
        outputs: Vec<TxOut>,
        script: &Script,
    ) -> (Transaction, Amount) {
        let reveal_tx = Transaction {
            input: reveal_inputs
                .iter()
                .map(|outpoint| TxIn {
                    previous_output: *outpoint,
                    script_sig: script::Builder::new().into_script(),
                    witness: Witness::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                })
                .collect(),
            output: outputs,
            lock_time: LockTime::ZERO,
            version: 2,
        };

        let fee = {
            let mut reveal_tx = reveal_tx.clone();

            for (current_index, txin) in reveal_tx.input.iter_mut().enumerate() {
                // add dummy inscription witness for reveal input/commit output
                if current_index == commit_input_index {
                    txin.witness.push(
                        Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
                            .unwrap()
                            .to_vec(),
                    );
                    txin.witness.push(script);
                    txin.witness.push(&control_block.serialize());
                } else {
                    txin.witness = Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]);
                }
            }

            fee_rate.fee(reveal_tx.vsize())
        };

        (reveal_tx, fee)
    }

    fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, TxOut>) -> u64 {
        tx.input
            .iter()
            .map(|txin| utxos.get(&txin.previous_output).unwrap().value)
            .sum::<u64>()
            .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
            .unwrap()
    }
}
