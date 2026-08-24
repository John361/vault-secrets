use anyhow::Result;
use serde::Deserialize;

use crate::vault::VaultConfig;

#[derive(Deserialize)]
pub struct AppConfig {
    pub vault: VaultConfig,
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
    fn test_load_success() {
        let config_content = r#"
        vault:
          address: "http://localhost:8200"
          username: "user"
          password: "pass"
          mount: "secret"
    "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path_with_extension = temp_file.path().with_extension("yml");
        std::fs::write(&path_with_extension, config_content).unwrap();

        let path = path_with_extension.to_str().unwrap();
        let result = AppConfig::load(path);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.vault.address, "http://localhost:8200");
        assert_eq!(config.vault.username, "user");
        assert_eq!(config.vault.password.deref(), "pass");
        assert_eq!(config.vault.mount, "secret");
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
        vault:
          username: "user"
          password: "pass"
          mount: "secret"
    "#;
        let temp_file = write_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();
        let result = panic::catch_unwind(|| AppConfig::load(path));

        assert!(result.is_err());
    }
}
