use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultFindConfig {
    pub mount: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vault_find_config_json() {
        let json = r#"
        {
            "mount": "my-path"
        }
        "#;

        let config: VaultFindConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.mount, "my-path");
    }

    #[test]
    fn test_deserialize_vault_find_config_missing_field() {
        let json = r#"
        {

        }
        "#;

        let result = serde_json::from_str::<VaultFindConfig>(json);
        assert!(result.is_err());
    }
}
