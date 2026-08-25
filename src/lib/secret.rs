use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Secret(Zeroizing<String>);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "*************")
    }
}

impl Deref for Secret {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_secret() {
        let secret = Secret::new("my_secret_value".to_string());
        assert_eq!(&*secret, "my_secret_value");
    }

    #[test]
    fn test_debug_masks_value() {
        let secret = Secret::new("super_secret".to_string());

        assert_eq!(format!("{:?}", secret), "*************");
        assert_eq!(
            format!("Secret is: {:?}", secret),
            "Secret is: *************"
        );
    }

    #[test]
    fn test_deref_allows_string_operations() {
        let secret = Secret::new("hello_world".to_string());

        assert_eq!(secret.len(), 11);
        assert!(secret.contains("world"));
        assert_eq!(&secret[0..5], "hello");
    }

    #[test]
    fn test_from_string() {
        let secret = Secret::from("converted_string".to_string());
        assert_eq!(&*secret, "converted_string");
    }

    #[test]
    fn test_deserialize_from_string() {
        let json = r#""my_deserialized_secret""#;
        let secret: Secret = serde_json::from_str(json).unwrap();

        assert_eq!(&*secret, "my_deserialized_secret");
    }

    #[test]
    fn test_serialize_from_secret() {
        let secret = Secret::new("my_secret_value".to_string());
        let json = serde_json::to_value(&secret).unwrap();

        assert_eq!(&json, "my_secret_value");
    }
}
