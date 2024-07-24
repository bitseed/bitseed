use crate::operation::Operation;
use anyhow::{anyhow, bail, Result};
use bitcoin::Address;
use bitcoin::OutPoint;
use bitcoin::TxOut;
use bitcoincore_rpc::RpcApi;
use clap::Parser;
use ord::inscriptions::ParsedEnvelope;
use ord::Chain;
use ord::InscriptionId;
use ord::Options;
use ordinals::SatPoint;
use reqwest::Url;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
pub struct WalletOption {
    #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
    pub name: String,
    #[arg(long, alias = "nosync", help = "Do not update index.")]
    pub no_sync: bool,
    #[arg(
        long,
        default_value = "http://127.0.0.1:80",
        help = "Use ord running at <SERVER_URL>."
    )]
    pub server_url: Url,

    #[clap(flatten)]
    pub chain_options: Options,
}
/// Wallet is a wrapper around ord::wallet::Wallet.
#[derive(Clone)]
pub struct Wallet {
    ord_wallet: Arc<ord::wallet::Wallet>,
}

impl Wallet {
    pub fn new(opt: WalletOption) -> Result<Self> {
        let ord_settings = ord::settings::Settings::load(opt.chain_options)?;
        let wallet = ord::wallet::wallet_constructor::WalletConstructor::construct(
            opt.name,
            opt.no_sync,
            ord_settings,
            opt.server_url,
        )?;

        Ok(Self {
            ord_wallet: Arc::new(wallet),
        })
    }

    pub fn bitcoin_client(&self) -> Result<&bitcoincore_rpc::Client> {
        Ok(&self.ord_wallet.bitcoin_client)
    }

    pub fn ord_client(&self) -> Result<&reqwest::blocking::Client> {
        Ok(&self.ord_wallet.ord_client)
    }

    pub fn get_unspent_outputs(&self) -> Result<BTreeMap<OutPoint, TxOut>> {
        let utxos = self.ord_wallet.utxos();

        let mut outputs = BTreeMap::new();
        for (outpoint, txout) in utxos.iter() {
            outputs.insert(*outpoint, txout.clone());
        }

        Ok(outputs)
    }

    pub fn get_output_sat_ranges(&self) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
        self.ord_wallet.get_wallet_sat_ranges()
    }

    pub fn inscription_exists(&self, inscription_id: InscriptionId) -> Result<bool> {
        self.ord_wallet.inscription_exists(inscription_id)
    }

    pub fn get_inscriptions(&self) -> Result<BTreeMap<SatPoint, Vec<InscriptionId>>> {
        let inscriptions = self.ord_wallet.inscriptions();

        let mut outputs = BTreeMap::new();
        for (satpoint, ids) in inscriptions.iter() {
            outputs.insert(*satpoint, ids.clone());
        }

        Ok(outputs)
    }

    pub fn get_inscription_satpoint(&self, inscription_id: InscriptionId) -> Result<SatPoint> {
        let inscriptions = self.ord_wallet.inscriptions();

        for (satpoint, ids) in inscriptions.iter() {
            if ids.contains(&inscription_id) {
                return Ok(*satpoint);
            }
        }

        bail!("Inscription ID not found: {}", inscription_id);
    }

    pub fn get_inscription(&self, inscription_id: InscriptionId) -> Result<ord::api::Inscription> {
        let inscription_info = self.ord_wallet.inscription_info.get(&inscription_id);

        match inscription_info {
            Some(ins) => Ok(ins.clone()),
            None => {
                bail!("Inscription ID not found: {}", inscription_id);
            }
        }
    }

    pub fn get_inscription_envelope(
        &self,
        inscription_id: InscriptionId,
    ) -> Result<ord::Envelope<ord::Inscription>> {
        let tx = self.get_raw_transaction(&inscription_id.txid)?;
        let inscriptions = ParsedEnvelope::from_transaction(&tx);

        let envelope = inscriptions
            .into_iter()
            .nth(inscription_id.index as usize)
            .ok_or_else(|| anyhow!("Inscription not found in the transaction"))?;

        Ok(envelope)
    }

    pub fn get_inscription_satpoint_v2(&self, inscription_id: InscriptionId) -> Result<SatPoint> {
        let envelope = self.get_inscription_envelope(inscription_id)?;

        Ok(SatPoint {
            outpoint: OutPoint {
                txid: inscription_id.txid,
                vout: envelope.input,
            },
            offset: envelope.offset as u64,
        })
    }

    pub fn get_runic_outputs(&self) -> Result<BTreeSet<OutPoint>> {
        self.ord_wallet.get_runic_outputs()
    }

    pub fn get_locked_outputs(&self) -> Result<BTreeSet<OutPoint>> {
        let locked_utxos = self.ord_wallet.locked_utxos();

        let mut outputs = BTreeSet::new();
        for (outpoint, _) in locked_utxos.iter() {
            outputs.insert(*outpoint);
        }

        Ok(outputs)
    }

    pub fn get_primary_address(&self) -> Result<Address> {
        let client = self.bitcoin_client()?;
        let address =
            client.get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

        address
            .require_network(self.chain().network())
            .map_err(anyhow::Error::from)
    }

    pub fn get_change_address(&self) -> Result<Address> {
        self.ord_wallet.get_change_address()
    }

    pub fn has_sat_index(&self) -> Result<bool> {
        Ok(self.ord_wallet.has_sat_index)
    }

    pub fn chain(&self) -> Chain {
        self.ord_wallet.chain()
    }

    pub fn get_raw_transaction(&self, txid: &bitcoin::Txid) -> Result<bitcoin::Transaction> {
        let client = self.bitcoin_client()?;
        Ok(client.get_raw_transaction(txid, None)?)
    }

    pub fn exists_utxo(&self, outpoint: &OutPoint) -> Result<bool> {
        Ok(self.get_unspent_outputs()?.contains_key(outpoint))
    }

    pub fn select_utxo(&self, destination: &Address) -> Result<OutPoint> {
        let utxos = self.get_unspent_outputs()?;

        let wallet_inscriptions = self.get_inscriptions()?;
        let runic_utxos = self.get_runic_outputs()?;
        let locked_utxos = self.get_locked_outputs()?;
        let inscribed_utxos = wallet_inscriptions
            .keys()
            .map(|satpoint| satpoint.outpoint)
            .collect::<BTreeSet<OutPoint>>();

        utxos
            .iter()
            .find(|(outpoint, txout)| {
                txout.value > destination.script_pubkey().dust_value().to_sat()
                    && !inscribed_utxos.contains(outpoint)
                    && !locked_utxos.contains(outpoint)
                    && !runic_utxos.contains(outpoint)
            })
            .map(|(outpoint, _amount)| *outpoint)
            .ok_or_else(|| anyhow!("wallet contains no cardinal utxos"))
    }

    pub fn get_operation_by_inscription_id(
        &self,
        inscription_id: InscriptionId,
    ) -> Result<Operation> {
        let tx = self.get_raw_transaction(&inscription_id.txid)?;
        let inscriptions = ParsedEnvelope::from_transaction(&tx);

        let envelope = inscriptions
            .into_iter()
            .nth(inscription_id.index as usize)
            .ok_or_else(|| anyhow!("Inscription not found in the transaction"))?;

        Operation::from_inscription(envelope.payload)
    }

    pub fn send_raw_transaction_v2<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
        maxfeerate: Option<f64>,
        maxburnamount: Option<f64>,
    ) -> Result<bitcoin::Txid> {
        let bitcoin_client = self.bitcoin_client()?;

        // Prepare the parameters for the RPC call
        let mut params = vec![tx.raw_hex().into()];

        // Add maxfeerate and maxburnamount to the params if they are Some
        if let Some(feerate) = maxfeerate {
            params.push(serde_json::to_value(feerate).unwrap());
        } else {
            params.push(serde_json::to_value(0.10).unwrap());
        }

        if let Some(burnamount) = maxburnamount {
            params.push(serde_json::to_value(burnamount).unwrap());
        } else {
            params.push(serde_json::to_value(0.0).unwrap());
        }

        // Make the RPC call
        let tx_id: bitcoin::Txid = bitcoin_client.call("sendrawtransaction", &params)?;

        Ok(tx_id)
    }
}
