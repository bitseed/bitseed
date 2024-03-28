use bitseed::BitseedCli;
use clap::Parser;
use std::io;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = BitseedCli::parse();
    let result = bitseed::run(cli);

    if cfg!(debug_assertions) {
        result.unwrap();
    } else {
        match result {
            Ok(output) => {
                let mut stdout = io::stdout();
                output.print_json(&mut stdout, true);
                println!();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
