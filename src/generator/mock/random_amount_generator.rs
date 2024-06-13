use crate::generator::{Generator, InscribeSeed};
use bitcoin::Address;
use primitive_types::U256;

pub struct RandomAmountGenerator;

impl Generator for RandomAmountGenerator {
    fn inscribe_generate(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        _recipient: Address,
        _user_input: Option<String>,
    ) -> crate::generator::InscribeGenerateOutput {
        let hash = seed.seed();
        let min = deploy_args[0].parse::<u64>().unwrap();
        let max = deploy_args[1].parse::<u64>().unwrap();
        let amount = (U256::from_little_endian(hash.as_bytes()) % (max - min) + min).as_u64();
        crate::generator::InscribeGenerateOutput {
            amount,
            attributes: None,
            content: None,
        }
    }

    fn inscribe_verify(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        recipient: Address,
        user_input: Option<String>,
        inscribe_output: crate::generator::InscribeGenerateOutput,
    ) -> bool {
        let output = self.inscribe_generate(deploy_args, seed, recipient, user_input);
        output == inscribe_output
    }
}
