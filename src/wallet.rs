use clap::Parser;
use ord::Options;
use reqwest::Url;

#[derive(Debug, Clone, Parser)]
pub struct WalletOption {
    #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
    pub name: String,
    #[arg(long, alias = "nosync", help = "Do not update index.")]
    pub no_sync: bool,
    #[arg(
        long,
        default_value = "http://127.0.0.1:80",
        help = "Use ord running at <SERVER_URL>."
    )]
    pub server_url: Url,

    #[clap(flatten)]
    pub chain_options: Options,
}
