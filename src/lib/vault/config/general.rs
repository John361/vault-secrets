use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub username: Option<String>,
    pub password: Option<Secret>,
    pub token: Option<Secret>,
    pub request_interval_ms: u64,
    pub encryption_passphrase: Secret,
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
            "request_interval_ms": 200,
            "encryption_passphrase": "my_secret_passphrase"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.address, "http://localhost:8200");
        assert_eq!(config.username, Some("admin".to_string()));
        assert_eq!(config.password.unwrap().deref(), "my_secret_password");
        assert_eq!(config.token.unwrap().deref(), "my_secret_token");
        assert_eq!(config.request_interval_ms, 200);
        assert_eq!(config.encryption_passphrase.deref(), "my_secret_passphrase");
    }

    #[test]
    fn test_deserialize_vault_config_username_password() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin",
            "password": "changeme",
            "request_interval_ms": 200,
            "encryption_passphrase": "my_secret_passphrase"
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
            "request_interval_ms": 200,
            "encryption_passphrase": "my_secret_passphrase"
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
            request_interval_ms: 200,
            encryption_passphrase: Secret::new("my_secret_passphrase".to_string()),
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
            request_interval_ms: 200,
            encryption_passphrase: Secret::new("my_secret_passphrase".to_string()),
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
}
