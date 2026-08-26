use serde::Deserialize;

use crate::secret::Secret;

#[derive(Deserialize)]
pub struct VaultConnectionConfig {
    pub mode: VaultConnectionModeConfig,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VaultConnectionModeConfig {
    Token(VaultConnectionTokenAuthConfig),
    UserPass(VaultConnectionUserPassConfig),
}

#[derive(Deserialize)]
pub struct VaultConnectionTokenAuthConfig {
    pub token: Secret,
}

#[derive(Deserialize)]
pub struct VaultConnectionUserPassConfig {
    pub username: String,
    pub password: Secret,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;
    use crate::vault::VaultConfig;

    #[test]
    fn test_deserialize_vault_connection_config_token() {
        let json = r#"
        {
            "mode": {
                "token": "changeme"
            }
        }
        "#;

        let config: VaultConnectionConfig = serde_json::from_str(json).unwrap();
        match config.mode {
            VaultConnectionModeConfig::Token(auth) => {
                assert_eq!(auth.token.deref(), "changeme");
            }
            _ => panic!("Expected Token variant"),
        }
    }

    #[test]
    fn test_deserialize_vault_config_username_password() {
        let json = r#"
        {
            "mode": {
                "username": "admin",
                "password": "changeme"
            }
        }
        "#;

        let config: VaultConnectionConfig = serde_json::from_str(json).unwrap();
        match config.mode {
            VaultConnectionModeConfig::UserPass(auth) => {
                assert_eq!(auth.username, "admin");
                assert_eq!(auth.password.deref(), "changeme");
            }
            _ => panic!("Expected UserPass variant"),
        }
    }

    #[test]
    fn test_deserialize_vault_config_username_password_missing_field() {
        let json = r#"
        {
            "mode": {
                "username": "admin"
            }
        }
        "#;

        let result = serde_json::from_str::<VaultConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_is_secret_type() {
        let config = VaultConnectionTokenAuthConfig {
            token: Secret::new("changeme".to_string()),
        };

        assert_eq!(config.token.deref(), "changeme");
    }

    #[test]
    fn test_password_is_secret_type() {
        let config = VaultConnectionUserPassConfig {
            username: "admin".to_string(),
            password: Secret::new("changeme".to_string()),
        };

        assert_eq!(config.password.deref(), "changeme");
    }
}
