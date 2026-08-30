use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::sys::mount;
use vaultrs_login::LoginClient;
use vaultrs_login::engines::userpass::UserpassLogin;

use crate::cli::SecretEngineType;
use crate::secret::Secret;
use crate::vault::client::{VaultClientCubbyhole, VaultClientKv1, VaultClientKv2};
use crate::vault::model::VaultData;
use crate::vault::{VaultConnectionConfig, VaultConnectionModeConfig};

pub trait VaultClientTrait: Sized {
    async fn build_client(
        engine: &SecretEngineType,
        connection: &VaultConnectionConfig,
        encode: bool,
        request_interval_ms: u64,
    ) -> anyhow::Result<VaultClientEngine> {
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

        let client =
            match engine {
                SecretEngineType::Kv1 => {
                    VaultClientEngine::Kv1(VaultClientKv1::new(client, encode, request_interval_ms))
                }

                SecretEngineType::Kv2 => {
                    VaultClientEngine::Kv2(VaultClientKv2::new(client, encode, request_interval_ms))
                }

                SecretEngineType::Cubbyhole => VaultClientEngine::Cubbyhole(
                    VaultClientCubbyhole::new(client, encode, request_interval_ms),
                ),
            };

        tracing::debug!("Vault connection initialized");
        Ok(client)
    }

    async fn find(&self, mount: &str, path: &str, key: &str) -> anyhow::Result<Secret> {
        let result = self.find_all(mount, path).await?;
        let value = result
            .get(key)
            .with_context(|| format!("Key {key} not found"))?
            .clone();

        Ok(value)
    }

    async fn create_mount_if_not_exists(
        &self,
        client: &vaultrs::client::VaultClient,
        mount: &str,
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<HashMap<String, String>> {
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

    async fn find_all(&self, mount: &str, path: &str) -> anyhow::Result<HashMap<String, Secret>>;
    async fn find_all_metadata(
        &self,
        mount: &str,
        path: &str,
    ) -> anyhow::Result<HashMap<String, Secret>>;
    async fn list_paths(&self, mount: &str, path: &str) -> anyhow::Result<Vec<String>>;
    async fn set_all(&self, mount: &str, data_list: Vec<VaultData>) -> anyhow::Result<()>;
    async fn set_all_metadata(&self, mount: &str, data_list: Vec<VaultData>) -> anyhow::Result<()>;
}

pub enum VaultClientEngine {
    Cubbyhole(VaultClientCubbyhole),
    Kv1(VaultClientKv1),
    Kv2(VaultClientKv2),
}
