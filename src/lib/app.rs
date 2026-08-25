use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultClient, VaultExportBusiness, VaultFindBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;
    let vault_client = VaultClient::new(config.vault, !cli.clear_output).await?;

    run_with_vault_client(cli, vault_client).await
}

async fn run_with_vault_client(cli: Cli, vault_client: VaultClient) -> Result<()> {
    match cli.command {
        Commands::Find(args) => {
            let business = VaultFindBusiness::new(vault_client);
            business.find(&args.path, &args.key).await?;
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(vault_client);
            business.export(&args.path).await?;
        }
    }

    Ok(())
}
