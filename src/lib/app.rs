use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultExportBusiness, VaultFindBusiness, VaultImportBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    match cli.command {
        Commands::Find(args) => {
            let business =
                VaultFindBusiness::new(config.connection, config.find, config.request_interval_ms)
                    .await?;
            let result = business.find(&args.path, &args.key).await?;
            println!("{result}");
        }

        Commands::Export(args) => {
            let business =
                VaultExportBusiness::new(config.connection, config.request_interval_ms).await?;
            let encryption_passphrase = config.encryption_passphrase;

            for mount in config.export.mounts {
                business
                    .export(
                        &mount,
                        &args.path,
                        &args.output_folder,
                        encryption_passphrase.clone(),
                    )
                    .await?;
            }
        }

        Commands::Import(args) => {
            let business =
                VaultImportBusiness::new(config.connection, config.request_interval_ms).await?;
            let encryption_passphrase = config.encryption_passphrase;

            for mount in config.import.mounts {
                business
                    .import(&mount, &args.input_folder, encryption_passphrase.clone())
                    .await?;
            }
        }
    }

    Ok(())
}
