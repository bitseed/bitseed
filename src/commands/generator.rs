use crate::generator;
use crate::generator::Generator;
use crate::inscribe::InscribeOptions;
use crate::inscription::InscriptionBuilder;
use crate::sft::Content;
use crate::wallet::Wallet;
use anyhow::Result;
use bitcoin::address::NetworkUnchecked;
use bitcoin::Address;
use bitcoin::Amount;
use bitcoin::OutPoint;
use clap::{Parser, Subcommand};
use ord::Inscription;
use ord::{FeeRate, InscriptionId};
use std::path::PathBuf;

/// Inscribe a new generator bytecode to Bitcoin
#[derive(Debug, Parser)]
pub struct GeneratorCommand {
    #[arg(long, help = "Name of the generator.")]
    name: String,
    #[arg(long, help = "Path to the generator bytecode file.")]
    generator: PathBuf,
    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl GeneratorCommand {
    pub fn run(&self, wallet: Wallet) -> Result<String> {
        //inscription.append_reveal_script_to_builder(builder)
        //inscription.append
        // load generator
        // generate output
        // mint
        println!("Mint command run");
        Ok("".to_string())
    }
}
