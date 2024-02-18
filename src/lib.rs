use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::mint::Mint;

pub const PROTOCOL: &str = "bitseed";

pub(crate) mod commands;
pub mod generator;
pub mod inscription;
pub mod operation;
mod ord_client;
pub mod sft;

#[derive(Parser)]
#[command(name = "bitseed")]
#[command(bin_name = "bitseed")]
pub struct BitseedCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mint(Mint),
}

pub fn run(cli: BitseedCli) -> Result<String> {
    match cli.command {
        Commands::Mint(mint) => mint.run(),
    }
}
