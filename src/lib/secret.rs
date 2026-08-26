use std::fmt;
use std::ops::Deref;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::Result;
use pbkdf2::pbkdf2_hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
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

#[derive(Serialize, Deserialize)]
pub struct EncryptedSecret {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 32],
}

impl EncryptedSecret {
    fn derive_key(password: &str, salt: &[u8; 32]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
        key
    }

    pub fn encrypt(secret: &str, passphrase: &str) -> Result<EncryptedSecret> {
        let mut salt = [0u8; 32];
        rand::fill(&mut salt);

        let mut nonce = [0u8; 12];
        rand::fill(&mut nonce);

        let key = Self::derive_key(passphrase, &salt);
        let key = Key::<Aes256Gcm>::try_from(key)?;
        let nonce = Nonce::try_from(nonce)?;
        let cipher = Aes256Gcm::new(&key);
        let ciphertext = cipher.encrypt(&nonce, secret.as_bytes())?;

        Ok(EncryptedSecret {
            ciphertext,
            nonce: nonce.into(),
            salt,
        })
    }

    pub fn decrypt(encrypted: &EncryptedSecret, passphrase: &str) -> Result<String> {
        let key = Self::derive_key(passphrase, &encrypted.salt);
        let key = Key::<Aes256Gcm>::try_from(key)?;
        let nonce = Nonce::try_from(encrypted.nonce)?;
        let cipher = Aes256Gcm::new(&key);
        let plaintext = cipher.decrypt(&nonce, encrypted.ciphertext.as_ref())?;

        Ok(String::from_utf8(plaintext)?)
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

    #[test]
    fn test_encryption_and_decryption() {
        let secret = "mon_secret_vault";
        let password = "mon_mot_de_passe_123";

        let encrypted = EncryptedSecret::encrypt(secret, password).unwrap();
        let decrypted = EncryptedSecret::decrypt(&encrypted, password).unwrap();

        assert_eq!(secret, decrypted);
    }
}
