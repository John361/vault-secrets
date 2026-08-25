use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::FormatArgs;
use crate::vault::VaultConfig;
use crate::vault::client::VaultClient;
use crate::vault::model::VaultData;

pub struct VaultImportBusiness {
    client: VaultClient,
}

impl VaultImportBusiness {
    pub async fn new(config: &VaultConfig, encoded: bool) -> Result<Self> {
        let client = VaultClient::new(config, encoded).await?;
        Ok(Self { client })
    }

    pub async fn import(
        &self,
        mount: &str,
        input_folder: &Path,
        input_format: &FormatArgs,
    ) -> Result<()> {
        self.check_folder(input_folder)?;

        let extension = match input_format {
            FormatArgs::Json => "json",
            FormatArgs::Yaml => "yaml",
        };

        let file_path = input_folder.join(format!("{mount}.{extension}"));
        let mut file_content = String::new();

        std::fs::File::open(&file_path)
            .with_context(|| format!("Failed to open file: {file_path:?}"))?
            .read_to_string(&mut file_content)
            .with_context(|| format!("Failed to read file content: {file_path:?}"))?;

        let data: Vec<VaultData> = match input_format {
            FormatArgs::Json => serde_json::from_str(&file_content)
                .with_context(|| format!("Failed to parse json content: {file_path:?}"))?,

            FormatArgs::Yaml => yaml_serde::from_str(&file_content)
                .with_context(|| format!("Failed to parse yaml content: {file_path:?}"))?,
        };

        self.import_data(mount, data).await
    }

    async fn import_data(&self, mount: &str, data: Vec<VaultData>) -> Result<()> {
        self.client
            .set_all(mount, data)
            .await
            .inspect(|_| tracing::debug!("Secrets imported to {mount}"))?;

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
