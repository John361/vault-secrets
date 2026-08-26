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
