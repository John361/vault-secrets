use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::kv2;
use vaultrs::sys::mount;
use vaultrs_login::LoginClient;
use vaultrs_login::engines::userpass::UserpassLogin;

use crate::secret::Secret;
use crate::vault::VaultConfig;
use crate::vault::model::VaultData;

pub struct VaultClient {
    client: vaultrs::client::VaultClient,
    encode: bool,
    request_interval_ms: u64,
}

impl VaultClient {
    pub async fn new(config: &VaultConfig, encode: bool) -> Result<Self> {
        let client_builder = if let Some(token) = config.token.clone() {
            VaultClientSettingsBuilder::default()
                .address(config.address.clone())
                .token(token.deref().to_string())
                .build()
                .with_context(|| "Failed to build Vault settings")?
        } else {
            VaultClientSettingsBuilder::default()
                .address(config.address.clone())
                .build()
                .with_context(|| "Failed to build Vault settings")?
        };

        let mut client = vaultrs::client::VaultClient::new(client_builder)
            .with_context(|| "Failed to create Vault client")?;

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
                .with_context(|| "Failed to login to Vault")?;
        }

        tracing::debug!("Vault connection initialized");
        Ok(Self {
            client,
            encode,
            request_interval_ms: config.request_interval_ms,
        })
    }

    pub async fn find(&self, mount: &str, path: &str, key: &str) -> Result<Secret> {
        let result = self.find_all(mount, path).await?;
        let value = result
            .get(key)
            .with_context(|| format!("Key {key} not found"))?
            .clone();

        Ok(value)
    }

    pub async fn find_all(&self, mount: &str, path: &str) -> Result<HashMap<String, Secret>> {
        let mut raw_results: HashMap<String, String> =
            kv2::read(&self.client, mount, path)
                .await
                .with_context(|| format!("Error reading path {path}"))?;
        let mut results = HashMap::new();

        for result in raw_results.iter_mut() {
            if self.encode {
                *result.1 = STANDARD.encode(result.1.as_bytes());
            }

            let secret = Secret::new(result.1.to_string());
            results.insert(result.0.to_string(), secret);
        }

        self.sleep().await;
        Ok(results)
    }

    pub async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        let result = kv2::list(&self.client, mount, path)
            .await
            .with_context(|| format!("Error listing path {path}"))?;

        self.sleep().await;
        Ok(result)
    }

    pub async fn set_all(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(mount).await?;

        for item in data_list.iter_mut() {
            if self.encode {
                for (key, secret) in item.data.iter_mut() {
                    let decoded = STANDARD.decode(secret.as_bytes()).with_context(|| {
                        format!("Error decoding data for key {key} on path {}", item.path)
                    })?;

                    let decoded = String::from_utf8(decoded).with_context(|| {
                        format!("Error converting data for key {key} on path {}", item.path)
                    })?;

                    *secret = Secret::new(decoded);
                }
            }

            kv2::set(&self.client, mount, &item.path, &item.data).await?;
            self.sleep().await;
        }

        Ok(())
    }

    async fn create_mount_if_not_exists(&self, mount: &str) -> Result<()> {
        let mount_exists = mount::list(&self.client)
            .await
            .with_context(|| "Error listing mounts")?
            .contains_key(&format!("{mount}/"));

        if !mount_exists {
            mount::enable(&self.client, mount, "kv-v2", None).await?;

            self.sleep().await;
            tracing::debug!("Mount {mount} enabled");
        }

        Ok(())
    }

    async fn sleep(&self) {
        let request_interval = Duration::from_millis(self.request_interval_ms);
        tokio::time::sleep(request_interval).await;
    }
}
