use std::ops::Deref;

use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultClient, VaultExportBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;
    let vault_client = VaultClient::new(config.vault, true).await?; // TODO: add global argument for encode or not and with default = true

    run_with_vault_client(cli, vault_client).await
}

async fn run_with_vault_client(cli: Cli, vault_client: VaultClient) -> Result<()> {
    match cli.command {
        Commands::Find(args) => {
            let result = vault_client.find(&args.path, &args.key).await?;
            println!("{}", result.deref());
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(vault_client);
            business.export(&args.path).await?;
        }
    }

    Ok(())
}
