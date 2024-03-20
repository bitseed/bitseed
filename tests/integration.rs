mod env;

use std::time::Duration;

use anyhow::{bail, Result};
use clap::Parser;
use cucumber::{given, then, when, World as _};
use jpst::TemplateContext;
use serde_json::Value;
use testcontainers::{clients::Cli, core::{ WaitFor, Container, ExecCommand}};
use tracing::{Level, error, debug, info};
use tracing_subscriber;
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

#[given(expr = "bitcoind and Ord servers")] // Cucumber Expression
async fn prepare_bitcoind_and_ord(w: &mut World) {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let docker = Cli::default();
    let test_env = TestEnv::build(&docker);

    w.bitcoind = Some(test_env.bitcoind);
    w.ord = Some(test_env.ord);
}

#[then(expr = "release bitcoind and Ord servers")] // Cucumber Expression
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

#[then(regex = r#"cmd ord: "(.*)?""#)]
fn ord_run_cmd(w: &mut World, input_tpl: String) {
    let bitcoind = w.bitcoind.as_ref().unwrap();
    let ord = w.ord.as_ref().unwrap();

    let bitcoin_rpc_url = format!("http://127.0.0.1:{}", bitcoind.get_host_port_ipv4(18443));

    let mut bitseed_args = vec![
        "--regtest".to_string(),
        format!("--rpc-url={}", bitcoin_rpc_url),
        format!("--bitcoin-rpc-user={}", "roochuser"),
        format!("--bitcoin-rpc-pass={}", "roochpass"),
    ];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0].clone();

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: ord {}", joined_args);

    let exec_cmd = ExecCommand{
        cmd:  joined_args,
        ready_conditions: vec![WaitFor::Duration{length: Duration::from_secs(5)}],
    };

    let output = ord.exec(exec_cmd);

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    tpl_ctx.entry(format!("{}-stdout", cmd_name)).append::<String>(stdout_string);
    tpl_ctx.entry(format!("{}-stderr", cmd_name)).append::<String>(stderr_string);

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"cmd bitcoind: "(.*)?""#)]
fn bitcoind_run_cmd(w: &mut World, input_tpl: String) {
    let bitcoind = w.bitcoind.as_ref().unwrap();
    let ord = w.ord.as_ref().unwrap();

    let bitcoin_rpc_url = format!("http://127.0.0.1:{}", bitcoind.get_host_port_ipv4(18443));

    let mut bitseed_args = vec![
        "--regtest".to_string(),
        format!("--rpc-url={}", bitcoin_rpc_url),
        format!("--bitcoin-rpc-user={}", "roochuser"),
        format!("--bitcoin-rpc-pass={}", "roochpass"),
    ];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0].clone();

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: {}", joined_args);

    let exec_cmd = ExecCommand{
        cmd:  joined_args,
        ready_conditions: vec![WaitFor::Duration{length: Duration::from_secs(5)}],
    };

    let output = bitcoind.exec(exec_cmd);

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    tpl_ctx.entry(format!("{}-stdout", cmd_name)).append::<String>(stdout_string);
    tpl_ctx.entry(format!("{}-stderr", cmd_name)).append::<String>(stderr_string);

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"cmd bitseed: "(.*)?""#)]
fn bitseed_run_cmd(w: &mut World, input_tpl: String) {
    let bitcoind = w.bitcoind.as_ref().unwrap();
    let ord = w.ord.as_ref().unwrap();

    let bitcoin_rpc_url = format!("http://127.0.0.1:{}", bitcoind.get_host_port_ipv4(18443));
    let ord_rpc_url = &format!("http://127.0.0.1:{}", ord.get_host_port_ipv4(80));

    let mut bitseed_args = vec![
        "--regtest".to_string(),
        format!("--rpc-url={}", bitcoin_rpc_url),
        format!("--bitcoin-rpc-user={}", "roochuser"),
        format!("--bitcoin-rpc-pass={}", "roochpass"),
        format!("--server-url={}", ord_rpc_url),
    ];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0].clone();

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: bitseed {}", joined_args);

    let opts = BitseedCli::parse_from(bitseed_args);
    let ret = bitseed::run(opts);

    match ret {
        Ok(output) => {
            let mut buffer = Vec::new();
            output.print_json(&mut buffer, true);

            let result_json = serde_json::from_slice::<Value>(&buffer);

            if let Ok(json_value) = result_json {
                debug!("bitseed cmd: {} output: {}", cmd_name, json_value);

                tpl_ctx.entry(cmd_name).append::<Value>(json_value);
            } else {
                debug!("result_json not ok!");
            }
        }
        Err(err) => {
            debug!("bitseed cmd: {} error, detail: {:?}", cmd_name, &err);

            let err_msg = Value::String(err.to_string());
            tpl_ctx.entry(cmd_name).append::<Value>(err_msg);
        }
    }

    debug!("current tpl_ctx: {:?}", tpl_ctx);
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
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    World::run("tests/features/generator.feature").await;
}
