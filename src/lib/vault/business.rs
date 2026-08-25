use std::ops::Deref;

use anyhow::Result;

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

    pub async fn find(&self, path: &str, key: &str) -> Result<()> {
        let result = self.client.find(path, key).await?;
        println!("{}", result.deref());

        Ok(())
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

    pub async fn export(&self, root_path: &str) -> Result<()> {
        let result = self.export_data(root_path).await?;
        let result = serde_json::to_value(&result)?;

        println!("{:?}", result);
        Ok(())
    }

    async fn export_data(&self, root_path: &str) -> Result<Vec<VaultExportData>> {
        let mut results = Vec::new();
        let mut stack = vec![root_path.to_string()];

        while let Some(current_path) = stack.pop() {
            let items = self.client.list_paths(&current_path).await?;

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
                            results.push(VaultExportData::new(full_path, secret_data));
                        }

                        Err(e) => {
                            eprintln!("Failed to read secret at {full_path}: {e}");
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
