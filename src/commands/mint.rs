use anyhow::Result;
use bitcoin::address::NetworkUnchecked;
use bitcoin::Address;
use bitcoin::Amount;
use bitcoin::OutPoint;
use clap::{Parser, Subcommand};
use ord::Inscription;
use ord::{FeeRate, InscriptionId};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Mint {
    #[arg(long, help = "The deploy inscription id.")]
    pub(crate) deploy_inscription_id: InscriptionId,
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
    //   #[arg(
    //     long,
    //     help = "Include JSON in file at <METADATA> converted to CBOR as inscription metadata",
    //     conflicts_with = "cbor_metadata"
    //   )]
    //   pub(crate) json_metadata: Option<PathBuf>,
    #[clap(long, help = "Set inscription metaprotocol to <METAPROTOCOL>.")]
    pub(crate) metaprotocol: Option<String>,
    #[arg(long, alias = "nobackup", help = "Do not back up recovery key.")]
    pub(crate) no_backup: bool,
    #[arg(
        long,
        alias = "nolimit",
        help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
    )]
    pub(crate) no_limit: bool,
    #[clap(long, help = "Make inscription a child of <PARENT>.")]
    pub(crate) parent: Option<InscriptionId>,
    #[arg(
        long,
        help = "Amount of postage to include in the inscription. Default `10000sat`."
    )]
    pub(crate) postage: Option<Amount>,
    #[clap(long, help = "Allow reinscription.")]
    pub(crate) reinscribe: bool,
    #[arg(
        long,
        help = "Use <UTXO> as the input for the inscription transaction."
    )]
    pub utxo: Option<OutPoint>,
}

impl Mint {
    pub fn run(&self) -> Result<String> {
        // load generator
        // generate output
        // mint
        println!("Mint command run");
        Ok("".to_string())
    }
}
