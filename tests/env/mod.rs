pub mod bitcoin;
pub mod ord;

use bitcoin::BitcoinD;
use ord::Ord;
use testcontainers::{clients::Cli, core::Container, RunnableImage};

pub struct TestEnv<'a> {
    pub bitcoind: Container<'a, BitcoinD>,
    pub ord: Container<'a, Ord>,
}

impl<'a> TestEnv<'a> {
    pub fn build(docker: &'a Cli) -> TestEnv<'a> {
        let network = "test_network_1";

        let mut bitcoind_image: RunnableImage<BitcoinD> = BitcoinD::default().into();
        bitcoind_image = bitcoind_image
          .with_network(network)
          .with_run_option(("--network-alias", "bitcoind"));

        let bitcoind = docker.run(bitcoind_image);
        dbg!("bitcoind ok");

        let mut ord_image: RunnableImage<Ord> = Ord::new(
            "http://bitcoind:18443".to_owned(),
            "roochuser".to_owned(),
            "roochpass".to_owned(),
        )
        .into();
        ord_image = ord_image
            .with_network(network);

        let ord = docker.run(ord_image);
        dbg!("ord ok");

        TestEnv { bitcoind, ord }
    }
}
