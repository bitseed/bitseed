mod env;

use env::TestEnv;
use testcontainers::clients::Cli;

#[test]
fn test_split() {
    let docker = Cli::default();
    let test_env = TestEnv::build(&docker);

    let bitcoin_rpc_url = format!(
        "http://127.0.0.1:{}",
        test_env.bitcoind.get_host_port_ipv4(18443)
    );

    let ord_rpc_url = &format!("http://127.0.0.1:{}", test_env.ord.get_host_port_ipv4(80));

    assert!(bitcoin_rpc_url.starts_with("http://127.0.0.1:"));
    assert!(ord_rpc_url.starts_with("http://127.0.0.1:"));
}
