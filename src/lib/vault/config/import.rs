use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultImportConfig {
    pub mounts: Vec<String>,
}
