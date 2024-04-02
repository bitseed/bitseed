use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::wallet::Wallet;
use crate::SubcommandResult;
use clap::Parser;
use ord::InscriptionId;

#[derive(Debug, Parser)]
pub struct DeployCommand {
    #[arg(long, help = "The SFT tick name.")]
    tick: String,

    #[arg(long, help = "The amount of the tick total supply.")]
    amount: u64,

    #[arg(long, help = "The generator Inscription id.")]
    generator: InscriptionId,

    #[arg(
        long,
        help = "The number of allowed the SFT attributes repeats. 0 means do not limit.",
        default_value = "0"
    )]
    repeat: u64,

    #[arg(long, help = "The deploy arguments to the generator program.")]
    deploy_args: Vec<String>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

impl DeployCommand {
    pub fn run(self, wallet: Wallet) -> SubcommandResult {
        //TODO check the tick name is valid
        let tick = self.tick.to_uppercase();
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_deploy(
                tick,
                self.amount,
                self.generator,
                self.repeat,
                self.deploy_args,
            )?
            .inscribe_v2()?;
        Ok(Box::new(output))
    }
}
