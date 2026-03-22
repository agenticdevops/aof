//! Tests for `agentix auth` CLI command logic.
//!
//! These tests exercise the auth command logic functions directly using a temp
//! directory (not the full CLI binary), avoiding actual OAuth server calls.

use agentix_core::{AuthService, normalize_provider};
use std::collections::HashMap;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_service(dir: &TempDir) -> AuthService {
    AuthService::new(dir.path(), false)
}

// ---------------------------------------------------------------------------
// Provider name normalization
// ---------------------------------------------------------------------------

#[test]
fn provider_name_normalization_openai() {
    assert_eq!(normalize_provider("openai").unwrap(), "openai");
    assert_eq!(normalize_provider("codex").unwrap(), "openai");
    assert_eq!(normalize_provider("openai-codex").unwrap(), "openai");
    assert_eq!(normalize_provider("OPENAI").unwrap(), "openai");
}

#[test]
fn provider_name_normalization_anthropic() {
    assert_eq!(normalize_provider("anthropic").unwrap(), "anthropic");
    assert_eq!(normalize_provider("claude").unwrap(), "anthropic");
    assert_eq!(normalize_provider("claude-code").unwrap(), "anthropic");
    assert_eq!(normalize_provider("Anthropic").unwrap(), "anthropic");
}

#[test]
fn provider_name_normalization_gemini() {
    assert_eq!(normalize_provider("gemini").unwrap(), "gemini");
    assert_eq!(normalize_provider("google").unwrap(), "gemini");
    assert_eq!(normalize_provider("vertex").unwrap(), "gemini");
    assert_eq!(normalize_provider("Google").unwrap(), "gemini");
}

#[test]
fn provider_name_normalization_empty_errors() {
    assert!(normalize_provider("").is_err());
    assert!(normalize_provider("   ").is_err());
}

// ---------------------------------------------------------------------------
// Status: empty store
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_status_shows_no_providers_when_empty() {
    let dir = TempDir::new().unwrap();
    let service = make_service(&dir);

    let profiles = service.load_profiles().await.unwrap();
    assert!(
        profiles.profiles.is_empty(),
        "Expected empty profiles but got: {:?}",
        profiles.profiles
    );
}

// ---------------------------------------------------------------------------
// Store + retrieve profile
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_status_shows_connected_after_store() {
    let dir = TempDir::new().unwrap();
    let service = make_service(&dir);

    // Store an Anthropic token profile
    service
        .store_provider_token("anthropic", "default", "sk-ant-api-test-key", HashMap::new(), true)
        .await
        .unwrap();

    let profile = service.get_profile("anthropic", None).await.unwrap();
    assert!(profile.is_some(), "Expected anthropic profile to be present");

    let p = profile.unwrap();
    assert_eq!(p.provider, "anthropic");
    assert_eq!(p.profile_name, "default");
}

// ---------------------------------------------------------------------------
// Disconnect: nonexistent provider
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_disconnect_nonexistent_provider_returns_false() {
    let dir = TempDir::new().unwrap();
    let service = make_service(&dir);

    // Disconnect a provider that was never connected
    let removed = service.remove_profile("openai", "default").await.unwrap();
    assert!(
        !removed,
        "Expected remove_profile to return false for nonexistent profile"
    );
}

// ---------------------------------------------------------------------------
// Disconnect: existing profile
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_disconnect_existing_profile_returns_true() {
    let dir = TempDir::new().unwrap();
    let service = make_service(&dir);

    // Store a profile first
    service
        .store_provider_token("openai", "default", "dummy-oauth-token", HashMap::new(), true)
        .await
        .unwrap();

    // Verify it exists
    let profile = service.get_profile("openai", None).await.unwrap();
    assert!(profile.is_some());

    // Disconnect
    let removed = service.remove_profile("openai", "default").await.unwrap();
    assert!(removed, "Expected remove_profile to return true for existing profile");

    // Verify it's gone
    let profile_after = service.get_profile("openai", None).await.unwrap();
    assert!(profile_after.is_none(), "Expected profile to be removed");
}

// ---------------------------------------------------------------------------
// Multiple providers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_status_shows_multiple_providers_independently() {
    let dir = TempDir::new().unwrap();
    let service = make_service(&dir);

    service
        .store_provider_token("anthropic", "default", "sk-ant-api-key", HashMap::new(), true)
        .await
        .unwrap();

    // Anthropic connected, openai not
    let anthropic = service.get_profile("anthropic", None).await.unwrap();
    let openai = service.get_profile("openai", None).await.unwrap();
    let gemini = service.get_profile("gemini", None).await.unwrap();

    assert!(anthropic.is_some(), "Anthropic should be connected");
    assert!(openai.is_none(), "OpenAI should not be connected");
    assert!(gemini.is_none(), "Gemini should not be connected");
}
