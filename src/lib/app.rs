use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{VaultExportBusiness, VaultFindBusiness};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    match cli.command {
        Commands::Find(args) => {
            let business = VaultFindBusiness::new(config.vault, !cli.clear_output).await?;
            let result = business.find("", &args.path, &args.key).await?; // TODO: mount
            println!("{result}");
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(config.vault, !cli.clear_output).await?;
            business
                .export("", &args.path, args.output_file, args.output_format) // TODO: mount
                .await?;
        }
    }

    Ok(())
}
