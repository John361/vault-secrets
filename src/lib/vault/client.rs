use std::collections::HashMap;
use std::ops::Deref;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::kv2;
use vaultrs_login::LoginClient;
use vaultrs_login::engines::userpass::UserpassLogin;

use crate::vault::VaultConfig;

#[async_trait::async_trait]
pub trait VaultProvider: Send + Sync {
    async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>>;
}

pub struct RealVaultProvider {
    client: vaultrs::client::VaultClient,
}

impl RealVaultProvider {
    pub async fn new(config: &VaultConfig) -> Result<Self> {
        let mut client = vaultrs::client::VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(config.address.clone())
                .build()
                .context("Failed to build Vault settings")?,
        )
        .context("Failed to create Vault client")?;

        let login = UserpassLogin {
            username: config.username.clone(),
            password: config.password.deref().to_string(),
        };

        client
            .login("userpass", &login)
            .await
            .context("Failed to login to Vault")?;

        tracing::debug!("Vault connection initialized");

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl VaultProvider for RealVaultProvider {
    async fn read_secret(&self, mount: &str, path: &str) -> Result<HashMap<String, String>> {
        let result: HashMap<String, String> = kv2::read(&self.client, mount, path).await?;
        Ok(result)
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
