pub mod bitcoin;
pub mod ord;

use uuid::Uuid;
use tracing::debug;
use bitcoin::BitcoinD;
use ord::Ord;
use testcontainers::{clients::Cli, core::Container, RunnableImage};

pub struct TestEnv {
    pub bitcoind: Container<BitcoinD>,
    pub ord: Container<Ord>,
}

impl TestEnv {
    pub fn build(docker: &Cli) -> TestEnv {
        let network_uuid = Uuid::new_v4();
        let network = format!("test_network_{}", network_uuid);

        let mut bitcoind_image: RunnableImage<BitcoinD> = BitcoinD::default().into();
        bitcoind_image = bitcoind_image
            .with_network(network.clone())
            .with_run_option(("--network-alias", "bitcoind"));

        let bitcoind = docker.run(bitcoind_image);
        debug!("bitcoind ok");

        let mut ord_image: RunnableImage<Ord> = Ord::new(
            "http://bitcoind:18443".to_owned(),
            "roochuser".to_owned(),
            "roochpass".to_owned(),
        )
        .into();
        ord_image = ord_image.with_network(network.clone());

        let ord = docker.run(ord_image);
        debug!("ord ok");

        TestEnv { bitcoind, ord }
    }
}
