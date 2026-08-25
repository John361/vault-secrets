use std::collections::HashMap;

use anyhow::Result;

use crate::secret::Secret;
use crate::vault::{VaultClient, VaultExportData, VaultProvider};

pub struct VaultBusiness<T: VaultProvider> {
    client: VaultClient<T>,
}

impl<T: VaultProvider> VaultBusiness<T> {
    pub fn new(client: VaultClient<T>) -> Self {
        Self { client }
    }

    pub async fn export_vault_secrets(&self, root_path: &str) -> Result<Vec<VaultExportData>> {
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
                    match self.read_secret_data(&full_path).await {
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

    async fn read_secret_data(&self, path: &str) -> Result<HashMap<String, Secret>> {
        let raw_data = self.client.find(path, "username").await?; // TODO: fix usage by implementing find_all
        let mut secrets = HashMap::new();

        secrets.insert("value".to_string(), Secret::new(raw_data));

        Ok(secrets)
    }
}
