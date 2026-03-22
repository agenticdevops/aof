//! Anthropic credential detection and auth kind (ported from zeroclaw).
//!
//! Anthropic supports two credential modes:
//! - `ApiKey` — standard `x-api-key` header (sk-ant-api* prefix)
//! - `Authorization` — Bearer token for subscription/setup tokens (JWT-like shape)
//!
//! Detection logic: if the token has 2+ dots, it's JWT-like → Authorization mode.
//! The `sk-ant-api` prefix → ApiKey. Default → ApiKey.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AnthropicAuthKind
// ---------------------------------------------------------------------------

/// How Anthropic credentials should be sent to the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnthropicAuthKind {
    /// Standard Anthropic API key via `x-api-key` header.
    ApiKey,
    /// Subscription / setup token via `Authorization: Bearer ...` header.
    Authorization,
}

impl AnthropicAuthKind {
    /// Returns the metadata key string for this auth kind.
    pub fn as_metadata_value(self) -> &'static str {
        match self {
            Self::ApiKey => "api-key",
            Self::Authorization => "authorization",
        }
    }

    /// Parse an auth kind from a metadata string.
    ///
    /// Accepts: `"api-key"`, `"x-api-key"`, `"apikey"` → `ApiKey`
    /// Accepts: `"authorization"`, `"bearer"`, `"auth-token"`, `"oauth"` → `Authorization`
    pub fn from_metadata_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "api-key" | "x-api-key" | "apikey" => Some(Self::ApiKey),
            "authorization" | "bearer" | "auth-token" | "oauth" => Some(Self::Authorization),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Detect the Anthropic auth kind for a given token, with optional explicit override.
///
/// Priority:
/// 1. `explicit` override → if parseable, use that
/// 2. Token has 2+ dots (JWT-like) → `Authorization`
/// 3. Token starts with `sk-ant-api` → `ApiKey`
/// 4. Default → `ApiKey`
pub fn detect_auth_kind(token: &str, explicit: Option<&str>) -> AnthropicAuthKind {
    if let Some(kind) = explicit.and_then(AnthropicAuthKind::from_metadata_value) {
        return kind;
    }

    let trimmed = token.trim();

    // JWT-like shape (2+ dots) strongly suggests Bearer token mode.
    if trimmed.matches('.').count() >= 2 {
        return AnthropicAuthKind::Authorization;
    }

    // Anthropic platform keys commonly start with this prefix.
    if trimmed.starts_with("sk-ant-api") {
        return AnthropicAuthKind::ApiKey;
    }

    // Default to API key for backward compatibility.
    AnthropicAuthKind::ApiKey
}
