use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::api::kv2::requests::SetSecretMetadataRequestBuilder;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::sys::mount;
use vaultrs::{cubbyhole, kv1, kv2};
use vaultrs_login::LoginClient;
use vaultrs_login::engines::userpass::UserpassLogin;

use crate::cli::SecretEngineType;
use crate::secret::Secret;
use crate::vault::model::VaultData;
use crate::vault::{VaultConnectionConfig, VaultConnectionModeConfig};

pub struct VaultClient {
    client: vaultrs::client::VaultClient,
    engine: SecretEngineType,
    encode: bool,
    request_interval_ms: u64,
}

impl VaultClient {
    pub async fn new(
        connection: &VaultConnectionConfig,
        engine: SecretEngineType,
        encode: bool,
        request_interval_ms: u64,
    ) -> Result<Self> {
        let client_builder = match &connection.mode {
            VaultConnectionModeConfig::Token(value) => VaultClientSettingsBuilder::default()
                .address(connection.address.clone())
                .token(value.token.deref().to_string())
                .build()
                .with_context(|| "Failed to build Vault settings")?,

            VaultConnectionModeConfig::UserPass(_) => VaultClientSettingsBuilder::default()
                .address(connection.address.clone())
                .build()
                .with_context(|| "Failed to build Vault settings")?,
        };

        let mut client = vaultrs::client::VaultClient::new(client_builder)
            .with_context(|| "Failed to create Vault client")?;

        if let VaultConnectionModeConfig::UserPass(value) = &connection.mode {
            let login = UserpassLogin {
                username: value.username.clone(),
                password: value.password.deref().to_string(),
            };

            client
                .login("userpass", &login)
                .await
                .with_context(|| "Failed to login to Vault")?;
        }

        tracing::debug!("Vault connection initialized");

        Ok(Self {
            client,
            engine,
            encode,
            request_interval_ms,
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
        let raw_results: HashMap<String, String> = match self.engine {
            SecretEngineType::Kv1 => kv1::get(&self.client, mount, path)
                .await
                .with_context(|| format!("Error reading path {path}"))?,

            SecretEngineType::Kv2 => kv2::read(&self.client, mount, path)
                .await
                .with_context(|| format!("Error reading path {path}"))?,

            SecretEngineType::Cubbyhole => cubbyhole::get(&self.client, mount, path)
                .await
                .with_context(|| format!("Error reading path {path}"))?,
        };

        let results = Self::encode_secrets(raw_results, self.encode);

        Self::sleep(self.request_interval_ms).await;
        Ok(results)
    }

    pub async fn find_all_metadata(
        &self,
        mount: &str,
        path: &str,
    ) -> Result<HashMap<String, Secret>> {
        let raw_results = match self.engine {
            SecretEngineType::Kv1 => HashMap::new(),

            SecretEngineType::Kv2 => kv2::read_metadata(&self.client, mount, path)
                .await
                .with_context(|| format!("Error reading metadata path {path}"))?
                .custom_metadata
                .unwrap_or_default(),

            SecretEngineType::Cubbyhole => HashMap::new(),
        };

        let results = Self::encode_secrets(raw_results, self.encode);

        Self::sleep(self.request_interval_ms).await;
        Ok(results)
    }

    pub async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        let result = match self.engine {
            SecretEngineType::Kv1 => {
                kv1::list(&self.client, mount, path)
                    .await
                    .with_context(|| format!("Error listing path {path}"))?
                    .data
                    .keys
            }

            SecretEngineType::Kv2 => kv2::list(&self.client, mount, path)
                .await
                .with_context(|| format!("Error listing path {path}"))?,

            SecretEngineType::Cubbyhole => {
                cubbyhole::list(&self.client, mount, path)
                    .await
                    .with_context(|| format!("Error listing path {path}"))?
                    .data
                    .keys
            }
        };

        Self::sleep(self.request_interval_ms).await;
        Ok(result)
    }

    pub async fn set_all(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(&self.client, mount).await?;

        for item in data_list.iter_mut() {
            let data = Self::decode_secrets(item.data.clone(), self.encode)?;

            match self.engine {
                SecretEngineType::Kv1 => {
                    let data: HashMap<&str, String> =
                        data.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();

                    kv1::set(&self.client, mount, &item.path, &data).await?;
                }

                SecretEngineType::Kv2 => {
                    kv2::set(&self.client, mount, &item.path, &data).await?;
                }

                SecretEngineType::Cubbyhole => {
                    let data: HashMap<&str, String> =
                        data.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();

                    cubbyhole::set(&self.client, mount, &item.path, &data).await?;
                }
            }

            Self::sleep(self.request_interval_ms).await;
        }

        Ok(())
    }

    pub async fn set_all_metadata(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(&self.client, mount).await?;

        for item in data_list.iter_mut() {
            let metadata = Self::decode_secrets(item.metadata.clone(), self.encode)?;

            match self.engine {
                SecretEngineType::Kv1 => tracing::debug!("No metadata support for kv1"),

                SecretEngineType::Kv2 => {
                    let mut builder = SetSecretMetadataRequestBuilder::default();
                    builder.custom_metadata(metadata);

                    kv2::set_metadata(&self.client, mount, &item.path, Some(&mut builder))
                        .await
                        .with_context(|| {
                            format!("Error setting metadata for {mount} {}", item.path)
                        })?;
                }

                SecretEngineType::Cubbyhole => tracing::debug!("No metadata support for cubbyhole"),
            }

            Self::sleep(self.request_interval_ms).await;
        }

        Ok(())
    }

    async fn create_mount_if_not_exists(
        &self,
        client: &vaultrs::client::VaultClient,
        mount: &str,
    ) -> Result<()> {
        let mount_exists = mount::list(client)
            .await
            .with_context(|| "Error listing mounts")?
            .contains_key(&format!("{mount}/"));

        if !mount_exists {
            mount::enable(client, mount, "kv-v2", None).await?;
            tracing::debug!("Mount {mount} enabled");
        }

        Ok(())
    }

    async fn sleep(request_interval_ms: u64) {
        let request_interval = Duration::from_millis(request_interval_ms);
        tokio::time::sleep(request_interval).await;
    }

    fn encode_secrets(mut data: HashMap<String, String>, encode: bool) -> HashMap<String, Secret> {
        let mut results = HashMap::new();

        for item in data.iter_mut() {
            if encode {
                *item.1 = STANDARD.encode(item.1.as_bytes());
            }

            let secret = Secret::new(item.1.to_string());
            results.insert(item.0.to_string(), secret);
        }

        results
    }

    fn decode_secrets(
        mut data: HashMap<String, Secret>,
        encode: bool,
    ) -> Result<HashMap<String, String>> {
        let mut results = HashMap::new();

        for item in data.iter_mut() {
            let mut value = item.1.deref().to_string();

            if encode {
                let decoded = STANDARD
                    .decode(item.1.as_bytes())
                    .with_context(|| format!("Error decoding encoded secret for {}", item.0))?;

                value = String::from_utf8(decoded)
                    .with_context(|| format!("Error converting encoded secret for {}", item.0))?;
            }

            results.insert(item.0.to_string(), value);
        }

        Ok(results)
    }
}
