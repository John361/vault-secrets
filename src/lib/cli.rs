use clap::Args;
use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    name = "vault-secrets",
    bin_name = "vault-secrets"
)]
pub enum Cli {
    #[command(about = "Find secret")]
    Find(FindArgs),
}

impl Cli {
    pub fn load() -> Self {
        Cli::parse()
    }
}

#[derive(Args, Debug)]
#[command(about = "Find arguments", long_about = None)]
pub struct FindArgs {
    #[arg(long, help = "Path to secret (required)", required = true)]
    pub path: String,

    #[arg(long, help = "Secret key name (required)", required = true)]
    pub key: String,
}
