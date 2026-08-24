use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{RealVaultProvider, VaultClient, VaultProvider};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    let vault_provider = RealVaultProvider::new(&config.vault).await?;
    let vault_client = VaultClient::new(vault_provider, config.vault.mount);

    run_with_vault_client(cli, vault_client).await
}

async fn run_with_vault_client(cli: Cli, vault_client: VaultClient<impl VaultProvider>) -> Result<()> {
    match cli.command {
        Commands::Find(args) => {
            let result = vault_client.find(&args.path, &args.key).await?;
            println!("{result:}");
        }
    }

    Ok(())
}
