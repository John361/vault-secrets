use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultImportConfig {
    pub mounts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vault_import_config_json() {
        let json = r#"
        {
            "mounts": ["my-path-1", "my-path-2"]
        }
        "#;

        let config: VaultImportConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.mounts, vec!["my-path-1", "my-path-2"]);
    }

    #[test]
    fn test_deserialize_vault_import_config_missing_field() {
        let json = r#"
        {

        }
        "#;

        let result = serde_json::from_str::<VaultImportConfig>(json);
        assert!(result.is_err());
    }
}
