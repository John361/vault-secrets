use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultFindConfig {
    pub mount: String,
}
