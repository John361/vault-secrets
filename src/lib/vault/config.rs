use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub username: String,
    pub password: Secret,
    pub mount: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::ops::Deref;

    #[test]
    fn test_deserialize_vault_config_json() {
        let json = r#"
        {
            "address": "http://localhost:8200",
            "username": "admin",
            "password": "my_secret_password",
            "mount": "secret"
        }
        "#;

        let config: VaultConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.address, "http://localhost:8200");
        assert_eq!(config.username, "admin");

        assert_eq!(config.password.deref(), "my_secret_password");
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

        assert_eq!(config.password.deref(), "");
    }

    #[test]
    fn test_password_is_secret_type() {
        let config = VaultConfig {
            address: "http://localhost:8200".to_string(),
            username: "admin".to_string(),
            password: Secret::new("top_secret".to_string()),
            mount: "secret".to_string(),
        };

        assert_eq!(config.password.deref(), "top_secret");
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
