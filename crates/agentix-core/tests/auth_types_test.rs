//! Tests for auth types, profile store, PKCE, and token expiry.

use agentix_core::auth::{
    generate_pkce_state, parse_query_params, profile_id, select_profile_id, url_decode, url_encode,
    AuthProfile, AuthProfileKind, AuthProfilesData, AuthProfilesStore, TokenSet,
};
use base64::Engine as _;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// profile_id
// ---------------------------------------------------------------------------

#[test]
fn profile_id_format() {
    assert_eq!(profile_id("openai", "default"), "openai:default");
    assert_eq!(profile_id("anthropic", "work"), "anthropic:work");
    assert_eq!(profile_id("gemini", "personal"), "gemini:personal");
}

// ---------------------------------------------------------------------------
// TokenSet expiry
// ---------------------------------------------------------------------------

#[test]
fn token_expiry_within() {
    let token_set = TokenSet {
        access_token: "token".into(),
        refresh_token: Some("refresh".into()),
        id_token: None,
        expires_at: Some(Utc::now() + chrono::Duration::seconds(10)),
        token_type: Some("Bearer".into()),
        scope: None,
    };
    assert!(
        token_set.is_expiring_within(Duration::from_secs(15)),
        "Should be expiring within 15s"
    );
    assert!(
        !token_set.is_expiring_within(Duration::from_secs(1)),
        "Should NOT be expiring within 1s"
    );
}

#[test]
fn token_not_expiring_when_no_deadline() {
    let token_set = TokenSet {
        access_token: "token".into(),
        refresh_token: None,
        id_token: None,
        expires_at: None,
        token_type: None,
        scope: None,
    };
    assert!(
        !token_set.is_expiring_within(Duration::from_secs(9999)),
        "No expiry means never expiring"
    );
}

// ---------------------------------------------------------------------------
// PKCE generation
// ---------------------------------------------------------------------------

#[test]
fn pkce_generation_is_valid() {
    let pkce = generate_pkce_state();
    // base64url of 64 bytes = 86 chars (no padding)
    assert!(
        pkce.code_verifier.len() >= 43,
        "code_verifier must be at least 43 chars, got {}",
        pkce.code_verifier.len()
    );
    assert!(
        !pkce.code_challenge.is_empty(),
        "code_challenge must not be empty"
    );
    assert!(!pkce.state.is_empty(), "state must not be empty");
}

#[test]
fn pkce_challenge_is_sha256_of_verifier() {
    let pkce = generate_pkce_state();
    let expected = {
        let digest = Sha256::digest(pkce.code_verifier.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    };
    assert_eq!(pkce.code_challenge, expected, "S256 challenge must match");
}

// ---------------------------------------------------------------------------
// URL encode / decode
// ---------------------------------------------------------------------------

#[test]
fn url_encode_decode_roundtrip() {
    let original = "hello world! @#$%^&*()";
    let encoded = url_encode(original);
    let decoded = url_decode(&encoded);
    assert_eq!(decoded, original);
}

// ---------------------------------------------------------------------------
// parse_query_params
// ---------------------------------------------------------------------------

#[test]
fn parse_query_params_basic() {
    let params = parse_query_params("code=abc123&state=xyz");
    assert_eq!(params.get("code"), Some(&"abc123".to_string()));
    assert_eq!(params.get("state"), Some(&"xyz".to_string()));
}

// ---------------------------------------------------------------------------
// select_profile_id — priority chain
// ---------------------------------------------------------------------------

#[test]
fn select_profile_prefers_override_then_active_then_default() {
    let mut data = AuthProfilesData::default();

    let id_default = profile_id("openai", "default");
    let id_work = profile_id("openai", "work");

    let make_profile = |id: &str, provider: &str, name: &str| AuthProfile {
        id: id.to_string(),
        provider: provider.to_string(),
        profile_name: name.to_string(),
        kind: AuthProfileKind::Token,
        account_id: None,
        workspace_id: None,
        token_set: None,
        token: Some("tok".into()),
        metadata: BTreeMap::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    data.profiles
        .insert(id_default.clone(), make_profile(&id_default, "openai", "default"));
    data.profiles
        .insert(id_work.clone(), make_profile(&id_work, "openai", "work"));
    data.active_profiles
        .insert("openai".into(), id_work.clone());

    // Explicit override selects "default" even though "work" is active
    assert_eq!(
        select_profile_id(&data, "openai", Some("default")),
        Some(id_default.clone()),
        "explicit override must win"
    );

    // No override → active profile
    assert_eq!(
        select_profile_id(&data, "openai", None),
        Some(id_work.clone()),
        "active profile must win when no override"
    );

    // Remove active → falls back to default
    data.active_profiles.remove("openai");
    assert_eq!(
        select_profile_id(&data, "openai", None),
        Some(id_default.clone()),
        "default profile must be selected when no active"
    );

    // Remove default → falls back to first profile
    data.profiles.remove(&id_default);
    assert_eq!(
        select_profile_id(&data, "openai", None),
        Some(id_work.clone()),
        "first profile must be selected when no active or default"
    );
}

// ---------------------------------------------------------------------------
// AuthProfilesStore — roundtrip with encryption
// ---------------------------------------------------------------------------

#[tokio::test]
async fn store_roundtrip_with_encryption() {
    let tmp = TempDir::new().unwrap();
    let store = AuthProfilesStore::new(tmp.path(), true);

    let mut profile = AuthProfile::new_oauth(
        "openai",
        "default",
        TokenSet {
            access_token: "access-123".into(),
            refresh_token: Some("refresh-123".into()),
            id_token: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_type: Some("Bearer".into()),
            scope: Some("openid offline_access".into()),
        },
    );
    profile.account_id = Some("acct_abc".into());

    store.upsert_profile(profile.clone(), true).await.unwrap();

    // Reload and verify decryption
    let data = store.load().await.unwrap();
    let loaded = data.profiles.get(&profile.id).unwrap();

    assert_eq!(loaded.provider, "openai");
    assert_eq!(loaded.profile_name, "default");
    assert_eq!(loaded.account_id.as_deref(), Some("acct_abc"));
    assert_eq!(
        loaded
            .token_set
            .as_ref()
            .map(|t| t.access_token.as_str()),
        Some("access-123")
    );
    assert_eq!(
        loaded
            .token_set
            .as_ref()
            .and_then(|t| t.refresh_token.as_deref()),
        Some("refresh-123")
    );

    // The raw JSON on disk must NOT contain plaintext tokens
    let raw = tokio::fs::read_to_string(store.path()).await.unwrap();
    assert!(raw.contains("enc2:"), "raw file must use enc2: prefix");
    assert!(!raw.contains("access-123"), "plaintext must not appear in file");
    assert!(!raw.contains("refresh-123"), "plaintext must not appear in file");
}

// ---------------------------------------------------------------------------
// AuthProfilesStore — atomic write creates the file
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_write_replaces_file() {
    let tmp = TempDir::new().unwrap();
    let store = AuthProfilesStore::new(tmp.path(), false);

    let profile = AuthProfile::new_token("anthropic", "default", "token-abc".into());
    store.upsert_profile(profile, true).await.unwrap();

    let path = store.path().to_path_buf();
    assert!(path.exists(), "profile file must exist after upsert");

    let contents = tokio::fs::read_to_string(path).await.unwrap();
    assert!(
        contents.contains("\"schema_version\": 1"),
        "file must contain schema_version"
    );
}
