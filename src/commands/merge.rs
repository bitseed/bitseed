use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
use ord::InscriptionId;

#[derive(Debug, Parser)]
pub struct MergeCommand {
    #[arg(long, help = "The merge SFT inscription IDs.")]
    sft_inscription_ids: Vec<InscriptionId>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl MergeCommand {
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_merge(self.sft_inscription_ids)?
            .inscribe()?;
        Ok(Box::new(output))
    }
}
