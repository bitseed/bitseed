use primitive_types::U256;

use crate::generator::Generator;

pub struct RandomAmountGenerator;

impl Generator for RandomAmountGenerator {
    fn inscribe_generate(
        deploy_args: Vec<String>,
        seed: &crate::generator::Seed,
        _sender: bitcoin::Address<bitcoin::address::NetworkUnchecked>,
        _user_input: String,
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
