use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::vault::{RealVaultProvider, VaultExportBusiness, VaultClient, VaultProvider};

pub async fn run() -> Result<()> {
    let cli = Cli::load();
    let config = AppConfig::load(&cli.config)?;

    let vault_provider = RealVaultProvider::new(&config.vault).await?;
    let vault_client = VaultClient::new(
        Box::new(vault_provider) as Box<dyn VaultProvider>,
        config.vault.mount,
    );

    run_with_vault_client(cli, vault_client).await
}

async fn run_with_vault_client(
    cli: Cli,
    vault_client: VaultClient<Box<dyn VaultProvider>>,
) -> Result<()> {
    match cli.command {
        Commands::Find(args) => {
            let result = vault_client.find(&args.path, &args.key).await?;
            println!("{result:}");
        }

        Commands::Export(args) => {
            let business = VaultExportBusiness::new(vault_client);
            business.export(&args.path).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::FindArgs;
    use mockall::mock;
    use std::collections::HashMap;

    mock! {
        pub VaultProvider {}

        #[async_trait::async_trait]
        impl VaultProvider for VaultProvider {
            async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>>;
            async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>>;
        }
    }

    fn create_cli_for_find() -> Cli {
        Cli {
            config: "/tmp/config.yaml".to_string(),
            command: Commands::Find(FindArgs {
                path: "secret/data/mysql".to_string(),
                key: "password".to_string(),
            }),
        }
    }

    #[tokio::test]
    async fn test_run_find_with_vault_client_success() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider
            .expect_read_secret()
            .with(
                mockall::predicate::eq("secret"),
                mockall::predicate::eq("secret/data/mysql"),
            )
            .times(1)
            .returning(|_, _| {
                let mut map = HashMap::new();
                map.insert("password".to_string(), "my_password".to_string());
                Ok(map)
            });

        let client = VaultClient::new(
            Box::new(mock_provider) as Box<dyn VaultProvider>,
            "secret".to_string(),
        );

        let cli = create_cli_for_find();
        let result = run_with_vault_client(cli, client).await;
        assert!(result.is_ok());
    }

    // #[tokio::test]
    // async fn test_run_list_paths_with_vault_client_success() {
    //     let mut mock_provider = MockVaultProvider::new();
    //
    //     mock_provider
    //         .expect_list_paths()
    //         .with(
    //             mockall::predicate::eq("secret"),
    //             mockall::predicate::eq("my-path"),
    //         )
    //         .times(1)
    //         .returning(|_, _| {
    //             let mut list = Vec::new();
    //             list.push("my-sub-path-1".to_string());
    //             list.push("my-sub-path-2".to_string());
    //             Ok(list)
    //         });
    //
    //     let client = VaultClient::new(
    //         Box::new(mock_provider) as Box<dyn VaultProvider>,
    //         "secret".to_string(),
    //     );
    //
    //     let cli = create_cli_for_find();
    //     let result = run_with_vault_client(cli, client).await;
    //     assert!(result.is_ok());
    // }

    #[tokio::test]
    async fn test_run_with_vault_client_error() {
        let mut mock_provider = MockVaultProvider::new();
        mock_provider
            .expect_read_secret()
            .times(1)
            .returning(|_, _| {
                anyhow::bail!("Vault connection error");
            });

        let client = VaultClient::new(
            Box::new(mock_provider) as Box<dyn VaultProvider>,
            "secret".to_string(),
        );
        let cli = create_cli_for_find();
        let result = run_with_vault_client(cli, client).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Vault connection error");
    }
}
