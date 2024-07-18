use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
use ord::InscriptionId;

use {
    crate::operation::{AsSFT, Operation},
    anyhow::bail,
};

#[derive(Debug, Parser)]
pub struct ViewCommand {
    #[arg(long, help = "The SFT inscription ID to view.")]
    sft_inscription_id: InscriptionId,
}

impl ViewCommand {
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        let operation = wallet.get_operation_by_inscription_id(self.sft_inscription_id)?;
        let sft = match operation {
            Operation::Mint(mint_record) => mint_record.as_sft(),
            Operation::Split(split_record) => split_record.as_sft(),
            Operation::Merge(merge_record) => merge_record.as_sft(),
            _ => bail!(
                "Inscription {} is not a valid SFT record",
                self.sft_inscription_id
            ),
        };

        Ok(Box::new(sft))
    }
}
