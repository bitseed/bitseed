use bitseed::BitseedCli;
use clap::Parser;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = BitseedCli::parse();
    let result = bitseed::run(cli);
    if cfg!(debug_assertions) {
        result.unwrap();
    } else {
        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
