use bitseed::BitseedCli;
use clap::Parser;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = BitseedCli::parse();
    let result = bitseed::run(cli);
    match result {
        Ok(_) => {}
        Err(e) => {
            debug_assert!(false, "Error: {}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
