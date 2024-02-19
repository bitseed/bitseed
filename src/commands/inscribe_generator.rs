use crate::generator;
use crate::generator::Generator;
use crate::inscribe::InscribeOptions;
use crate::inscription::InscriptionBuilder;
use crate::sft::Content;
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
pub struct InscribeGenerator {
    #[arg(long, help = "Name of the generator.")]
    name: String,
    #[arg(long, help = "Path to the generator bytecode file.")]
    generator: PathBuf,
    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl InscribeGenerator {
    pub fn run(&self) -> Result<String> {
        let bytecode = std::fs::read(&self.generator)?;
        let content = Content::new(generator::CONTENT_TYPE.to_string(), bytecode);
        let mut inscription = InscriptionBuilder::new()
            .amount(1)
            .tick(generator::TICK)
            .content(content)
            .add_metadata_string("name", self.name.clone())
            .finish();
        //inscription.append_reveal_script_to_builder(builder)
        //inscription.append
        // load generator
        // generate output
        // mint
        println!("Mint command run");
        Ok("".to_string())
    }
}
