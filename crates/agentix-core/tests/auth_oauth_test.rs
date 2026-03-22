//! Integration tests for OAuth provider flows and AuthService.
//!
//! Tests cover:
//! - Provider name normalization
//! - OpenAI OAuth URL generation (PKCE)
//! - Gemini OAuth URL generation (PKCE, env-based credentials)
//! - OpenAI redirect code parsing and state validation
//! - Anthropic auth kind detection
//! - JWT account ID extraction

use agentix_core::{
    auth::{
        generate_pkce_state,
        openai_oauth::{
            build_authorize_url as openai_build_authorize_url,
            extract_account_id_from_jwt,
            parse_code_from_redirect as openai_parse_code_from_redirect,
            OPENAI_OAUTH_CLIENT_ID,
        },
        gemini_oauth::{
            build_authorize_url as gemini_build_authorize_url,
            GOOGLE_OAUTH_AUTHORIZE_URL,
        },
    },
    auth::anthropic_token::{AnthropicAuthKind, detect_auth_kind},
    auth_service::normalize_provider,
};
use base64::Engine;

// ---------------------------------------------------------------------------
// Helper: isolate environment variable changes
// ---------------------------------------------------------------------------

struct EnvGuard {
    key: &'static str,
    original: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let original = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, original }
    }

    fn remove(key: &'static str) -> Self {
        let original = std::env::var(key).ok();
        std::env::remove_var(key);
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

// ---------------------------------------------------------------------------
// Provider normalization
// ---------------------------------------------------------------------------

#[test]
fn normalize_provider_aliases() {
    // OpenAI aliases
    assert_eq!(normalize_provider("openai").unwrap(), "openai");
    assert_eq!(normalize_provider("codex").unwrap(), "openai");
    assert_eq!(normalize_provider("openai-codex").unwrap(), "openai");
    assert_eq!(normalize_provider("OPENAI").unwrap(), "openai");

    // Anthropic aliases
    assert_eq!(normalize_provider("anthropic").unwrap(), "anthropic");
    assert_eq!(normalize_provider("claude").unwrap(), "anthropic");
    assert_eq!(normalize_provider("claude-code").unwrap(), "anthropic");

    // Gemini aliases
    assert_eq!(normalize_provider("gemini").unwrap(), "gemini");
    assert_eq!(normalize_provider("google").unwrap(), "gemini");
    assert_eq!(normalize_provider("vertex").unwrap(), "gemini");

    // Unknown provider passes through
    assert_eq!(normalize_provider("custom-llm").unwrap(), "custom-llm");

    // Empty provider fails
    assert!(normalize_provider("").is_err());
    assert!(normalize_provider("   ").is_err());
}

// ---------------------------------------------------------------------------
// OpenAI OAuth URL generation
// ---------------------------------------------------------------------------

#[test]
fn openai_authorize_url_contains_pkce() {
    let pkce = generate_pkce_state();
    let url = openai_build_authorize_url(&pkce);

    assert!(url.contains("auth.openai.com"));
    assert!(url.contains("client_id="));
    assert!(url.contains(OPENAI_OAUTH_CLIENT_ID));
    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("offline_access"));
}

#[test]
fn openai_authorize_url_contains_state_param() {
    let pkce = generate_pkce_state();
    let url = openai_build_authorize_url(&pkce);

    // State should be present in the URL
    assert!(url.contains("state="));
    // URL should contain the actual PKCE state value
    assert!(url.contains(pkce.state.as_str()));
}

#[test]
fn openai_authorize_url_contains_redirect_uri() {
    let pkce = generate_pkce_state();
    let url = openai_build_authorize_url(&pkce);

    assert!(url.contains("redirect_uri="));
    assert!(url.contains("1455"));
}

// ---------------------------------------------------------------------------
// Gemini OAuth URL generation
// ---------------------------------------------------------------------------

#[test]
fn gemini_authorize_url_contains_pkce() {
    let _client_id = EnvGuard::set("GEMINI_OAUTH_CLIENT_ID", "test-client-id-123");
    let _client_secret = EnvGuard::set("GEMINI_OAUTH_CLIENT_SECRET", "test-client-secret");

    let pkce = generate_pkce_state();
    let url = gemini_build_authorize_url(&pkce).expect("Failed to build Gemini authorize URL");

    assert!(url.contains(GOOGLE_OAUTH_AUTHORIZE_URL));
    assert!(url.contains("client_id="));
    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("access_type=offline"));
    assert!(url.contains("prompt=consent"));
    assert!(url.contains("redirect_uri="));
    assert!(url.contains("1456"));
}

#[test]
fn gemini_authorize_url_fails_without_env_vars() {
    let _client_id = EnvGuard::remove("GEMINI_OAUTH_CLIENT_ID");
    let _client_secret = EnvGuard::remove("GEMINI_OAUTH_CLIENT_SECRET");

    let pkce = generate_pkce_state();
    let result = gemini_build_authorize_url(&pkce);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("GEMINI_OAUTH_CLIENT_ID"));
}

// ---------------------------------------------------------------------------
// OpenAI redirect code parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_openai_redirect_extracts_code() {
    let code = openai_parse_code_from_redirect(
        "http://127.0.0.1:1455/auth/callback?code=abc123&state=xyz789",
        Some("xyz789"),
    )
    .unwrap();
    assert_eq!(code, "abc123");
}

#[test]
fn parse_openai_redirect_rejects_state_mismatch() {
    let result = openai_parse_code_from_redirect(
        "/auth/callback?code=abc123&state=wrong",
        Some("expected-state"),
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("state mismatch"));
}

#[test]
fn parse_openai_redirect_rejects_error() {
    let result = openai_parse_code_from_redirect(
        "/auth/callback?error=access_denied&error_description=user+cancelled",
        Some("xyz"),
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("access_denied"));
}

#[test]
fn parse_openai_redirect_accepts_raw_code() {
    // A raw code (no URL structure) should be returned as-is
    let code = openai_parse_code_from_redirect("my-raw-code-value", None).unwrap();
    assert_eq!(code, "my-raw-code-value");
}

// ---------------------------------------------------------------------------
// Anthropic auth kind detection
// ---------------------------------------------------------------------------

#[test]
fn anthropic_detect_jwt_as_bearer() {
    // JWT-like tokens (2+ dots) should be detected as Authorization
    let kind = detect_auth_kind("aaa.bbb.ccc", None);
    assert_eq!(kind, AnthropicAuthKind::Authorization);

    let kind = detect_auth_kind("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEyMyJ9.sig", None);
    assert_eq!(kind, AnthropicAuthKind::Authorization);
}

#[test]
fn anthropic_detect_api_key_prefix() {
    let kind = detect_auth_kind("sk-ant-api-xxxxx", None);
    assert_eq!(kind, AnthropicAuthKind::ApiKey);
}

#[test]
fn anthropic_detect_explicit_override() {
    // Explicit override takes precedence even over JWT-like format
    let kind = detect_auth_kind("aaa.bbb.ccc", Some("api-key"));
    assert_eq!(kind, AnthropicAuthKind::ApiKey);

    let kind = detect_auth_kind("sk-ant-api-xxx", Some("authorization"));
    assert_eq!(kind, AnthropicAuthKind::Authorization);
}

#[test]
fn anthropic_detect_default_is_api_key() {
    // Default for unknown tokens is ApiKey
    let kind = detect_auth_kind("some-random-token", None);
    assert_eq!(kind, AnthropicAuthKind::ApiKey);
}

#[test]
fn anthropic_auth_kind_metadata_roundtrip() {
    let api_key_str = AnthropicAuthKind::ApiKey.as_metadata_value();
    let auth_str = AnthropicAuthKind::Authorization.as_metadata_value();

    assert_eq!(
        AnthropicAuthKind::from_metadata_value(api_key_str),
        Some(AnthropicAuthKind::ApiKey)
    );
    assert_eq!(
        AnthropicAuthKind::from_metadata_value(auth_str),
        Some(AnthropicAuthKind::Authorization)
    );

    // Aliases
    assert_eq!(
        AnthropicAuthKind::from_metadata_value("x-api-key"),
        Some(AnthropicAuthKind::ApiKey)
    );
    assert_eq!(
        AnthropicAuthKind::from_metadata_value("bearer"),
        Some(AnthropicAuthKind::Authorization)
    );
    assert_eq!(AnthropicAuthKind::from_metadata_value("unknown"), None);
}

// ---------------------------------------------------------------------------
// OpenAI JWT account ID extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_openai_account_id_from_jwt() {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("{}");
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"account_id":"acct_abc123","sub":"user_xyz"}"#);
    let token = format!("{header}.{payload}.sig");

    let account_id = extract_account_id_from_jwt(&token);
    assert_eq!(account_id.as_deref(), Some("acct_abc123"));
}

#[test]
fn extract_openai_account_id_falls_back_to_sub() {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("{}");
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"sub":"user_xyz"}"#);
    let token = format!("{header}.{payload}.sig");

    let account_id = extract_account_id_from_jwt(&token);
    assert_eq!(account_id.as_deref(), Some("user_xyz"));
}

#[test]
fn extract_openai_account_id_returns_none_for_invalid_jwt() {
    assert!(extract_account_id_from_jwt("not-a-jwt").is_none());
    assert!(extract_account_id_from_jwt("only.two").is_none());
}
