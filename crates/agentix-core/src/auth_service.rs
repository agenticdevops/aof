//! AuthService: coordinates all three provider OAuth flows with token refresh and profile management.
//!
//! Adapted from zeroclaw's `auth/mod.rs` AuthService.
//!
//! ## Provider names
//!
//! Use [`normalize_provider`] to canonicalize provider aliases:
//! - `"openai"` | `"codex"` → `"openai"`
//! - `"anthropic"` | `"claude"` → `"anthropic"`
//! - `"gemini"` | `"google"` → `"gemini"`
//!
//! ## Token refresh
//!
//! OAuth tokens are refreshed with 3 attempts, 350ms × attempt delay, and a 10s
//! failure backoff. Per-profile async locking prevents duplicate refresh calls.

use crate::auth::{
    profile_id, select_profile_id, AuthProfile, AuthProfileKind, AuthProfilesData,
    AuthProfilesStore, TokenSet,
};
use crate::auth::{openai_oauth, gemini_oauth};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const OPENAI_PROVIDER: &str = "openai";
const ANTHROPIC_PROVIDER: &str = "anthropic";
const GEMINI_PROVIDER: &str = "gemini";
const DEFAULT_PROFILE_NAME: &str = "default";

/// Refresh tokens this many seconds before actual expiry to avoid races.
const REFRESH_SKEW_SECS: u64 = 90;
/// Back off this many seconds after a token refresh failure.
const REFRESH_FAILURE_BACKOFF_SECS: u64 = 10;
/// Maximum number of refresh attempts before giving up.
const OAUTH_REFRESH_MAX_ATTEMPTS: usize = 3;
/// Base delay (ms) for linear backoff between refresh retries (350ms × attempt).
const OAUTH_REFRESH_RETRY_BASE_DELAY_MS: u64 = 350;

// ---------------------------------------------------------------------------
// Static state for refresh locking and backoffs
// ---------------------------------------------------------------------------

static REFRESH_BACKOFFS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

// ---------------------------------------------------------------------------
// AuthService
// ---------------------------------------------------------------------------

/// Coordinator for all three provider OAuth credentials.
///
/// Manages profile storage, token refresh with retry/backoff, and per-profile
/// async locking to prevent duplicate refresh calls.
#[derive(Clone)]
pub struct AuthService {
    store: AuthProfilesStore,
    client: reqwest::Client,
}

impl AuthService {
    /// Create a new `AuthService` with the given state directory and encryption setting.
    ///
    /// - `state_dir` — directory where `auth-profiles.json` and `.secret_key` live
    /// - `encrypt_secrets` — when `true`, tokens are encrypted at rest with AES-256-GCM
    pub fn new(state_dir: &Path, encrypt_secrets: bool) -> Self {
        Self {
            store: AuthProfilesStore::new(state_dir, encrypt_secrets),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new `AuthService` from a state directory path.
    pub fn from_state_dir(state_dir: impl AsRef<Path>, encrypt_secrets: bool) -> Self {
        Self::new(state_dir.as_ref(), encrypt_secrets)
    }

    /// Load all profiles from disk.
    pub async fn load_profiles(&self) -> Result<AuthProfilesData> {
        self.store.load().await
    }

    /// Store OpenAI OAuth tokens for a profile.
    ///
    /// Returns the stored profile.
    pub async fn store_openai_tokens(
        &self,
        profile_name: &str,
        token_set: TokenSet,
        account_id: Option<String>,
        set_active: bool,
    ) -> Result<AuthProfile> {
        let mut profile = AuthProfile::new_oauth(OPENAI_PROVIDER, profile_name, token_set);
        profile.account_id = account_id;
        self.store.upsert_profile(profile.clone(), set_active).await?;
        Ok(profile)
    }

    /// Store Gemini OAuth tokens for a profile.
    ///
    /// Returns the stored profile.
    pub async fn store_gemini_tokens(
        &self,
        profile_name: &str,
        token_set: TokenSet,
        account_id: Option<String>,
        set_active: bool,
    ) -> Result<AuthProfile> {
        let mut profile = AuthProfile::new_oauth(GEMINI_PROVIDER, profile_name, token_set);
        profile.account_id = account_id;
        self.store.upsert_profile(profile.clone(), set_active).await?;
        Ok(profile)
    }

    /// Store a static bearer token or API key for any provider.
    pub async fn store_provider_token(
        &self,
        provider: &str,
        profile_name: &str,
        token: &str,
        metadata: HashMap<String, String>,
        set_active: bool,
    ) -> Result<AuthProfile> {
        let provider = normalize_provider(provider)?;
        let mut profile = AuthProfile::new_token(&provider, profile_name, token.to_string());
        profile.metadata.extend(metadata);
        self.store.upsert_profile(profile.clone(), set_active).await?;
        Ok(profile)
    }

    /// Set the active profile for a provider.
    ///
    /// Returns the profile ID that was set as active.
    pub async fn set_active_profile(
        &self,
        provider: &str,
        requested_profile: &str,
    ) -> Result<String> {
        let provider = normalize_provider(provider)?;
        let data = self.store.load().await?;
        let profile_id = resolve_requested_profile_id(&provider, requested_profile);

        let profile = data
            .profiles
            .get(&profile_id)
            .ok_or_else(|| anyhow::anyhow!("Auth profile not found: {profile_id}"))?;

        if profile.provider != provider {
            anyhow::bail!(
                "Profile {profile_id} belongs to provider {}, not {}",
                profile.provider,
                provider
            );
        }

        self.store.set_active_profile(&provider, &profile_id).await?;
        Ok(profile_id)
    }

    /// Remove a profile by provider and name. Returns `true` if the profile existed.
    pub async fn remove_profile(&self, provider: &str, requested_profile: &str) -> Result<bool> {
        let provider = normalize_provider(provider)?;
        let profile_id = resolve_requested_profile_id(&provider, requested_profile);
        self.store.remove_profile(&profile_id).await
    }

    /// Get a profile for a provider, following the selection priority chain.
    pub async fn get_profile(
        &self,
        provider: &str,
        profile_override: Option<&str>,
    ) -> Result<Option<AuthProfile>> {
        let provider = normalize_provider(provider)?;
        let data = self.store.load().await?;
        let Some(profile_id) = select_profile_id(&data, &provider, profile_override) else {
            return Ok(None);
        };
        Ok(data.profiles.get(&profile_id).cloned())
    }

    /// Get the raw bearer token string for a provider profile.
    ///
    /// For OAuth profiles, returns the current access token (without refresh).
    /// For token profiles, returns the stored static token.
    pub async fn get_provider_bearer_token(
        &self,
        provider: &str,
        profile_override: Option<&str>,
    ) -> Result<Option<String>> {
        let profile = self.get_profile(provider, profile_override).await?;
        let Some(profile) = profile else {
            return Ok(None);
        };

        let credential = match profile.kind {
            AuthProfileKind::Token => profile.token,
            AuthProfileKind::OAuth => profile.token_set.map(|t| t.access_token),
        };

        Ok(credential.filter(|t| !t.trim().is_empty()))
    }

    /// Get a valid OpenAI access token, refreshing if it is expiring soon.
    ///
    /// Returns `None` if no OpenAI profile exists.
    /// Returns an error if refresh fails after retries.
    ///
    /// - Checks expiry with 90s skew window
    /// - Acquires per-profile async lock before refreshing
    /// - Re-loads profile after acquiring lock (avoids duplicate refreshes)
    /// - Retries up to 3 times with 350ms × attempt delay
    /// - Sets 10s backoff after repeated failures
    pub async fn get_valid_openai_access_token(
        &self,
        profile_override: Option<&str>,
    ) -> Result<Option<String>> {
        let data = self.store.load().await?;
        let Some(pid) = select_profile_id(&data, OPENAI_PROVIDER, profile_override) else {
            return Ok(None);
        };

        let Some(profile) = data.profiles.get(&pid) else {
            return Ok(None);
        };

        let Some(token_set) = profile.token_set.as_ref() else {
            anyhow::bail!("OpenAI auth profile is not OAuth-based: {pid}");
        };

        if !token_set.is_expiring_within(Duration::from_secs(REFRESH_SKEW_SECS)) {
            return Ok(Some(token_set.access_token.clone()));
        }

        let Some(refresh_token) = token_set.refresh_token.clone() else {
            // No refresh token — return current access token even if expiring
            return Ok(Some(token_set.access_token.clone()));
        };

        let refresh_lock = refresh_lock_for_profile(&pid);
        let _guard = refresh_lock.lock().await;

        // Re-load after acquiring lock to avoid duplicate refreshes.
        let data = self.store.load().await?;
        let Some(latest_profile) = data.profiles.get(&pid) else {
            return Ok(None);
        };

        let Some(latest_tokens) = latest_profile.token_set.as_ref() else {
            anyhow::bail!("OpenAI auth profile is missing token set after lock: {pid}");
        };

        if !latest_tokens.is_expiring_within(Duration::from_secs(REFRESH_SKEW_SECS)) {
            return Ok(Some(latest_tokens.access_token.clone()));
        }

        let refresh_token = latest_tokens.refresh_token.clone().unwrap_or(refresh_token);

        if let Some(remaining) = refresh_backoff_remaining(&pid) {
            anyhow::bail!(
                "OpenAI token refresh is in backoff for {remaining}s due to previous failures"
            );
        }

        let mut refreshed =
            match refresh_openai_access_token_with_retries(&self.client, &refresh_token).await {
                Ok(tokens) => {
                    clear_refresh_backoff(&pid);
                    tokens
                }
                Err(err) => {
                    set_refresh_backoff(&pid, Duration::from_secs(REFRESH_FAILURE_BACKOFF_SECS));
                    return Err(err);
                }
            };

        // Preserve existing refresh token if new one wasn't returned
        if refreshed.refresh_token.is_none() {
            refreshed.refresh_token.clone_from(&latest_tokens.refresh_token);
        }

        let account_id = openai_oauth::extract_account_id_from_jwt(&refreshed.access_token)
            .or_else(|| latest_profile.account_id.clone());

        let updated = self
            .store
            .update_profile(&pid, |profile| {
                profile.kind = AuthProfileKind::OAuth;
                profile.token_set = Some(refreshed.clone());
                profile.account_id.clone_from(&account_id);
                Ok(())
            })
            .await?;

        Ok(updated.token_set.map(|t| t.access_token))
    }

    /// Get a valid Gemini access token, refreshing if it is expiring soon.
    ///
    /// Returns `None` if no Gemini profile exists.
    /// Returns an error if refresh fails after retries.
    ///
    /// Same retry/backoff/locking pattern as OpenAI.
    pub async fn get_valid_gemini_access_token(
        &self,
        profile_override: Option<&str>,
    ) -> Result<Option<String>> {
        let data = self.store.load().await?;
        let Some(pid) = select_profile_id(&data, GEMINI_PROVIDER, profile_override) else {
            return Ok(None);
        };

        let Some(profile) = data.profiles.get(&pid) else {
            return Ok(None);
        };

        let Some(token_set) = profile.token_set.as_ref() else {
            anyhow::bail!("Gemini auth profile is not OAuth-based: {pid}");
        };

        if !token_set.is_expiring_within(Duration::from_secs(REFRESH_SKEW_SECS)) {
            return Ok(Some(token_set.access_token.clone()));
        }

        let Some(refresh_token) = token_set.refresh_token.clone() else {
            return Ok(Some(token_set.access_token.clone()));
        };

        let refresh_lock = refresh_lock_for_profile(&pid);
        let _guard = refresh_lock.lock().await;

        let data = self.store.load().await?;
        let Some(latest_profile) = data.profiles.get(&pid) else {
            return Ok(None);
        };

        let Some(latest_tokens) = latest_profile.token_set.as_ref() else {
            anyhow::bail!("Gemini auth profile is missing token set after lock: {pid}");
        };

        if !latest_tokens.is_expiring_within(Duration::from_secs(REFRESH_SKEW_SECS)) {
            return Ok(Some(latest_tokens.access_token.clone()));
        }

        let refresh_token = latest_tokens.refresh_token.clone().unwrap_or(refresh_token);

        if let Some(remaining) = refresh_backoff_remaining(&pid) {
            anyhow::bail!(
                "Gemini token refresh is in backoff for {remaining}s due to previous failures"
            );
        }

        let mut refreshed =
            match refresh_gemini_access_token_with_retries(&self.client, &refresh_token).await {
                Ok(tokens) => {
                    clear_refresh_backoff(&pid);
                    tokens
                }
                Err(err) => {
                    set_refresh_backoff(&pid, Duration::from_secs(REFRESH_FAILURE_BACKOFF_SECS));
                    return Err(err);
                }
            };

        if refreshed.refresh_token.is_none() {
            refreshed.refresh_token.clone_from(&latest_tokens.refresh_token);
        }

        let account_id = refreshed
            .id_token
            .as_deref()
            .and_then(gemini_oauth::extract_account_email_from_id_token)
            .or_else(|| latest_profile.account_id.clone());

        let updated = self
            .store
            .update_profile(&pid, |profile| {
                profile.kind = AuthProfileKind::OAuth;
                profile.token_set = Some(refreshed.clone());
                profile.account_id.clone_from(&account_id);
                Ok(())
            })
            .await?;

        Ok(updated.token_set.map(|t| t.access_token))
    }

    /// Get the Gemini profile (for provider initialization).
    pub async fn get_gemini_profile(
        &self,
        profile_override: Option<&str>,
    ) -> Result<Option<AuthProfile>> {
        self.get_profile(GEMINI_PROVIDER, profile_override).await
    }
}

// ---------------------------------------------------------------------------
// Provider name normalization
// ---------------------------------------------------------------------------

/// Normalize a provider name to its canonical form.
///
/// - `"openai"` | `"codex"` → `"openai"`
/// - `"anthropic"` | `"claude"` → `"anthropic"`
/// - `"gemini"` | `"google"` → `"gemini"`
/// - Other non-empty strings → returned as-is (lowercased)
/// - Empty string → error
pub fn normalize_provider(provider: &str) -> Result<String> {
    let normalized = provider.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "openai" | "codex" | "openai-codex" | "openai_codex" => Ok(OPENAI_PROVIDER.to_string()),
        "anthropic" | "claude" | "claude-code" => Ok(ANTHROPIC_PROVIDER.to_string()),
        "gemini" | "google" | "vertex" => Ok(GEMINI_PROVIDER.to_string()),
        other if !other.is_empty() => Ok(other.to_string()),
        _ => anyhow::bail!("Provider name cannot be empty"),
    }
}

// ---------------------------------------------------------------------------
// Profile ID helpers
// ---------------------------------------------------------------------------

pub fn default_profile_id(provider: &str) -> String {
    profile_id(provider, DEFAULT_PROFILE_NAME)
}

fn resolve_requested_profile_id(provider: &str, requested: &str) -> String {
    if requested.contains(':') {
        requested.to_string()
    } else {
        profile_id(provider, requested)
    }
}

// ---------------------------------------------------------------------------
// Token refresh with retries
// ---------------------------------------------------------------------------

async fn refresh_openai_access_token_with_retries(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenSet> {
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 1..=OAUTH_REFRESH_MAX_ATTEMPTS {
        match openai_oauth::refresh_access_token(client, refresh_token).await {
            Ok(tokens) => return Ok(tokens),
            Err(err) => {
                let should_retry = attempt < OAUTH_REFRESH_MAX_ATTEMPTS;
                tracing::warn!(
                    attempt,
                    max_attempts = OAUTH_REFRESH_MAX_ATTEMPTS,
                    retry = should_retry,
                    error = %err,
                    "OpenAI token refresh failed"
                );
                last_error = Some(err);
                if should_retry {
                    tokio::time::sleep(Duration::from_millis(
                        OAUTH_REFRESH_RETRY_BASE_DELAY_MS * attempt as u64,
                    ))
                    .await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("OpenAI token refresh failed")))
}

async fn refresh_gemini_access_token_with_retries(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenSet> {
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 1..=OAUTH_REFRESH_MAX_ATTEMPTS {
        match gemini_oauth::refresh_access_token(client, refresh_token).await {
            Ok(tokens) => return Ok(tokens),
            Err(err) => {
                let should_retry = attempt < OAUTH_REFRESH_MAX_ATTEMPTS;
                tracing::warn!(
                    attempt,
                    max_attempts = OAUTH_REFRESH_MAX_ATTEMPTS,
                    retry = should_retry,
                    error = %err,
                    "Gemini token refresh failed"
                );
                last_error = Some(err);
                if should_retry {
                    tokio::time::sleep(Duration::from_millis(
                        OAUTH_REFRESH_RETRY_BASE_DELAY_MS * attempt as u64,
                    ))
                    .await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Gemini token refresh failed")))
}

// ---------------------------------------------------------------------------
// Per-profile refresh locking
// ---------------------------------------------------------------------------

/// Get or create a per-profile async mutex to prevent concurrent token refreshes.
fn refresh_lock_for_profile(profile_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();

    let table = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = table.lock().expect("refresh lock table poisoned");

    guard
        .entry(profile_id.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

// ---------------------------------------------------------------------------
// Refresh backoff (10s cooldown after failures)
// ---------------------------------------------------------------------------

/// Returns remaining backoff seconds if a profile is in cooldown, else `None`.
fn refresh_backoff_remaining(profile_id: &str) -> Option<u64> {
    let map = REFRESH_BACKOFFS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().ok()?;
    let now = Instant::now();
    let deadline = guard.get(profile_id).copied()?;
    if deadline <= now {
        guard.remove(profile_id);
        return None;
    }
    Some((deadline - now).as_secs().max(1))
}

/// Set a refresh backoff for a profile (called after refresh failure).
fn set_refresh_backoff(profile_id: &str, duration: Duration) {
    let map = REFRESH_BACKOFFS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = map.lock() {
        guard.insert(profile_id.to_string(), Instant::now() + duration);
    }
}

/// Clear the refresh backoff for a profile (called after successful refresh).
fn clear_refresh_backoff(profile_id: &str) {
    let map = REFRESH_BACKOFFS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = map.lock() {
        guard.remove(profile_id);
    }
}

// ---------------------------------------------------------------------------
// Path helper
// ---------------------------------------------------------------------------

/// Derive the state directory from a config file path.
pub fn state_dir_from_config_path(config_path: impl AsRef<Path>) -> PathBuf {
    config_path
        .as_ref()
        .parent()
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}
