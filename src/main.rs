use anyhow::Result;
use bitseed::BitseedCli;
use clap::{Parser, Subcommand};

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = BitseedCli::parse();
    bitseed::run(cli).unwrap();
    // let result = bitseed::run(cli);
    // match result {
    //     Ok(_) => {}
    //     Err(e) => {
    //         eprintln!("Error: {}", e);
    //         //panic!("Error: {}", e);
    //         std::process::exit(1);
    //     }
    // }
}
