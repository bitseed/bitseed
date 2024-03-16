mod env;

use env::TestEnv;
use testcontainers::clients::Cli;
use bitseed::BitseedCli;
use clap::Parser;

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

    let args = vec![
        "--regtest".to_string(),
        format!("--rpc-url={}", bitcoin_rpc_url),
        format!("--bitcoin-rpc-user={}", "roochuser"),
        format!("--bitcoin-rpc-pass={}", "roochpass"),
        format!("--server-url={}", ord_rpc_url),
        "split".to_string(),
        format!("--asset-inscription-id={}", "xxxx"), //TODO how to get a real asset inscription to split
        format!("--amount={}", "1000"),
        format!("--fee-rate={}", "1")
    ];

    let opts = BitseedCli::parse_from(args);

    let ret = bitseed::run(opts);
    match ret {
        Ok(output) => {
            let mut buffer = Vec::new();
            output.print_json(&mut buffer, true);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
