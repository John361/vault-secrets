use std::ops::Deref;

use anyhow::Result;

use crate::vault::client::{VaultClientKv2, VaultClientTrait};
use crate::vault::{VaultConnectionConfig, VaultFindConfig};

pub struct VaultFindBusiness {
    client: VaultClientKv2,
    config: VaultFindConfig,
}

impl VaultFindBusiness {
    pub async fn new(
        connection: VaultConnectionConfig,
        config: VaultFindConfig,
        request_interval_ms: u64,
    ) -> Result<Self> {
        let client = VaultClientKv2::new(connection, config.encode, request_interval_ms).await?;
        Ok(Self { client, config })
    }

    pub async fn find(&self, path: &str, key: &str) -> Result<String> {
        let result = self
            .client
            .find(&self.config.mount, path, key)
            .await
            .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;
        Ok(result.deref().to_string())
    }
}
