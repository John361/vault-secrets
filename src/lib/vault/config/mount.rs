use serde::Deserialize;

use crate::cli::SecretEngineType;

#[derive(Deserialize)]
pub struct VaultMountConfig {
    pub name: String,
    pub engine: SecretEngineType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vault_mount_config_json() {
        let json = r#"
        {
            "name": "my-path",
            "engine": "Kv2"
        }
        "#;

        let config: VaultMountConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.name, "my-path");
        assert_eq!(config.engine, SecretEngineType::Kv2);
    }

    #[test]
    fn test_deserialize_vault_mount_config_missing_field() {
        let json = r#"
        {

        }
        "#;

        let result = serde_json::from_str::<VaultMountConfig>(json);
        assert!(result.is_err());
    }
}
