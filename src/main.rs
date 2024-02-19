use anyhow::Result;
use bitseed::BitseedCli;
use clap::{Parser, Subcommand};

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = BitseedCli::parse();
    let result = bitseed::run(cli);
    match result {
        Ok(ouput) => {
            println!("{}", ouput);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
