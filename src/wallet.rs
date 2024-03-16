use crate::operation::Operation;
use anyhow::ensure;
use anyhow::{anyhow, bail, Result};
use bitcoin::Address;
use bitcoin::OutPoint;
use bitcoin::TxOut;
use bitcoincore_rpc::RpcApi;
use clap::Parser;
use ord::inscriptions::ParsedEnvelope;
use ord::templates::inscription::InscriptionJson;
use ord::templates::status::StatusJson;
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
        let wallet = ord::wallet::Wallet {
            name: opt.name,
            no_sync: opt.no_sync,
            options: opt.chain_options,
            ord_url: opt.server_url,
        };
        Ok(Self {
            ord_wallet: Arc::new(wallet),
        })
    }

    pub fn bitcoin_client(&self) -> Result<bitcoincore_rpc::Client> {
        self.ord_wallet.bitcoin_client()
    }

    pub fn ord_client(&self) -> Result<reqwest::blocking::Client> {
        self.ord_wallet.ord_client()
    }

    pub fn get_unspent_outputs(&self) -> Result<BTreeMap<OutPoint, TxOut>> {
        self.ord_wallet.get_unspent_outputs()
    }

    pub fn get_output_sat_ranges(&self) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
        self.ord_wallet.get_output_sat_ranges()
    }

    pub fn inscription_exists(&self, inscription_id: InscriptionId) -> Result<bool> {
        self.ord_wallet.inscription_exists(inscription_id)
    }

    pub fn get_inscriptions(&self) -> Result<BTreeMap<SatPoint, Vec<InscriptionId>>> {
        self.ord_wallet.get_inscriptions()
    }

    pub fn get_inscription_satpoint(&self, inscription_id: InscriptionId) -> Result<SatPoint> {
        self.ord_wallet.get_inscription_satpoint(inscription_id)
    }

    pub fn get_inscription(&self, inscription_id: InscriptionId) -> Result<InscriptionJson> {
        let response = self
            .ord_client()?
            .get(
                self.ord_wallet
                    .ord_url
                    .join(&format!("/inscription/{inscription_id}"))
                    .unwrap(),
            )
            .send()?;

        if !response.status().is_success() {
            bail!("inscription {inscription_id} not found");
        }

        Ok(serde_json::from_str(&response.text()?)?)
    }

    pub fn get_runic_outputs(&self) -> Result<BTreeSet<OutPoint>> {
        self.ord_wallet.get_runic_outputs()
    }

    pub fn get_locked_outputs(&self) -> Result<BTreeSet<OutPoint>> {
        self.ord_wallet.get_locked_outputs()
    }

    pub fn get_change_address(&self) -> Result<Address> {
        self.ord_wallet.get_change_address()
    }

    pub fn get_server_status(&self) -> Result<StatusJson> {
        self.ord_wallet.get_server_status()
    }

    pub fn has_sat_index(&self) -> Result<bool> {
        Ok(self.get_server_status()?.sat_index)
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
        let inscription_json = self.get_inscription(inscription_id)?;
        let tx = self.get_raw_transaction(&inscription_json.inscription_id.txid)?;
        let inscriptions = ParsedEnvelope::from_transaction(&tx);
        //TODO do we support batch inscriptions?
        ensure!(
            inscriptions.len() == 1,
            "bitseed transaction must have exactly one inscription"
        );
        let envelope = inscriptions
            .into_iter()
            .next()
            .expect("inscriptions length checked");
        Operation::from_inscription(envelope.payload)
    }
}
