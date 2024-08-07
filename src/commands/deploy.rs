use crate::inscribe::InscribeOptions;
use crate::inscribe::Inscriber;
use crate::operation::deploy_args_cbor_encode;
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

    #[arg(long, help = "The generator Inscription id on Bitcoin.")]
    generator: Option<InscriptionId>,

    #[arg(long, help = "The mint factory name.")]
    factory: Option<String>,

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
        //TODO how to encode the factory args.
        let deploy_args = deploy_args_cbor_encode(self.deploy_args);

        //TODO check the tick name is valid
        let tick = self.tick.to_uppercase();
        let output = Inscriber::new(wallet, self.inscribe_options)?
            .with_deploy(
                tick,
                self.amount,
                self.generator,
                self.factory,
                self.repeat,
                deploy_args,
            )?
            .inscribe()?;
        Ok(Box::new(output))
    }
}
