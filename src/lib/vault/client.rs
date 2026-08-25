use std::collections::HashMap;
use std::ops::Deref;

use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::kv2;
use vaultrs_login::LoginClient;
use vaultrs_login::engines::userpass::UserpassLogin;

use crate::vault::VaultConfig;

#[async_trait]
pub trait VaultProvider: Send + Sync {
    async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>>;
    async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>>;
}

pub struct RealVaultProvider {
    client: vaultrs::client::VaultClient,
}

impl RealVaultProvider {
    pub async fn new(config: &VaultConfig) -> Result<Self> {
        let client_builder = if let Some(token) = config.token.clone() {
            VaultClientSettingsBuilder::default()
                .address(config.address.clone())
                .token(token.deref().to_string())
                .build()
                .context("Failed to build Vault settings")?
        } else {
            VaultClientSettingsBuilder::default()
                .address(config.address.clone())
                .build()
                .context("Failed to build Vault settings")?
        };

        let mut client = vaultrs::client::VaultClient::new(client_builder)
            .context("Failed to create Vault client")?;

        if let Some(username) = config.username.clone()
            && let Some(password) = config.password.clone()
        {
            let login = UserpassLogin {
                username,
                password: password.deref().to_string(),
            };

            client
                .login("userpass", &login)
                .await
                .context("Failed to login to Vault")?;
        }

        tracing::debug!("Vault connection initialized");

        Ok(Self { client })
    }
}

#[async_trait]
impl VaultProvider for RealVaultProvider {
    async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>> {
        let result: HashMap<String, String> = kv2::read(&self.client, mount, path).await?;
        Ok(result)
    }

    async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        let result = kv2::list(&self.client, mount, path).await?;
        Ok(result)
    }
}

#[async_trait]
impl VaultProvider for Box<dyn VaultProvider> {
    async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>> {
        self.as_ref().read_secret(mount, path).await
    }

    async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        self.as_ref().list_paths(mount, path).await
    }
}

pub struct VaultClient<T: VaultProvider> {
    provider: T,
    mount: String,
}

impl<T: VaultProvider> VaultClient<T> {
    pub fn new(provider: T, mount: String) -> Self {
        Self { provider, mount }
    }

    pub async fn find(&self, path: &str, key: &str) -> Result<String> {
        let result = self.provider.read_secret(&self.mount, path).await?;

        let value = result
            .get(key)
            .with_context(|| format!("Key {key} not found"))?
            .clone();

        let encoded = STANDARD.encode(value.as_bytes());
        Ok(encoded)
    }

    pub async fn list_paths(&self, path: &str) -> Result<Vec<String>> {
        let result = self
            .provider
            .list_paths(&self.mount, path)
            .await
            .with_context(|| format!("Failed to list {path} path"))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub VaultProvider {}

        #[async_trait::async_trait]
        impl VaultProvider for VaultProvider {
            async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>>;
            async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>>;
        }
    }

    #[tokio::test]
    async fn test_find_success() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider
            .expect_read_secret()
            .with(
                mockall::predicate::eq("secret"),
                mockall::predicate::eq("my-path"),
            )
            .times(1)
            .returning(|_, _| {
                let mut map = HashMap::new();
                map.insert("my-key".to_string(), "my-value".to_string());
                Ok(map)
            });

        let client = VaultClient::new(mock_provider, "secret".to_string());
        let result = client.find("my-path", "my-key").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bXktdmFsdWU=");
    }

    #[tokio::test]
    async fn test_list_paths_success() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider
            .expect_list_paths()
            .with(
                mockall::predicate::eq("secret"),
                mockall::predicate::eq("my-path"),
            )
            .times(1)
            .returning(|_, _| {
                let mut list = Vec::new();
                list.push("my-sub-path-1".to_string());
                list.push("my-sub-path-2".to_string());
                Ok(list)
            });

        let client = VaultClient::new(mock_provider, "secret".to_string());
        let result = client.list_paths("my-path").await;
        let expected = vec!["my-sub-path-1".to_string(), "my-sub-path-2".to_string()];

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn test_find_key_not_found() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider.expect_read_secret().returning(|_, _| {
            let map = HashMap::new();
            Ok(map)
        });

        let client = VaultClient::new(mock_provider, "secret".to_string());
        let result = client.find("my-path", "missing-key").await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Key missing-key not found")
        );
    }

    #[tokio::test]
    async fn test_list_paths_not_found_success() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider
            .expect_list_paths()
            .returning(|_, _| Err(anyhow::anyhow!("Failed to list my-path path")));

        let client = VaultClient::new(mock_provider, "secret".to_string());
        let result = client.list_paths("my-path").await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to list my-path path")
        );
    }

    #[tokio::test]
    async fn test_find_provider_error() {
        let mut mock_provider = MockVaultProvider::new();

        mock_provider.expect_read_secret().returning(|_, _| {
            anyhow::bail!("Vault connection error");
        });

        let client = VaultClient::new(mock_provider, "secret".to_string());
        let result = client.find("my-path", "my-key").await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Vault connection error")
        );
    }
}
