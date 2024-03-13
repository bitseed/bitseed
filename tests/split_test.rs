use testcontainers::clients::Cli;

mod image;

use image::bitcoin::BitcoinD;
use image::ord::Ord;

#[test]
fn test_split() {
    let docker = Cli::default();

    let bitcoind_image = BitcoinD::default();
    let bitcoind = docker.run(bitcoind_image);

    let bitcoin_rpc_url = &format!(
        "http://127.0.0.1:{}",
        bitcoind.get_host_port_ipv4(18443)
    );
    
    let ord = docker.run(Ord::new(bitcoin_rpc_url.to_owned(), "roochuser".to_owned(), "roochpass".to_owned()));
    let ord_rpc_url = &format!(
        "http://127.0.0.1:{}",
        ord.get_host_port_ipv4(80)
    );

    assert_eq!(ord_rpc_url, "http://127.0.0.1:1993");
}
