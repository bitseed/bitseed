use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
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
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_generator(self.name, self.generator)?
            .inscribe_v2()?;

        Ok(Box::new(output))
    }
}
