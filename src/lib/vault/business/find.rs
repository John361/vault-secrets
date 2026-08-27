use std::ops::Deref;

use anyhow::Result;

use crate::vault::client::VaultClient;
use crate::vault::{VaultConnectionConfig, VaultGeneralConfig};

pub struct VaultFindBusiness {
    client: VaultClient,
}

impl VaultFindBusiness {
    pub async fn new(
        general: VaultGeneralConfig,
        connection: VaultConnectionConfig,
        encoded: bool,
    ) -> Result<Self> {
        let client = VaultClient::new(connection, encoded, general.request_interval_ms).await?;
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
