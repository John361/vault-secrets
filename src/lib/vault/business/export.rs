use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::FILE_EXTENSION;
use crate::cli::SecretEngineType;
use crate::secret::{EncryptedSecret, Secret};
use crate::vault::client::{
    VaultClientCubbyhole, VaultClientEngine, VaultClientKv1, VaultClientKv2, VaultClientTrait,
};
use crate::vault::model::VaultData;
use crate::vault::{VaultConnectionConfig, VaultMountConfig};

pub struct VaultExportBusiness {
    connection: VaultConnectionConfig,
    request_interval_ms: u64,
}

impl VaultExportBusiness {
    pub async fn new(connection: VaultConnectionConfig, request_interval_ms: u64) -> Result<Self> {
        Ok(Self {
            connection,
            request_interval_ms,
        })
    }

    pub async fn export(
        &self,
        mount: &VaultMountConfig,
        root_path: &str,
        output_folder: &PathBuf,
        encryption_passphrase: Secret,
    ) -> Result<()> {
        self.check_folder(output_folder)?;

        let result = self
            .export_data(mount, root_path)
            .await
            .inspect(|_| tracing::debug!("Secrets from {root_path} exported"))?;
        let result = serde_json::to_string_pretty(&result)?;
        let result = EncryptedSecret::encrypt(result, encryption_passphrase)?;

        let file_path = output_folder.join(format!("{}{FILE_EXTENSION}", mount.name));
        let file = std::fs::File::create(&file_path)
            .with_context(|| format!("Failed to create file: {file_path:?}"))?;

        serde_json::to_writer_pretty(file, &result)
            .with_context(|| format!("Failed to write to file: {file_path:?}"))?;

        tracing::debug!("Exported secrets written in file");
        Ok(())
    }

    async fn export_data(
        &self,
        mount: &VaultMountConfig,
        root_path: &str,
    ) -> Result<Vec<VaultData>> {
        let mut results = Vec::new();
        let mut stack = vec![root_path.to_string()];

        let client: VaultClientEngine = match mount.engine {
            SecretEngineType::Kv1 => VaultClientEngine::Kv1(
                VaultClientKv1::new(&self.connection, false, self.request_interval_ms).await?,
            ),

            SecretEngineType::Kv2 => VaultClientEngine::Kv2(
                VaultClientKv2::new(&self.connection, false, self.request_interval_ms).await?,
            ),

            SecretEngineType::Cubbyhole => VaultClientEngine::Cubbyhole(
                VaultClientCubbyhole::new(&self.connection, false, self.request_interval_ms)
                    .await?,
            ),
        };

        while let Some(current_path) = stack.pop() {
            let items = match &client {
                VaultClientEngine::Kv1(item) => item.list_paths(&mount.name, &current_path).await?,
                VaultClientEngine::Kv2(item) => item.list_paths(&mount.name, &current_path).await?,
                VaultClientEngine::Cubbyhole(item) => {
                    item.list_paths(&mount.name, &current_path).await?
                }
            };

            for item in items {
                let full_path = if current_path.ends_with("/") {
                    format!("{}{}", current_path, item)
                } else {
                    format!("{}/{}", current_path, item)
                };

                if item.ends_with("/") {
                    stack.push(full_path);
                } else {
                    match &client {
                        VaultClientEngine::Kv1(item) => {
                            let data = item.find_all(&mount.name, &full_path).await?;
                            let metadata = item.find_all_metadata(&mount.name, &full_path).await?;
                            let cleaned_path = &full_path[1..];

                            results.push(VaultData::new(cleaned_path.to_string(), data, metadata));
                        }

                        VaultClientEngine::Kv2(item) => {
                            let data = item.find_all(&mount.name, &full_path).await?;
                            let metadata = item.find_all_metadata(&mount.name, &full_path).await?;
                            let cleaned_path = &full_path[1..];

                            results.push(VaultData::new(cleaned_path.to_string(), data, metadata));
                        }

                        VaultClientEngine::Cubbyhole(item) => {
                            let data = item.find_all(&mount.name, &full_path).await?;
                            let metadata = item.find_all_metadata(&mount.name, &full_path).await?;
                            let cleaned_path = &full_path[1..];

                            results.push(VaultData::new(cleaned_path.to_string(), data, metadata));
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn check_folder(&self, folder: &PathBuf) -> Result<()> {
        if folder.exists() && !folder.is_dir() {
            anyhow::bail!("Output folder already exist and is not a directory");
        }

        if !folder.exists() {
            std::fs::create_dir_all(folder)
                .with_context(|| format!("Could not create output folder {}", folder.display()))?;
        }

        Ok(())
    }
}
