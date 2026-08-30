use std::ops::Deref;

use anyhow::Result;

use crate::cli::SecretEngineType;
use crate::vault::client::{VaultClientEngine, VaultClientKv1, VaultClientTrait};
use crate::vault::{VaultConnectionConfig, VaultFindConfig};

pub struct VaultFindBusiness {
    connection: VaultConnectionConfig,
    config: VaultFindConfig,
    request_interval_ms: u64,
}

impl VaultFindBusiness {
    pub async fn new(
        connection: VaultConnectionConfig,
        config: VaultFindConfig,
        request_interval_ms: u64,
    ) -> Result<Self> {
        Ok(Self {
            connection,
            config,
            request_interval_ms,
        })
    }

    pub async fn find(
        &self,
        mount: &str,
        path: &str,
        key: &str,
        engine: &SecretEngineType,
    ) -> Result<String> {
        let client = VaultClientKv1::build_client(
            engine,
            &self.connection,
            self.config.encode,
            self.request_interval_ms,
        )
        .await?;

        match client {
            VaultClientEngine::Kv1(client) => {
                let result = client
                    .find(mount, path, key)
                    .await
                    .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;

                Ok(result.deref().to_string())
            }

            VaultClientEngine::Kv2(client) => {
                let result = client
                    .find(mount, path, key)
                    .await
                    .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;

                Ok(result.deref().to_string())
            }

            VaultClientEngine::Cubbyhole(client) => {
                let result = client
                    .find(mount, path, key)
                    .await
                    .inspect(|_| tracing::debug!("Secret for {key} found at {path}"))?;

                Ok(result.deref().to_string())
            }
        }
    }
}
