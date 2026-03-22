//! Google/Gemini OAuth2 authentication flow (ported from zeroclaw).
//!
//! Supports:
//! - Authorization code flow with PKCE (loopback redirect on port 1456)
//! - Device code flow for headless environments
//!
//! Client credentials come from environment variables:
//! - `GEMINI_OAUTH_CLIENT_ID`
//! - `GEMINI_OAUTH_CLIENT_SECRET`

use crate::auth::{parse_query_params, url_encode, PkceState, TokenSet};
use anyhow::{Context, Result};
use base64::Engine;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const GOOGLE_OAUTH_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_OAUTH_DEVICE_CODE_URL: &str = "https://oauth2.googleapis.com/device/code";
pub const GEMINI_OAUTH_REDIRECT_URI: &str = "http://localhost:1456/auth/callback";

/// Scopes required for Gemini API access.
pub const GEMINI_OAUTH_SCOPES: &str =
    "openid profile email https://www.googleapis.com/auth/cloud-platform";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result from starting a device code flow.
#[derive(Debug, Clone)]
pub struct DeviceCodeStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_url: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

// ---------------------------------------------------------------------------
// Credential helpers
// ---------------------------------------------------------------------------

/// Get Gemini OAuth client ID from environment.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID` to be set.
pub fn gemini_oauth_client_id() -> Option<String> {
    std::env::var("GEMINI_OAUTH_CLIENT_ID")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Get Gemini OAuth client secret from environment.
///
/// Requires `GEMINI_OAUTH_CLIENT_SECRET` to be set.
pub fn gemini_oauth_client_secret() -> Option<String> {
    std::env::var("GEMINI_OAUTH_CLIENT_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
}

fn get_oauth_credentials() -> Result<(String, String)> {
    let client_id = gemini_oauth_client_id().ok_or_else(|| {
        anyhow::anyhow!("GEMINI_OAUTH_CLIENT_ID environment variable is required")
    })?;
    let client_secret = gemini_oauth_client_secret().ok_or_else(|| {
        anyhow::anyhow!("GEMINI_OAUTH_CLIENT_SECRET environment variable is required")
    })?;
    Ok((client_id, client_secret))
}

// ---------------------------------------------------------------------------
// OAuth flow functions
// ---------------------------------------------------------------------------

/// Build the Google OAuth2 authorization URL with PKCE.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID` environment variable.
pub fn build_authorize_url(pkce: &PkceState) -> Result<String> {
    let (client_id, _) = get_oauth_credentials()?;
    let mut params = BTreeMap::new();
    params.insert("response_type", "code");
    params.insert("client_id", client_id.as_str());
    params.insert("redirect_uri", GEMINI_OAUTH_REDIRECT_URI);
    params.insert("scope", GEMINI_OAUTH_SCOPES);
    params.insert("code_challenge", pkce.code_challenge.as_str());
    params.insert("code_challenge_method", "S256");
    params.insert("state", pkce.state.as_str());
    params.insert("access_type", "offline");
    params.insert("prompt", "consent");

    let mut encoded: Vec<String> = Vec::with_capacity(params.len());
    for (k, v) in params {
        encoded.push(format!("{}={}", url_encode(k), url_encode(v)));
    }

    Ok(format!(
        "{}?{}",
        GOOGLE_OAUTH_AUTHORIZE_URL,
        encoded.join("&")
    ))
}

/// Exchange an authorization code for tokens.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID` and `GEMINI_OAUTH_CLIENT_SECRET`.
pub async fn exchange_code_for_tokens(
    client: &Client,
    code: &str,
    pkce: &PkceState,
) -> Result<TokenSet> {
    let (client_id, client_secret) = get_oauth_credentials()?;
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", GEMINI_OAUTH_REDIRECT_URI),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code_verifier", &pkce.code_verifier),
    ];

    let response = client
        .post(GOOGLE_OAUTH_TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("Failed to send token exchange request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read token response body")?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            anyhow::bail!(
                "Google OAuth error: {} - {}",
                err.error,
                err.error_description.unwrap_or_default()
            );
        }
        anyhow::bail!("Google OAuth token exchange failed ({}): {}", status, body);
    }

    let token_response: TokenResponse =
        serde_json::from_str(&body).context("Failed to parse token response")?;

    let expires_at = token_response
        .expires_in
        .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    Ok(TokenSet {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        id_token: token_response.id_token,
        expires_at,
        token_type: token_response.token_type.or_else(|| Some("Bearer".into())),
        scope: token_response.scope,
    })
}

/// Refresh a Gemini access token using a refresh token.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID` and `GEMINI_OAUTH_CLIENT_SECRET`.
pub async fn refresh_access_token(client: &Client, refresh_token: &str) -> Result<TokenSet> {
    let (client_id, client_secret) = get_oauth_credentials()?;
    let form = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
    ];

    let response = client
        .post(GOOGLE_OAUTH_TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("Failed to send refresh token request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read refresh response body")?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            anyhow::bail!(
                "Google OAuth refresh error: {} - {}",
                err.error,
                err.error_description.unwrap_or_default()
            );
        }
        anyhow::bail!("Google OAuth refresh failed ({}): {}", status, body);
    }

    let token_response: TokenResponse =
        serde_json::from_str(&body).context("Failed to parse refresh response")?;

    let expires_at = token_response
        .expires_in
        .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    Ok(TokenSet {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        id_token: token_response.id_token,
        expires_at,
        token_type: token_response.token_type.or_else(|| Some("Bearer".into())),
        scope: token_response.scope,
    })
}

/// Start a device code flow for headless environments.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID`.
pub async fn start_device_code_flow(client: &Client) -> Result<DeviceCodeStart> {
    let (client_id, _) = get_oauth_credentials()?;
    let form = [
        ("client_id", client_id.as_str()),
        ("scope", GEMINI_OAUTH_SCOPES),
    ];

    let response = client
        .post(GOOGLE_OAUTH_DEVICE_CODE_URL)
        .form(&form)
        .send()
        .await
        .context("Failed to start device code flow")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read device code response")?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            anyhow::bail!(
                "Google device code error: {} - {}",
                err.error,
                err.error_description.unwrap_or_default()
            );
        }
        anyhow::bail!("Google device code request failed ({}): {}", status, body);
    }

    let device_response: DeviceCodeResponse =
        serde_json::from_str(&body).context("Failed to parse device code response")?;

    let user_code = device_response.user_code;
    let verification_url = device_response.verification_url;

    Ok(DeviceCodeStart {
        device_code: device_response.device_code,
        verification_uri_complete: Some(format!("{}?user_code={}", &verification_url, &user_code)),
        user_code,
        verification_uri: verification_url,
        expires_in: device_response.expires_in.unwrap_or(1800),
        interval: device_response.interval.unwrap_or(5),
    })
}

/// Poll the device code token endpoint until authorization completes or times out.
///
/// Handles: `authorization_pending`, `slow_down`, `access_denied`, `expired_token`.
///
/// Requires `GEMINI_OAUTH_CLIENT_ID` and `GEMINI_OAUTH_CLIENT_SECRET`.
pub async fn poll_device_code_tokens(
    client: &Client,
    device: &DeviceCodeStart,
) -> Result<TokenSet> {
    let (client_id, client_secret) = get_oauth_credentials()?;
    let deadline = std::time::Instant::now() + Duration::from_secs(device.expires_in);
    let interval = Duration::from_secs(device.interval.max(5));

    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Device code expired before authorization was completed");
        }

        tokio::time::sleep(interval).await;

        let form = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("device_code", device.device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        let response = client
            .post(GOOGLE_OAUTH_TOKEN_URL)
            .form(&form)
            .send()
            .await
            .context("Failed to poll device code")?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            let token_response: TokenResponse =
                serde_json::from_str(&body).context("Failed to parse token response")?;

            let expires_at = token_response
                .expires_in
                .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

            return Ok(TokenSet {
                access_token: token_response.access_token,
                refresh_token: token_response.refresh_token,
                id_token: token_response.id_token,
                expires_at,
                token_type: token_response.token_type.or_else(|| Some("Bearer".into())),
                scope: token_response.scope,
            });
        }

        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            match err.error.as_str() {
                "authorization_pending" => {}
                "slow_down" => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                "access_denied" => {
                    anyhow::bail!("User denied authorization");
                }
                "expired_token" => {
                    anyhow::bail!("Device code expired");
                }
                _ => {
                    anyhow::bail!(
                        "Google OAuth error: {} - {}",
                        err.error,
                        err.error_description.unwrap_or_default()
                    );
                }
            }
        }
    }
}

/// Start a loopback server on port 1456, wait for the OAuth callback, and return the code.
///
/// On failure to bind, falls back to reading from stdin (for remote/headless environments).
pub async fn receive_loopback_code(expected_state: &str, timeout: Duration) -> Result<String> {
    let listener = match TcpListener::bind("127.0.0.1:1456").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Could not bind to localhost:1456: {e}");
            eprintln!("Falling back to manual input.");
            return receive_code_from_stdin(expected_state).await;
        }
    };

    println!("Waiting for callback at http://localhost:1456/auth/callback ...");
    println!("(Or paste the full callback URL / authorization code here if running remotely)");

    tokio::select! {
        accept_result = async {
            tokio::time::timeout(timeout, listener.accept()).await
        } => {
            match accept_result {
                Ok(Ok((mut stream, _))) => {
                    let mut buffer = vec![0u8; 4096];
                    let n = stream
                        .read(&mut buffer)
                        .await
                        .context("Failed to read from callback connection")?;

                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let (code, state) = parse_callback_request(&request)?;

                    if state != expected_state {
                        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n\
                             <html><body><h1>State mismatch</h1><p>Please try again.</p></body></html>";
                        let _ = stream.write_all(response.as_bytes()).await;
                        anyhow::bail!("OAuth state mismatch");
                    }

                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                         <html><body><h1>OpenAgentiX login complete!</h1><p>You can close this window.</p></body></html>";
                    let _ = stream.write_all(response.as_bytes()).await;

                    Ok(code)
                }
                Ok(Err(e)) => Err(anyhow::anyhow!("Failed to accept connection: {e}")),
                Err(_) => {
                    eprintln!("\nCallback timeout. Falling back to manual input.");
                    receive_code_from_stdin(expected_state).await
                }
            }
        }
        stdin_result = receive_code_from_stdin(expected_state) => {
            stdin_result
        }
    }
}

/// Parse an OAuth authorization code from a redirect URL or raw string.
///
/// Validates the `state` parameter if `expected_state` is provided.
pub fn parse_code_from_redirect(input: &str, expected_state: Option<&str>) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("No OAuth code provided");
    }

    let query = if let Some((_, right)) = trimmed.split_once('?') {
        right
    } else {
        trimmed
    };

    let params = parse_query_params(query);

    if let Some(code) = params.get("code") {
        if let Some(expected) = expected_state {
            if let Some(actual) = params.get("state") {
                if actual != expected {
                    anyhow::bail!("OAuth state mismatch: expected {expected}, got {actual}");
                }
            }
        }
        return Ok(code.clone());
    }

    // Assume raw code if long enough and no spaces
    if trimmed.len() > 10 && !trimmed.contains(' ') && !trimmed.contains('&') {
        return Ok(trimmed.to_string());
    }

    anyhow::bail!("Could not parse OAuth code from input")
}

/// Extract the Google account email from a Google ID token (OIDC).
pub fn extract_account_email_from_id_token(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()?;

    #[derive(Deserialize)]
    struct IdTokenPayload {
        email: Option<String>,
    }

    let payload: IdTokenPayload = serde_json::from_slice(&payload).ok()?;
    payload.email
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_callback_request(request: &str) -> Result<(String, String)> {
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_string();

    let query_start = path.find('?').map(|i| i + 1).unwrap_or(path.len());
    let query = &path[query_start..];

    let mut code = None;
    let mut state = None;

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            match key {
                "code" => code = Some(crate::auth::url_decode(value)),
                "state" => state = Some(crate::auth::url_decode(value)),
                _ => {}
            }
        }
    }

    let code = code.ok_or_else(|| anyhow::anyhow!("No 'code' parameter in callback"))?;
    let state = state.ok_or_else(|| anyhow::anyhow!("No 'state' parameter in callback"))?;

    Ok((code, state))
}

async fn receive_code_from_stdin(expected_state: &str) -> Result<String> {
    use std::io::{self, BufRead};

    let expected = expected_state.to_string();
    let input = tokio::task::spawn_blocking(move || {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).ok();
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("No input received"));
        }
        parse_code_from_redirect(&trimmed, Some(&expected))
    })
    .await
    .context("Failed to read from stdin")??;

    Ok(input)
}
