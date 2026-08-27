use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::FILE_EXTENSION;
use crate::secret::{EncryptedSecret, Secret};
use crate::vault::VaultConnectionConfig;
use crate::vault::client::VaultClient;
use crate::vault::model::VaultData;

pub struct VaultExportBusiness {
    client: VaultClient,
}

impl VaultExportBusiness {
    pub async fn new(connection: VaultConnectionConfig, request_interval_ms: u64) -> Result<Self> {
        let client = VaultClient::new(connection, false, request_interval_ms).await?;
        Ok(Self { client })
    }

    pub async fn export(
        &self,
        mount: &str,
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

        let file_path = output_folder.join(format!("{mount}{FILE_EXTENSION}"));
        let file = std::fs::File::create(&file_path)
            .with_context(|| format!("Failed to create file: {file_path:?}"))?;

        serde_json::to_writer_pretty(file, &result)
            .with_context(|| format!("Failed to write to file: {file_path:?}"))?;

        tracing::debug!("Exported secrets written in file");
        Ok(())
    }

    async fn export_data(&self, mount: &str, root_path: &str) -> Result<Vec<VaultData>> {
        let mut results = Vec::new();
        let mut stack = vec![root_path.to_string()];

        while let Some(current_path) = stack.pop() {
            let items = self
                .client
                .list_paths(mount, &current_path)
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
                    match self.client.find_all(mount, &full_path).await {
                        Ok(secret_data) => {
                            let cleaned_path = &full_path[1..];
                            results.push(VaultData::new(cleaned_path.to_string(), secret_data));
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
