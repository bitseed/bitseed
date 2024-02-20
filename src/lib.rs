use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io;

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

pub trait Output: Send {
    fn print_json(&self, minify: bool);
}

impl<T> Output for T
where
    T: Serialize + Send,
{
    fn print_json(&self, minify: bool) {
        if minify {
            serde_json::to_writer(io::stdout(), self).ok();
        } else {
            serde_json::to_writer_pretty(io::stdout(), self).ok();
        }
        println!();
    }
}

pub(crate) type SubcommandResult = Result<Box<dyn Output>>;

#[derive(Subcommand)]
enum Commands {
    Generator(commands::generator::GeneratorCommand),
    Deploy(commands::deploy::DeployCommand),
    Mint(commands::mint::MintCommand),
}

pub fn run(cli: BitseedCli) -> Result<()> {
    let wallet = wallet::Wallet::new(cli.wallet_options)?;
    let output = match cli.command {
        Commands::Generator(generator) => generator.run(wallet),
        Commands::Deploy(deploy) => deploy.run(wallet),
        Commands::Mint(mint) => mint.run(wallet),
    }?;
    output.print_json(true);
    Ok(())
}
