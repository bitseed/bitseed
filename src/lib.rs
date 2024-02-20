use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::mint::MintCommand;

pub const PROTOCOL: &str = "bitseed";
pub const METADATA_OP: &str = "op";
pub const METADATA_TICK: &str = "tick";
pub const METADATA_AMOUNT: &str = "amount";
pub const METADATA_ATTRIBUTES: &str = "attributes";

pub(crate) mod commands;
pub mod generator;
pub mod inscribe;
pub mod inscription;
pub mod operation;
pub mod sft;
mod wallet;

#[derive(Parser)]
#[command(name = "bitseed")]
#[command(bin_name = "bitseed")]
pub struct BitseedCli {
    #[clap(flatten)]
    wallet_options: wallet::WalletOption,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generator(commands::generator::GeneratorCommand),
    Mint(MintCommand),
}

pub fn run(cli: BitseedCli) -> Result<String> {
    let wallet = wallet::Wallet::new(cli.wallet_options)?;
    match cli.command {
        Commands::Generator(generator) => generator.run(wallet),
        Commands::Mint(mint) => mint.run(wallet),
    }
}
