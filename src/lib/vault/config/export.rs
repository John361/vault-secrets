use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultExportConfig {
    pub mounts: Vec<String>,
}
