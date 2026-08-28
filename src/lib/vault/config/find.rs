use serde::Deserialize;

#[derive(Deserialize)]
pub struct VaultFindConfig {
    pub encode: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vault_find_config_json() {
        let json = r#"
        {
            "encode": true
        }
        "#;

        let config: VaultFindConfig = serde_json::from_str(json).unwrap();
        assert!(config.encode);
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
