use std::collections::HashMap;

use anyhow::{Context, Result};
use vaultrs::api::kv2::requests::SetSecretMetadataRequestBuilder;
use vaultrs::kv2;

use crate::secret::Secret;
use crate::vault::VaultConnectionConfig;
use crate::vault::client::VaultClientTrait;
use crate::vault::model::VaultData;

pub struct VaultClientKv2 {
    client: vaultrs::client::VaultClient,
    encode: bool,
    request_interval_ms: u64,
}

impl VaultClientKv2 {
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

impl VaultClientTrait for VaultClientKv2 {
    async fn find_all(&self, mount: &str, path: &str) -> Result<HashMap<String, Secret>> {
        let raw_results: HashMap<String, String> = kv2::read(&self.client, mount, path)
            .await
            .with_context(|| format!("Error reading path {path}"))?;
        let results = Self::encode_secrets(raw_results, self.encode);

        Self::sleep(self.request_interval_ms).await;
        Ok(results)
    }

    async fn find_all_metadata(&self, mount: &str, path: &str) -> Result<HashMap<String, Secret>> {
        let raw_results = kv2::read_metadata(&self.client, mount, path)
            .await
            .with_context(|| format!("Error reading metadata path {path}"))?;
        let mut results = HashMap::new();

        if let Some(metadata) = raw_results.custom_metadata {
            results = Self::encode_secrets(metadata, self.encode);
        }

        Self::sleep(self.request_interval_ms).await;
        Ok(results)
    }

    async fn list_paths(&self, mount: &str, path: &str) -> Result<Vec<String>> {
        let result = kv2::list(&self.client, mount, path)
            .await
            .with_context(|| format!("Error listing path {path}"))?;

        Self::sleep(self.request_interval_ms).await;
        Ok(result)
    }

    async fn set_all(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(&self.client, mount).await?;

        for item in data_list.iter_mut() {
            let data = Self::decode_secrets(item.data.clone(), self.encode)?;

            kv2::set(&self.client, mount, &item.path, &data).await?;
            Self::sleep(self.request_interval_ms).await;
        }

        Ok(())
    }

    async fn set_all_metadata(&self, mount: &str, mut data_list: Vec<VaultData>) -> Result<()> {
        self.create_mount_if_not_exists(&self.client, mount).await?;

        for item in data_list.iter_mut() {
            let metadata = Self::decode_secrets(item.metadata.clone(), self.encode)?;
            let mut builder = SetSecretMetadataRequestBuilder::default();
            builder.custom_metadata(metadata);

            kv2::set_metadata(&self.client, mount, &item.path, Some(&mut builder))
                .await
                .with_context(|| format!("Error setting metadata for {mount} {}", item.path))?;
            Self::sleep(self.request_interval_ms).await;
        }

        Ok(())
    }
}
