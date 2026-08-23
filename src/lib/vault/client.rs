use std::collections::HashMap;
use std::ops::Deref;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::kv2;
use vaultrs_login::engines::userpass::UserpassLogin;
use vaultrs_login::LoginClient;

use crate::vault::VaultConfig;

pub struct VaultClient {
    client: vaultrs::client::VaultClient,
    mount: String,
}

impl VaultClient {
    pub async fn new(config: VaultConfig) -> Self {
        let mut client = vaultrs::client::VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(config.address)
                .build()
                .unwrap_or_else(|_| panic!("Failed to build Vault")),
        )
            .unwrap_or_else(|_| panic!("Failed to create Vault client"));

        let login = UserpassLogin {
            username: config.username,
            password: config.password.deref().to_string(),
        };

        client
            .login("userpass", &login)
            .await
            .inspect(|_| tracing::debug!("Vault connection initialized"))
            .unwrap_or_else(|_| panic!("Failed to login to Vault"));

        Self {
            client,
            mount: config.mount,
        }
    }

    pub async fn find(&self, path: &str, key: &str) -> Result<String> {
        let result: HashMap<String, String> = kv2::read(&self.client, &self.mount, path).await?;

        let result = result
            .get(key)
            .with_context(|| format!("Key {key:} not found"))?
            .clone();

        let result = STANDARD.encode(result.as_bytes());

        tracing::debug!("Key {key:} found");
        Ok(result)
    }
}
