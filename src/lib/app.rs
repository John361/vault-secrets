use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultExportBusiness, VaultFindBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    match cli.command {
        Commands::Find(args) => {
            if let Some(mount) = config.vault.mount.first() {
                let business = VaultFindBusiness::new(&config.vault, !cli.clear_output).await?;
                let result = business.find(mount, &args.path, &args.key).await?;
                println!("{result}");
            } else {
                tracing::error!(
                    "At least one mount path must be provided in the configuration file"
                );
            }
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(&config.vault, !cli.clear_output).await?;

            for mount in config.vault.mount {
                business
                    .export(&mount, &args.path, &args.output_folder, &args.output_format)
                    .await?;
            }
        }
    }

    Ok(())
}
