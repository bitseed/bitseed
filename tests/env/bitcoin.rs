use std::collections::HashMap;

use testcontainers::{
  core::WaitFor,
  Image, ImageArgs,
};

const NAME: &str = "lncm/bitcoind";
const TAG: &str = "v25.1";

#[derive(Debug, Default, Clone)]
pub struct BitcoindImageArgs;

impl ImageArgs for BitcoindImageArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(
          vec![
            "-chain=regtest".to_string(),
            "-txindex=1".to_string(),
            "-fallbackfee=0.00001".to_string(),
            "-zmqpubrawblock=tcp://0.0.0.0:28332".to_string(),
            "-zmqpubrawtx=tcp://0.0.0.0:28333".to_string(),
            "-rpcallowip=0.0.0.0/0".to_string(),
            "-rpcbind=0.0.0.0".to_string(),
            "-rpcauth=roochuser:925300af2deda1996d8ff66f2a69dc84$681057d8bdccae2d119411befa9a5f949eff770933fc377816348024d25a2402".to_string(),
          ].into_iter(),
        )
    }
}

pub struct BitcoinD {
    env_vars: HashMap<String, String>,
}

impl Default for BitcoinD {
    fn default() -> Self {
        BitcoinD {
          env_vars: HashMap::new(),
        }
    }
}

impl Image for BitcoinD {
    type Args = BitcoindImageArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("txindex thread start")]
    }
 
    fn expose_ports(&self) -> Vec<u16> {
        vec![18443, 18444, 28333, 28332]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
      Box::new(self.env_vars.iter())
    }
}
