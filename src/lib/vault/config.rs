use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub username: String,
    pub password: Secret,
    pub mount: String,
}
