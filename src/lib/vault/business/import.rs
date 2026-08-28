use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

use crate::FILE_EXTENSION;
use crate::secret::{EncryptedSecret, Secret};
use crate::vault::VaultConnectionConfig;
use crate::vault::client::{VaultClientKv2, VaultClientTrait};
use crate::vault::model::VaultData;

pub struct VaultImportBusiness {
    client: VaultClientKv2,
}

impl VaultImportBusiness {
    pub async fn new(connection: VaultConnectionConfig, request_interval_ms: u64) -> Result<Self> {
        let client = VaultClientKv2::new(connection, false, request_interval_ms).await?;
        Ok(Self { client })
    }

    pub async fn import(
        &self,
        mount: &str,
        input_folder: &Path,
        encryption_passphrase: Secret,
    ) -> Result<()> {
        self.check_folder(input_folder)?;

        let file_path = input_folder.join(format!("{mount}{FILE_EXTENSION}"));
        let mut file_content = String::new();

        std::fs::File::open(&file_path)
            .with_context(|| format!("Failed to open file: {file_path:?}"))?
            .read_to_string(&mut file_content)
            .with_context(|| format!("Failed to read file content: {file_path:?}"))?;

        let file_content: EncryptedSecret = serde_json::from_str(&file_content)
            .with_context(|| format!("Failed to parse json content: {file_path:?}"))?;
        let file_content = EncryptedSecret::decrypt(&file_content, encryption_passphrase)?;
        let data = serde_json::from_str(&file_content)?;

        self.import_data(mount, data).await
    }

    async fn import_data(&self, mount: &str, data: Vec<VaultData>) -> Result<()> {
        self.client
            .set_all(mount, data.clone())
            .await
            .inspect(|_| tracing::debug!("Secrets imported to {mount}"))?;

        self.client
            .set_all_metadata(mount, data)
            .await
            .inspect(|_| tracing::debug!("Metadata imported to {mount}"))?;

        Ok(())
    }

    fn check_folder(&self, folder: &Path) -> Result<()> {
        if folder.exists() && !folder.is_dir() {
            anyhow::bail!("Input folder already exist and is not a directory");
        }

        if !folder.exists() {
            anyhow::bail!("Input folder does not exist");
        }

        Ok(())
    }
}
