use serde::Deserialize;

use crate::vault::VaultMountConfig;

#[derive(Deserialize)]
pub struct VaultExportConfig {
    pub mounts: Vec<VaultMountConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SecretEngineType;

    #[test]
    fn test_deserialize_vault_export_config_json() {
        let json = r#"
        {
            "mounts": [{
                "name": "my-path-1",
                "engine": "Kv1"
            }, {
                "name": "my-path-2",
                "engine": "Kv2"
            }]
        }
        "#;

        let config: VaultExportConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.mounts.len(), 2);
        assert_eq!(config.mounts[0].name, "my-path-1");
        assert_eq!(config.mounts[0].engine, SecretEngineType::Kv1);
        assert_eq!(config.mounts[1].name, "my-path-2");
        assert_eq!(config.mounts[1].engine, SecretEngineType::Kv2);
    }

    #[test]
    fn test_deserialize_vault_export_config_missing_field() {
        let json = r#"
        {

        }
        "#;

        let result = serde_json::from_str::<VaultExportConfig>(json);
        assert!(result.is_err());
    }
}
