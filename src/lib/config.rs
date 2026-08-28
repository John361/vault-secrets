use anyhow::Result;
use serde::Deserialize;

use crate::secret::Secret;
use crate::vault::{VaultConnectionConfig, VaultExportConfig, VaultFindConfig, VaultImportConfig};

#[derive(Deserialize)]
pub struct AppConfig {
    pub connection: VaultConnectionConfig,
    pub find: VaultFindConfig,
    pub export: VaultExportConfig,
    pub import: VaultImportConfig,
    pub request_interval_ms: u64,
    pub encryption_passphrase: Secret,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .unwrap_or_else(|e| panic!("Cannot get app config file path: {e:?}"));

        let configurations = config
            .try_deserialize::<AppConfig>()
            .unwrap_or_else(|e| panic!("Cannot deserialize app config: {e:?}"));

        Ok(configurations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::VaultConnectionModeConfig;
    use std::io::Write;
    use std::ops::Deref;
    use std::panic;
    use tempfile::NamedTempFile;

    fn write_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_load_success_login_password() {
        let config_content = r#"
        connection:
          address: "http://localhost:8200"
          mode:
              username: "admin"
              password: "changeme"
        find:
          encode: true
        export:
          mounts:
            - name: "secret-1"
              engine: "Kv1"
            - name: "secret-2"
              engine: "Kv2"
        import:
          mounts:
            - name: "secret-1"
              engine: "Kv1"
            - name: "secret-2"
              engine: "Kv2"
        request_interval_ms: 200
        encryption_passphrase: "changeme"
    "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path_with_extension = temp_file.path().with_extension("yml");
        std::fs::write(&path_with_extension, config_content).unwrap();

        let path = path_with_extension.to_str().unwrap();
        let result = AppConfig::load(path);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.connection.address, "http://localhost:8200");
        assert_eq!(config.request_interval_ms, 200);
        assert_eq!(config.encryption_passphrase.deref(), "changeme");

        match config.connection.mode {
            VaultConnectionModeConfig::UserPass(auth) => {
                assert_eq!(auth.username, "admin");
                assert_eq!(auth.password.deref(), "changeme");
            }
            _ => panic!("Expected UserPass variant"),
        }
    }

    #[test]
    fn test_load_success_token() {
        let config_content = r#"
        connection:
          address: "http://localhost:8200"
          mode:
            token: "changeme"
        find:
          encode: true
        export:
          mounts:
            - name: "secret-1"
              engine: "Kv1"
            - name: "secret-2"
              engine: "Kv2"
        import:
          mounts:
            - name: "secret-1"
              engine: "Kv1"
            - name: "secret-2"
              engine: "Kv2"
        request_interval_ms: 200
        encryption_passphrase: "changeme"
    "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path_with_extension = temp_file.path().with_extension("yml");
        std::fs::write(&path_with_extension, config_content).unwrap();

        let path = path_with_extension.to_str().unwrap();
        let result = AppConfig::load(path);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.connection.address, "http://localhost:8200");
        assert_eq!(config.request_interval_ms, 200);
        assert_eq!(config.encryption_passphrase.deref(), "changeme");

        match config.connection.mode {
            VaultConnectionModeConfig::Token(auth) => {
                assert_eq!(auth.token.deref(), "changeme");
            }
            _ => panic!("Expected Token variant"),
        }
    }

    #[test]
    fn test_load_file_not_found_panics() {
        let path = "/tmp/this_file_does_not_exist_12345.yml";
        let result = panic::catch_unwind(|| AppConfig::load(path));

        assert!(result.is_err());
    }

    #[test]
    fn test_load_deserialize_error_panics() {
        let config_content = r#"
        connection:
          mode:
            username: "admin"
            password: "changeme"
    "#;
        let temp_file = write_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();
        let result = panic::catch_unwind(|| AppConfig::load(path));

        assert!(result.is_err());
    }
}
