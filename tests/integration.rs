mod env;

use std::time::Duration;

use anyhow::{bail, Result};
use testcontainers::{clients::Cli, core::Container};
use clap::Parser;
use cucumber::{given, then, when, World as _};
use jpst::TemplateContext;
use serde_json::Value;
use tracing::{debug, info};

use bitseed::BitseedCli;

use env::bitcoin::BitcoinD;
use env::ord::Ord;
use env::TestEnv;

#[derive(cucumber::World, Debug)]
struct World {
    bitcoind: Option<Container<BitcoinD>>,
    ord: Option<Container<Ord>>,
    tpl_ctx: Option<TemplateContext>,
}

impl Default for World {
    fn default() -> Self {
        World {
            bitcoind: None,
            ord: None,
            tpl_ctx: None,
        }
    }
}

#[then(regex = r#"sleep: "(.*)?""#)]
async fn sleep(_world: &mut World, args: String) {
    let args = args.trim().parse::<u64>().unwrap();
    debug!("sleep: {}", args);
    tokio::time::sleep(tokio::time::Duration::from_secs(args)).await;
}

#[given(expr = "Prepare bitcoind and Ord")] // Cucumber Expression
async fn prepare_bitcoind_and_ord(w: &mut World) {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let docker = Cli::default();
    let test_env = TestEnv::build(&docker);

    w.bitcoind = Some(test_env.bitcoind);
    w.ord = Some(test_env.ord);
}

#[then(expr = "release")] // Cucumber Expression
async fn release_bitcoind_and_ord(w: &mut World) {
    println!("stop server");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match w.ord.take() {
        Some(ord) => {
            ord.stop();
            info!("Shutdown Sever");
        }
        None => {
            info!("service is none");
        }
    }

    match w.bitcoind.take() {
        Some(bitcoind) => {
            bitcoind.stop();
            info!("Shutdown Sever");
        }
        None => {
            info!("service is none");
        }
    }
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn bitseed_run_cmd(w: &mut World, input_tpl: String) {
    let bitcoind = w.bitcoind.as_ref().unwrap();
    let ord = w.ord.as_ref().unwrap();

    let bitcoin_rpc_url = format!(
        "http://127.0.0.1:{}",
        bitcoind.get_host_port_ipv4(18443)
    );
    let ord_rpc_url = &format!(
        "http://127.0.0.1:{}", 
        ord.get_host_port_ipv4(80)
    );

    let mut bitseed_args = vec![
        "--regtest".to_string(),
        format!("--rpc-url={}", bitcoin_rpc_url),
        format!("--bitcoin-rpc-user={}", "roochuser"),
        format!("--bitcoin-rpc-pass={}", "roochpass"),
        format!("--server-url={}", ord_rpc_url),
    ];


    if w.tpl_ctx.is_none() {
        let mut tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0].clone();

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let opts = BitseedCli::parse_from(bitseed_args);

    let ret = bitseed::run(opts);
    match ret {
        Ok(output) => {
            let mut buffer = Vec::new();
            output.print_json(&mut buffer, true);

            let result_json = serde_json::from_slice::<Value>(&buffer);

            if result_json.is_ok() {
                tpl_ctx
                    .entry(cmd_name)
                    .append::<Value>(result_json.unwrap());
            }
        }
        Err(err) => {
            let err_msg = Value::String(err.to_string());
            tpl_ctx.entry(cmd_name).append::<Value>(err_msg);
        }
    }
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, orginal_args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    assert!(orginal_args.len() > 0, "assert args is empty");
    let args = eval_command_args(world.tpl_ctx.as_ref().unwrap(), orginal_args.clone());
    let splited_args = split_string_with_quotes(&args).expect("Invalid commands");
    debug!(
        "originl args: {}\n after eval: {}\n after split: {:?}",
        orginal_args, args, splited_args
    );
    assert!(
        !splited_args.is_empty(),
        "splited_args should not empty, the orginal_args:{}",
        orginal_args
    );
    for chunk in splited_args.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        debug!("assert value: {:?} {:?} {:?}", first, op, second);

        match (first, op, second) {
            (Some(first), Some(op), Some(second)) => match op.as_str() {
                "==" => assert_eq!(first, second, "Assert {:?} == {:?} failed", first, second),
                "!=" => assert_ne!(first, second, "Assert {:?} 1= {:?} failed", first, second),
                "contains" => assert!(
                    first.contains(&second),
                    "Assert {:?} contains {:?} failed",
                    first,
                    second
                ),
                _ => panic!("unsupported operator {:?}", op.as_str()),
            },
            _ => panic!(
                "expected 3 arguments: first [==|!=] second, but got input {:?}",
                args
            ),
        }
    }
    info!("assert ok!");
}

/// Split a string into a vector of strings, splitting on spaces, but ignoring spaces inside quotes.
/// And quotes will alse be removed.
fn split_string_with_quotes(s: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_escape = false;
    let mut in_single_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                in_escape = true;
            }
            '"' => {
                if in_escape {
                    current.push(c);
                    in_escape = false;
                } else if in_single_quotes {
                    current.push(c);
                } else {
                    // Skip the quote
                    in_quotes = !in_quotes;
                }
            }
            '\'' => {
                if in_escape {
                    current.push(c);
                    in_escape = false;
                } else if in_quotes {
                    current.push(c);
                } else {
                    // Skip the quote
                    in_single_quotes = !in_single_quotes;
                }
            }
            ' ' if !in_quotes && !in_single_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        bail!("Mismatched quotes")
    }

    if !current.is_empty() {
        result.push(current);
    }

    Ok(result)
}

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    let eval_args = jpst::format_str!(&args, ctx);
    eval_args
}

#[tokio::main]
async fn main() {
    World::run("tests/features/generator.feature").await;
}
