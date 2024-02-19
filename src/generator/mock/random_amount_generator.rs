use crate::generator::{Generator, InscribeSeed};
use bitcoin::{address::NetworkUnchecked, Address};
use primitive_types::U256;

pub struct RandomAmountGenerator;

impl Generator for RandomAmountGenerator {
    fn inscribe_generate(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        _recipient: Address<NetworkUnchecked>,
        _user_input: Option<String>,
    ) -> crate::generator::InscribeGenerateOutput {
        let hash = seed.seed();
        let min = deploy_args[1].parse::<u64>().unwrap();
        let max = deploy_args[0].parse::<u64>().unwrap();
        let amount = (U256::from_little_endian(hash.as_bytes()) % (max - min) + min).as_u64();
        crate::generator::InscribeGenerateOutput {
            amount,
            attributes: None,
            content_type: None,
            content: None,
        }
    }
}
