use std::ops::Deref;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::vault::VaultConfig;
use crate::vault::client::VaultClient;
use crate::vault::model::VaultExportData;

pub struct VaultFindBusiness {
    client: VaultClient,
}

impl VaultFindBusiness {
    pub async fn new(config: VaultConfig, encoded: bool) -> Result<Self> {
        let client = VaultClient::new(config, encoded).await?;
        Ok(Self { client })
    }

    pub async fn find(&self, path: &str, key: &str) -> Result<String> {
        let result = self
            .client
            .find(path, key)
            .await
            .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;
        Ok(result.deref().to_string())
    }
}

pub struct VaultExportBusiness {
    client: VaultClient,
}

impl VaultExportBusiness {
    pub async fn new(config: VaultConfig, encoded: bool) -> Result<Self> {
        let client = VaultClient::new(config, encoded).await?;
        Ok(Self { client })
    }

    pub async fn export(&self, root_path: &str, output_file: PathBuf) -> Result<()> {
        let result = self
            .export_data(root_path)
            .await
            .inspect(|_| tracing::debug!("Secrets from {root_path} exported"))?;

        let file = std::fs::File::create(&output_file)
            .with_context(|| format!("Failed to create file: {:?}", output_file))?;

        serde_json::to_writer_pretty(file, &result)
            .inspect(|_| tracing::debug!("Exported secrets written in file"))
            .with_context(|| format!("Failed to write to file: {:?}", output_file))?;

        Ok(())
    }

    async fn export_data(&self, root_path: &str) -> Result<Vec<VaultExportData>> {
        let mut results = Vec::new();
        let mut stack = vec![root_path.to_string()];

        while let Some(current_path) = stack.pop() {
            let items = self
                .client
                .list_paths(&current_path)
                .await
                .inspect(|_| tracing::debug!("Listing path from {current_path}"))?;

            for item in items {
                let full_path = if current_path.ends_with("/") {
                    format!("{}{}", current_path, item)
                } else {
                    format!("{}/{}", current_path, item)
                };

                if item.ends_with("/") {
                    stack.push(full_path);
                } else {
                    match self.client.find_all(&full_path).await {
                        Ok(secret_data) => {
                            let cleaned_path = &full_path[1..];
                            results
                                .push(VaultExportData::new(cleaned_path.to_string(), secret_data));
                        }

                        Err(e) => {
                            tracing::error!("Failed to read secret at {full_path}: {e}");
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
