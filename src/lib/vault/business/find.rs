use std::ops::Deref;

use anyhow::Result;

use crate::vault::VaultConnectionConfig;
use crate::vault::client::VaultClient;

pub struct VaultFindBusiness {
    client: VaultClient,
}

impl VaultFindBusiness {
    pub async fn new(
        connection: VaultConnectionConfig,
        encoded: bool,
        request_interval_ms: u64,
    ) -> Result<Self> {
        let client = VaultClient::new(connection, encoded, request_interval_ms).await?;
        Ok(Self { client })
    }

    pub async fn find(&self, mount: &str, path: &str, key: &str) -> Result<String> {
        let result = self
            .client
            .find(mount, path, key)
            .await
            .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;
        Ok(result.deref().to_string())
    }
}
