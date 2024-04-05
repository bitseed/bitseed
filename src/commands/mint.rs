use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
use ord::InscriptionId;

#[derive(Debug, Parser)]
pub struct MintCommand {
    #[arg(long, help = "The deploy inscription id.")]
    deploy_inscription_id: InscriptionId,

    #[arg(long, help = "The user input argument to the generator.")]
    user_input: Option<String>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl MintCommand {
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_mint(self.deploy_inscription_id, self.user_input)?
            .inscribe()?;
        Ok(Box::new(output))
    }
}
