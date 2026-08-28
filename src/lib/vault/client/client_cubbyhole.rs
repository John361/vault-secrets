use std::collections::HashMap;

use anyhow::{Context, Result};
use vaultrs::cubbyhole;

use crate::secret::Secret;
use crate::vault::VaultConnectionConfig;
use crate::vault::client::VaultClientTrait;
use crate::vault::model::VaultData;

pub struct VaultClientCubbyhole {
    client: vaultrs::client::VaultClient,
    encode: bool,
    request_interval_ms: u64,
}

impl VaultClientCubbyhole {
    pub async fn new(
        connection: &VaultConnectionConfig,
        encode: bool,
        request_interval_ms: u64,
    ) -> Result<Self> {
        let client = Self::build_client(connection).await?;

        Ok(Self {
            client,
            encode,
            request_interval_ms,
        })
    }
}

impl VaultClientTrait for VaultClientCubbyhole {
    async fn find_all(&self, mount: &str, path: &str) -> Result<HashMap<String, Secret>> {
        let raw_results: HashMap<String, String> = cubbyhole::get(&self.client, mount, path)
            .await
            .with_context(|| format!("Error reading path {path}"))?;
        let results = Self::encode_secrets(raw_results, self.encode);

        Self::sleep(self.request_interval_ms).await;
        Ok(results)
    }

    async fn find_all_metadata(&self, _: &str, _: &str) -> Result<HashMap<String, Secret>> {
        Ok(HashMap::new())
    }

    async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        let result = cubbyhole::list(&self.client, mount, path)
            .await
            .with_context(|| format!("Error listing path {path}"))?;

        Self::sleep(self.request_interval_ms).await;
        Ok(result.data.keys)
    }

    async fn set_all(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(&self.client, mount).await?;

        for item in data_list.iter_mut() {
            let data = Self::decode_secrets(item.data.clone(), self.encode)?;
            let data: HashMap<&str, String> =
                data.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();

            cubbyhole::set(&self.client, mount, &item.path, &data).await?;
            Self::sleep(self.request_interval_ms).await;
        }

        Ok(())
    }

    async fn set_all_metadata(&self, _: &str, _: Vec<VaultData>) -> Result<()> {
        Ok(())
    }
}
