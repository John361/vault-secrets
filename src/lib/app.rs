use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultExportBusiness, VaultFindBusiness, VaultImportBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    match cli.command {
        Commands::Find(args) => {
            let business = VaultFindBusiness::new(&config.vault, !cli.clear_output).await?;
            let result = business
                .find(&config.find.mount, &args.path, &args.key)
                .await?;
            println!("{result}");
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(config.vault, !cli.clear_output).await?;

            for mount in config.export.mounts {
                business
                    .export(&mount, &args.path, &args.output_folder)
                    .await?;
            }
        }

        Commands::Import(args) => {
            let business = VaultImportBusiness::new(&config.vault, !cli.clear_output).await?;

            for mount in config.import.mounts {
                business
                    .import(&mount, &args.input_folder, &args.input_format)
                    .await?;
            }
        }
    }

    Ok(())
}
