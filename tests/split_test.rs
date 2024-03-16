mod env;

use testcontainers::clients::Cli;
use env::TestEnv;

#[test]
fn test_split() {
    let docker = Cli::default();
    let test_env = TestEnv::build(&docker);

    let bitcoin_rpc_url = format!(
        "http://127.0.0.1:{}", 
        test_env.bitcoind.get_host_port_ipv4(18443)
    );

    let ord_rpc_url = &format!(
        "http://127.0.0.1:{}",
        test_env.ord.get_host_port_ipv4(80)
    );
  
    assert_eq!(bitcoin_rpc_url, "http://127.0.0.1:1993");
    assert_eq!(ord_rpc_url, "http://127.0.0.1:1993");
}
