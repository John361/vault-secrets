use anyhow::Result;

use lib_vault_secrets::cli::{Cli, Commands};
use lib_vault_secrets::config::AppConfig;
use lib_vault_secrets::vault::{RealVaultProvider, VaultClient};

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(e) = run().await {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    let vault_provider = RealVaultProvider::new(&config.vault).await?;
    let vault_client = VaultClient::new(vault_provider, config.vault.mount);

    match cli.command {
        Commands::Find(args) => {
            let result = vault_client.find(&args.path, &args.key).await?;
            println!("{result:}");
        }
    }

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vaultrs::client=warn".parse().unwrap())
                .add_directive("rustify=warn".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap())
                .add_directive("hyper_util=warn".parse().unwrap()),
        )
        .init();

    tracing::debug!("Tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_init_tracing() {
        let _ = panic::catch_unwind(init_tracing);
    }
}
