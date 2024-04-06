use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
use ord::InscriptionId;

#[derive(Debug, Parser)]
pub struct SplitCommand {
    #[arg(long, help = "The split SFT inscription ID.")]
    sft_inscription_id: InscriptionId,
    
    #[arg(long, help = "The split amounts.", num_args = 1..)]
    amounts: Vec<u64>,
    
    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl SplitCommand {
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_split(self.sft_inscription_id, self.amounts)?
            .inscribe()?;
        Ok(Box::new(output))
    }
}