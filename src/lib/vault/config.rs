use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub username: Option<String>,
    pub password: Option<Secret>,
    pub token: Option<Secret>,
    pub mount: String,
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
            "token": "my_secret_token",
            "mount": "secret"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.address, "http://localhost:8200");
        assert_eq!(config.username, Some("admin".to_string()));
        assert_eq!(config.password.unwrap().deref(), "my_secret_password");
        assert_eq!(config.token.unwrap().deref(), "my_secret_token");
        assert_eq!(config.mount, "secret");
    }

    #[test]
    fn test_deserialize_vault_config_empty_password() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin",
            "password": "",
            "mount": "secret"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.password.unwrap().deref(), "");
    }

    #[test]
    fn test_deserialize_vault_config_empty_token() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "token": "",
            "mount": "secret"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.token.unwrap().deref(), "");
    }

    #[test]
    fn test_password_is_secret_type() {
        let config = VaultConfig {
            address: "http://localhost:8200".to_string(),
            username: Some("admin".to_string()),
            password: Some(Secret::new("top_secret".to_string())),
            token: None,
            mount: "secret".to_string(),
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
            mount: "secret".to_string(),
        };

        assert_eq!(config.token.unwrap().deref(), "top_secret");
    }

    #[test]
    fn test_deserialize_vault_config_missing_field() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin"
        }
        "#;

        let result = serde_json::from_str::<VaultConfig>(json);
        assert!(result.is_err());
    }
}
