use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub username: Option<String>,
    pub password: Option<Secret>,
    pub token: Option<Secret>,
}

#[derive(Deserialize)]
pub struct VaultFindConfig {
    pub mount: String,
}

#[derive(Deserialize)]
pub struct VaultExportConfig {
    pub sleep: u16,
    pub mounts: Vec<String>,
}

#[derive(Deserialize)]
pub struct VaultImportConfig {
    pub sleep: u16,
    pub mounts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn test_deserialize_vault_config_json() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin",
            "password": "my_secret_password",
            "token": "my_secret_token"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.address, "http://localhost:8200");
        assert_eq!(config.username, Some("admin".to_string()));
        assert_eq!(config.password.unwrap().deref(), "my_secret_password");
        assert_eq!(config.token.unwrap().deref(), "my_secret_token");
    }

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
    fn test_deserialize_vault_export_config_json() {
        let json = r#"
        {
            "sleep": 3,
            "mounts": ["my-path-1", "my-path-2"]
        }
        "#;

        let config: VaultExportConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.sleep, 3);
        assert_eq!(config.mounts, vec!["my-path-1", "my-path-2"]);
    }

    #[test]
    fn test_deserialize_vault_import_config_json() {
        let json = r#"
        {
            "sleep": 3,
            "mounts": ["my-path-1", "my-path-2"]
        }
        "#;

        let config: VaultImportConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.sleep, 3);
        assert_eq!(config.mounts, vec!["my-path-1", "my-path-2"]);
    }

    #[test]
    fn test_deserialize_vault_config_username_password() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin",
            "password": "changeme",
            "mount": ["secret"]
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.password.unwrap().deref(), "changeme");
    }

    #[test]
    fn test_deserialize_vault_config_token() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "token": "changeme",
            "mount": ["secret"]
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.token.unwrap().deref(), "changeme");
    }

    #[test]
    fn test_password_is_secret_type() {
        let config = VaultConfig {
            address: "http://localhost:8200".to_string(),
            username: Some("admin".to_string()),
            password: Some(Secret::new("top_secret".to_string())),
            token: None,
        };

        assert_eq!(config.password.unwrap().deref(), "top_secret");
    }

    #[test]
    fn test_token_is_secret_type() {
        let config = VaultConfig {
            address: "http://localhost:8200".to_string(),
            username: Some("admin".to_string()),
            password: None,
            token: Some(Secret::new("top_secret".to_string())),
        };

        assert_eq!(config.token.unwrap().deref(), "top_secret");
    }

    #[test]
    fn test_deserialize_vault_config_missing_field() {
        let json = r#"
        {
            "username": "admin"
        }
        "#;

        let result = serde_json::from_str::<VaultConfig>(json);
        assert!(result.is_err());
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

    #[test]
    fn test_deserialize_vault_export_config_missing_field() {
        let json = r#"
        {
            "sleep": 3
        }
        "#;

        let result = serde_json::from_str::<VaultExportConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_vault_import_config_missing_field() {
        let json = r#"
        {
            "sleep": 3
        }
        "#;

        let result = serde_json::from_str::<VaultImportConfig>(json);
        assert!(result.is_err());
    }
}
