use anyhow::Result;
use serde::Deserialize;

use crate::vault::VaultConfig;

#[derive(Deserialize)]
pub struct AppConfig {
    pub vault: VaultConfig,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .unwrap_or_else(|e| panic!("Cannot get app config file path: {e:?}"));

        let configurations = config
            .try_deserialize::<AppConfig>()
            .unwrap_or_else(|e| panic!("Cannot deserialize app config: {e:?}"));

        Ok(configurations)
    }
}
