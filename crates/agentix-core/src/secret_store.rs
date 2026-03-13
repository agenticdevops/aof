//! Secret encryption and redaction for OpenAgentiX.
//!
//! Provides AES-256-GCM encryption for secrets at rest and a redactor that prevents
//! secret values from appearing in logs, traces, or error output.

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

// ---------------------------------------------------------------------------
// SecretError
// ---------------------------------------------------------------------------

/// Error type for secret encryption/decryption operations.
#[derive(Debug, Clone)]
pub enum SecretError {
    /// Encryption operation failed.
    EncryptionFailed(String),
    /// Decryption operation failed (wrong key or corrupted data).
    DecryptionFailed(String),
    /// Invalid encryption key.
    InvalidKey(String),
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretError::EncryptionFailed(msg) => write!(f, "encryption failed: {}", msg),
            SecretError::DecryptionFailed(msg) => write!(f, "decryption failed: {}", msg),
            SecretError::InvalidKey(msg) => write!(f, "invalid key: {}", msg),
        }
    }
}

impl std::error::Error for SecretError {}

// ---------------------------------------------------------------------------
// EncryptedSecret
// ---------------------------------------------------------------------------

/// An encrypted secret value with its nonce and creation timestamp.
///
/// Ciphertext and nonce are serialized as base64 strings in JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecret {
    /// AES-256-GCM ciphertext (base64-encoded in JSON).
    #[serde(with = "base64_serde")]
    pub ciphertext: Vec<u8>,
    /// 12-byte nonce used for this encryption (base64-encoded in JSON).
    #[serde(with = "base64_serde")]
    pub nonce: Vec<u8>,
    /// When this secret was encrypted.
    pub created_at: DateTime<Utc>,
}

/// Serde module for serializing Vec<u8> as base64 strings.
mod base64_serde {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = BASE64.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BASE64.decode(&s).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// SecretStore
// ---------------------------------------------------------------------------

/// AES-256-GCM secret store for encrypting and decrypting secret values.
///
/// The encryption key is derived from a passphrase using SHA-256.
pub struct SecretStore {
    /// 256-bit AES key derived from passphrase via SHA-256.
    key: [u8; 32],
}

impl SecretStore {
    /// Create a new SecretStore with a key derived from the given passphrase.
    ///
    /// The passphrase is hashed with SHA-256 to produce a 256-bit key.
    pub fn new(passphrase: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        Self { key }
    }

    /// Encrypt a plaintext string, returning an EncryptedSecret.
    ///
    /// Each call generates a unique 12-byte random nonce, so encrypting
    /// the same plaintext twice produces different ciphertexts.
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedSecret, SecretError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| SecretError::InvalidKey(e.to_string()))?;

        // Generate a random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| SecretError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedSecret {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            created_at: Utc::now(),
        })
    }

    /// Decrypt an EncryptedSecret back to the original plaintext.
    ///
    /// Returns an error if the key is wrong or the data is corrupted.
    pub fn decrypt(&self, encrypted: &EncryptedSecret) -> Result<String, SecretError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| SecretError::InvalidKey(e.to_string()))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext_bytes = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| SecretError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| SecretError::DecryptionFailed(format!("invalid UTF-8: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// SecretRedactor
// ---------------------------------------------------------------------------

/// Scans text for known secret values and replaces them with `[REDACTED:<name>]`.
///
/// Processes secrets in order of longest value first to avoid partial matches.
pub struct SecretRedactor {
    /// Known secret (name, value) pairs, sorted by value length descending.
    secrets: Vec<(String, String)>,
}

impl std::fmt::Debug for SecretRedactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretRedactor")
            .field("secrets_count", &self.secrets.len())
            .finish()
    }
}

impl SecretRedactor {
    /// Create a new SecretRedactor with the given secret name-value pairs.
    pub fn new(mut secrets: Vec<(String, String)>) -> Self {
        // Sort by value length descending to avoid partial matches
        secrets.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        Self { secrets }
    }

    /// Replace all occurrences of known secret values in the text.
    pub fn redact(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (name, value) in &self.secrets {
            if !value.is_empty() {
                result = result.replace(value.as_str(), &format!("[REDACTED:{}]", name));
            }
        }
        result
    }

    /// Recursively redact secret values in a JSON tree.
    ///
    /// String values are passed through `redact()`. Objects and arrays are recursed.
    /// Other types (numbers, booleans, null) are returned unchanged.
    pub fn redact_json(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => {
                serde_json::Value::String(self.redact(s))
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.redact_json(v));
                }
                serde_json::Value::Object(new_map)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.redact_json(v)).collect())
            }
            other => other.clone(),
        }
    }
}
