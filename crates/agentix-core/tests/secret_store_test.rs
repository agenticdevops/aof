use agentix_core::{SecretStore, EncryptedSecret, SecretRedactor};

#[test]
fn test_encrypt_decrypt_round_trip() {
    let store = SecretStore::new("my-secret-passphrase");
    let plaintext = "sk-abc123-my-api-key";
    let encrypted = store.encrypt(plaintext).unwrap();
    let decrypted = store.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_unique_nonce_per_encryption() {
    let store = SecretStore::new("my-secret-passphrase");
    let plaintext = "same-secret-twice";
    let enc1 = store.encrypt(plaintext).unwrap();
    let enc2 = store.encrypt(plaintext).unwrap();
    // Nonces must differ
    assert_ne!(enc1.nonce, enc2.nonce);
    // Both must decrypt to the same plaintext
    assert_eq!(store.decrypt(&enc1).unwrap(), plaintext);
    assert_eq!(store.decrypt(&enc2).unwrap(), plaintext);
}

#[test]
fn test_wrong_key_fails() {
    let store_correct = SecretStore::new("correct-key");
    let encrypted = store_correct.encrypt("my-secret").unwrap();
    let store_wrong = SecretStore::new("wrong-key");
    let result = store_wrong.decrypt(&encrypted);
    assert!(result.is_err());
}

#[test]
fn test_encrypted_secret_json_round_trip() {
    let store = SecretStore::new("test-passphrase");
    let plaintext = "sk-json-round-trip";
    let encrypted = store.encrypt(plaintext).unwrap();
    let json = serde_json::to_string(&encrypted).unwrap();
    let deserialized: EncryptedSecret = serde_json::from_str(&json).unwrap();
    let decrypted = store.decrypt(&deserialized).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypted_secret_json_uses_base64() {
    let store = SecretStore::new("test-passphrase");
    let encrypted = store.encrypt("base64-test-secret").unwrap();
    let json = serde_json::to_string(&encrypted).unwrap();
    // JSON should not contain raw binary bytes
    // Base64 chars are alphanumeric, +, /, =
    assert!(!json.contains('\0'), "JSON should not contain null bytes");
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // ciphertext and nonce should be string fields (base64-encoded)
    assert!(parsed["ciphertext"].is_string(), "ciphertext should be a base64 string");
    assert!(parsed["nonce"].is_string(), "nonce should be a base64 string");
}

#[test]
fn test_secret_redactor_basic() {
    let redactor = SecretRedactor::new(vec![
        ("API_KEY".to_string(), "sk-abc123".to_string()),
    ]);
    let result = redactor.redact("My key is sk-abc123 and it works");
    assert_eq!(result, "My key is [REDACTED:API_KEY] and it works");
}

#[test]
fn test_secret_redactor_multiple_secrets() {
    let redactor = SecretRedactor::new(vec![
        ("DB_PASS".to_string(), "p@ssw0rd!".to_string()),
        ("TOKEN".to_string(), "ghp_1234".to_string()),
    ]);
    let result = redactor.redact("DB: p@ssw0rd! Token: ghp_1234");
    assert!(result.contains("[REDACTED:DB_PASS]"), "Should redact DB_PASS: {}", result);
    assert!(result.contains("[REDACTED:TOKEN]"), "Should redact TOKEN: {}", result);
    assert!(!result.contains("p@ssw0rd!"), "Should not contain raw DB_PASS");
    assert!(!result.contains("ghp_1234"), "Should not contain raw TOKEN");
}

#[test]
fn test_secret_redactor_json() {
    let redactor = SecretRedactor::new(vec![
        ("API_KEY".to_string(), "sk-abc123".to_string()),
    ]);
    let json_val = serde_json::json!({
        "url": "https://api.com",
        "auth": "Bearer sk-abc123",
        "nested": {
            "key": "sk-abc123"
        }
    });
    let redacted = redactor.redact_json(&json_val);
    let auth = redacted["auth"].as_str().unwrap();
    assert!(auth.contains("[REDACTED:API_KEY]"), "auth should be redacted: {}", auth);
    let nested_key = redacted["nested"]["key"].as_str().unwrap();
    assert!(nested_key.contains("[REDACTED:API_KEY]"), "nested key should be redacted: {}", nested_key);
}

#[test]
fn test_secret_redactor_no_false_positives() {
    let redactor = SecretRedactor::new(vec![
        ("SECRET".to_string(), "hidden-value".to_string()),
    ]);
    let input = "This text has no secrets at all";
    let result = redactor.redact(input);
    assert_eq!(result, input);
}

#[test]
fn test_empty_secret_store() {
    let store = SecretStore::new("");
    let encrypted = store.encrypt("test-data").unwrap();
    let decrypted = store.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, "test-data");
}
