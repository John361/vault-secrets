use std::ops::Deref;

use anyhow::Result;

use crate::cli::SecretEngineType;
use crate::vault::client::VaultClient;
use crate::vault::{VaultConnectionConfig, VaultFindConfig};

pub struct VaultFindBusiness {
    connection: VaultConnectionConfig,
    config: VaultFindConfig,
    request_interval_ms: u64,
}

impl VaultFindBusiness {
    pub async fn new(
        connection: VaultConnectionConfig,
        config: VaultFindConfig,
        request_interval_ms: u64,
    ) -> Result<Self> {
        Ok(Self {
            connection,
            config,
            request_interval_ms,
        })
    }

    pub async fn find(
        &self,
        mount: &str,
        path: &str,
        key: &str,
        engine: &SecretEngineType,
    ) -> Result<String> {
        let client = VaultClient::new(
            &self.connection,
            engine.clone(),
            self.config.encode,
            self.request_interval_ms,
        )
        .await?;
        let result = client
            .find(mount, path, key)
            .await
            .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;

        Ok(result.deref().to_string())
    }
}
