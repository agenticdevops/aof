//! OAuth auth profiles, encrypted token storage, PKCE utilities, and per-provider flows.
//!
//! This module provides the foundation for LLM subscription proxy authentication:
//!
//! - [`AuthProfile`] / [`TokenSet`] — credentials for a single provider account
//! - [`AuthProfilesStore`] — encrypted JSON profile store at `~/.agentix/auth-profiles.json`
//! - [`PkceState`] — PKCE code verifier + challenge for OAuth2 authorization code flow
//! - [`generate_pkce_state`] / [`url_encode`] / [`parse_query_params`] — OAuth2 utilities
//! - [`profile_id`] / [`select_profile_id`] — profile selection with priority chain
//! - [`openai_oauth`] — OpenAI OAuth2 flow (auth.openai.com, loopback on :1455)
//! - [`gemini_oauth`] — Google/Gemini OAuth2 flow (loopback on :1456)
//! - [`anthropic_token`] — Anthropic API key / Bearer token detection
//!
//! ## Profile Selection Priority
//!
//! 1. Explicit override (caller-supplied profile name)
//! 2. Active profile for the provider (stored in `active_profiles`)
//! 3. Profile named `"default"` for the provider
//! 4. First profile found for the provider
//!
//! ## Encryption
//!
//! Token values are encrypted at rest using AES-256-GCM with a random key stored
//! in `<state_dir>/.secret_key`. Encrypted values are hex-encoded with an `enc2:` prefix.
//! If encryption is disabled, values are stored as plaintext.

pub mod anthropic_token;
pub mod gemini_oauth;
pub mod openai_oauth;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use anyhow::{Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CURRENT_SCHEMA_VERSION: u32 = 1;
const PROFILES_FILENAME: &str = "auth-profiles.json";
const LOCK_FILENAME: &str = "auth-profiles.lock";
const LOCK_WAIT_MS: u64 = 50;
const LOCK_TIMEOUT_MS: u64 = 10_000;

/// Nonce length for AES-256-GCM (12 bytes).
const NONCE_LEN: usize = 12;
/// Key length for AES-256-GCM (32 bytes / 256 bits).
const KEY_LEN: usize = 32;
/// Key file name stored alongside the profile store.
const SECRET_KEY_FILENAME: &str = ".secret_key";

// ---------------------------------------------------------------------------
// Auth refresh constants (from zeroclaw)
// ---------------------------------------------------------------------------

/// Refresh tokens this many seconds before expiry to avoid races.
pub const OPENAI_REFRESH_SKEW_SECS: u64 = 90;
/// Back off this many seconds after a token refresh failure.
pub const OPENAI_REFRESH_FAILURE_BACKOFF_SECS: u64 = 10;
/// Maximum number of refresh attempts before giving up.
pub const OAUTH_REFRESH_MAX_ATTEMPTS: usize = 3;
/// Base delay (ms) for exponential backoff between refresh retries.
pub const OAUTH_REFRESH_RETRY_BASE_DELAY_MS: u64 = 350;

// ---------------------------------------------------------------------------
// AuthProfileKind
// ---------------------------------------------------------------------------

/// Distinguishes between OAuth2 token-set credentials and static bearer tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthProfileKind {
    /// OAuth2 credentials: access token + optional refresh token.
    OAuth,
    /// Static bearer token or API key.
    Token,
}

// ---------------------------------------------------------------------------
// TokenSet
// ---------------------------------------------------------------------------

/// An OAuth2 token set for a provider account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    /// The current OAuth2 access token.
    pub access_token: String,
    /// Refresh token for obtaining new access tokens.
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// OpenID Connect ID token (provider-dependent).
    #[serde(default)]
    pub id_token: Option<String>,
    /// When the access token expires. `None` means no known expiry.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// Token type string (usually `"Bearer"`).
    #[serde(default)]
    pub token_type: Option<String>,
    /// OAuth2 scopes granted.
    #[serde(default)]
    pub scope: Option<String>,
}

impl TokenSet {
    /// Returns `true` if the access token will expire within `skew` of now.
    ///
    /// Returns `false` if no expiry is set (token assumed permanent).
    pub fn is_expiring_within(&self, skew: Duration) -> bool {
        match self.expires_at {
            Some(expires_at) => {
                let now_plus_skew =
                    Utc::now() + chrono::Duration::from_std(skew).unwrap_or_default();
                expires_at <= now_plus_skew
            }
            None => false,
        }
    }
}

// ---------------------------------------------------------------------------
// AuthProfile
// ---------------------------------------------------------------------------

/// A stored credential profile for a single provider account.
///
/// Secrets (`token_set`, `token`) are NOT printed in `Debug` output to prevent
/// accidental exposure in logs.
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    /// Unique profile ID in `"{provider}:{profile_name}"` format.
    pub id: String,
    /// Provider identifier (e.g., `"anthropic"`, `"openai"`, `"gemini"`).
    pub provider: String,
    /// Human-readable profile name (e.g., `"default"`, `"work"`).
    pub profile_name: String,
    /// Whether this profile uses OAuth or a static token.
    pub kind: AuthProfileKind,
    /// Provider-reported account ID (e.g., sub claim from OIDC).
    #[serde(default)]
    pub account_id: Option<String>,
    /// Provider-reported workspace/org ID.
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// OAuth2 token set (populated for `kind = OAuth`).
    #[serde(default)]
    pub token_set: Option<TokenSet>,
    /// Static bearer token (populated for `kind = Token`).
    #[serde(default)]
    pub token: Option<String>,
    /// Arbitrary key-value metadata (e.g., email, display name).
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    /// When the profile was first created.
    pub created_at: DateTime<Utc>,
    /// When the profile was last updated.
    pub updated_at: DateTime<Utc>,
}

impl std::fmt::Debug for AuthProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthProfile")
            .field("id", &self.id)
            .field("provider", &self.provider)
            .field("profile_name", &self.profile_name)
            .field("kind", &self.kind)
            .field("account_id", &self.account_id)
            .field("workspace_id", &self.workspace_id)
            .field("metadata", &self.metadata)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish_non_exhaustive()
    }
}

impl AuthProfile {
    /// Create a new OAuth profile with the given token set.
    pub fn new_oauth(provider: &str, profile_name: &str, token_set: TokenSet) -> Self {
        let now = Utc::now();
        let id = profile_id(provider, profile_name);
        Self {
            id,
            provider: provider.to_string(),
            profile_name: profile_name.to_string(),
            kind: AuthProfileKind::OAuth,
            account_id: None,
            workspace_id: None,
            token_set: Some(token_set),
            token: None,
            metadata: BTreeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new static token profile.
    pub fn new_token(provider: &str, profile_name: &str, token: String) -> Self {
        let now = Utc::now();
        let id = profile_id(provider, profile_name);
        Self {
            id,
            provider: provider.to_string(),
            profile_name: profile_name.to_string(),
            kind: AuthProfileKind::Token,
            account_id: None,
            workspace_id: None,
            token_set: None,
            token: Some(token),
            metadata: BTreeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

// ---------------------------------------------------------------------------
// AuthProfilesData
// ---------------------------------------------------------------------------

/// In-memory representation of the decrypted profile store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfilesData {
    /// Monotonically increasing schema version.
    pub schema_version: u32,
    /// Last modification time of the store.
    pub updated_at: DateTime<Utc>,
    /// Maps `provider` → `profile_id` for each provider's active profile.
    pub active_profiles: BTreeMap<String, String>,
    /// All profiles keyed by their `profile_id`.
    pub profiles: BTreeMap<String, AuthProfile>,
}

impl Default for AuthProfilesData {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            updated_at: Utc::now(),
            active_profiles: BTreeMap::new(),
            profiles: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// AuthProfilesStore
// ---------------------------------------------------------------------------

/// Encrypted JSON profile store at `<state_dir>/auth-profiles.json`.
///
/// Implements file-level locking with 50ms polling and a 10s timeout.
/// Uses atomic writes (temp file + rename) to prevent partial writes.
///
/// Token values are encrypted with AES-256-GCM using a random key stored
/// at `<state_dir>/.secret_key`. Encrypted values use the `enc2:` prefix.
#[derive(Debug, Clone)]
pub struct AuthProfilesStore {
    /// Path to the JSON file.
    path: PathBuf,
    /// Path to the lock file.
    lock_path: PathBuf,
    /// Encryption key file path.
    key_path: PathBuf,
    /// Whether encryption is enabled.
    encrypt_secrets: bool,
}

impl AuthProfilesStore {
    /// Create a new store rooted at `state_dir`.
    ///
    /// - `state_dir` — directory where `auth-profiles.json` and `.secret_key` live.
    /// - `encrypt_secrets` — when `false`, tokens are stored as plaintext.
    pub fn new(state_dir: &Path, encrypt_secrets: bool) -> Self {
        Self {
            path: state_dir.join(PROFILES_FILENAME),
            lock_path: state_dir.join(LOCK_FILENAME),
            key_path: state_dir.join(SECRET_KEY_FILENAME),
            encrypt_secrets,
        }
    }

    /// Path to the profile store file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load and decrypt all profiles from disk.
    pub async fn load(&self) -> Result<AuthProfilesData> {
        let _lock = self.acquire_lock().await?;
        self.load_locked().await
    }

    /// Insert or update a profile.
    ///
    /// If `set_active` is `true`, this profile becomes the active profile
    /// for its provider.
    pub async fn upsert_profile(&self, mut profile: AuthProfile, set_active: bool) -> Result<()> {
        let _lock = self.acquire_lock().await?;
        let mut data = self.load_locked().await?;

        profile.updated_at = Utc::now();
        if let Some(existing) = data.profiles.get(&profile.id) {
            profile.created_at = existing.created_at;
        }

        if set_active {
            data.active_profiles
                .insert(profile.provider.clone(), profile.id.clone());
        }

        data.profiles.insert(profile.id.clone(), profile);
        data.updated_at = Utc::now();
        self.save_locked(&data).await
    }

    /// Remove a profile by its ID. Returns `true` if the profile existed.
    pub async fn remove_profile(&self, profile_id: &str) -> Result<bool> {
        let _lock = self.acquire_lock().await?;
        let mut data = self.load_locked().await?;

        let removed = data.profiles.remove(profile_id).is_some();
        if !removed {
            return Ok(false);
        }

        data.active_profiles
            .retain(|_, active| active != profile_id);
        data.updated_at = Utc::now();
        self.save_locked(&data).await?;
        Ok(true)
    }

    /// Set the active profile for a provider.
    pub async fn set_active_profile(&self, provider: &str, profile_id: &str) -> Result<()> {
        let _lock = self.acquire_lock().await?;
        let mut data = self.load_locked().await?;

        if !data.profiles.contains_key(profile_id) {
            anyhow::bail!("Auth profile not found: {profile_id}");
        }

        data.active_profiles
            .insert(provider.to_string(), profile_id.to_string());
        data.updated_at = Utc::now();
        self.save_locked(&data).await
    }

    /// Clear the active profile for a provider (does not delete the profile).
    pub async fn clear_active_profile(&self, provider: &str) -> Result<()> {
        let _lock = self.acquire_lock().await?;
        let mut data = self.load_locked().await?;
        data.active_profiles.remove(provider);
        data.updated_at = Utc::now();
        self.save_locked(&data).await
    }

    /// Update a profile in place using a closure.
    ///
    /// Returns the updated profile after persisting.
    pub async fn update_profile<F>(&self, profile_id: &str, mut updater: F) -> Result<AuthProfile>
    where
        F: FnMut(&mut AuthProfile) -> Result<()>,
    {
        let _lock = self.acquire_lock().await?;
        let mut data = self.load_locked().await?;

        let profile = data
            .profiles
            .get_mut(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Auth profile not found: {profile_id}"))?;

        updater(profile)?;
        profile.updated_at = Utc::now();
        let updated_profile = profile.clone();
        data.updated_at = Utc::now();
        self.save_locked(&data).await?;
        Ok(updated_profile)
    }

    // -----------------------------------------------------------------------
    // Internal: locked I/O
    // -----------------------------------------------------------------------

    async fn load_locked(&self) -> Result<AuthProfilesData> {
        let mut persisted = self.read_persisted_locked().await?;
        let mut migrated = false;

        let mut profiles = BTreeMap::new();
        for (id, p) in &mut persisted.profiles {
            let (access_token, access_migrated) =
                self.decrypt_optional(p.access_token.as_deref())?;
            let (refresh_token, refresh_migrated) =
                self.decrypt_optional(p.refresh_token.as_deref())?;
            let (id_token, id_migrated) = self.decrypt_optional(p.id_token.as_deref())?;
            let (token, token_migrated) = self.decrypt_optional(p.token.as_deref())?;

            if let Some(value) = access_migrated {
                p.access_token = Some(value);
                migrated = true;
            }
            if let Some(value) = refresh_migrated {
                p.refresh_token = Some(value);
                migrated = true;
            }
            if let Some(value) = id_migrated {
                p.id_token = Some(value);
                migrated = true;
            }
            if let Some(value) = token_migrated {
                p.token = Some(value);
                migrated = true;
            }

            let kind = parse_profile_kind(&p.kind)?;
            let token_set = match kind {
                AuthProfileKind::OAuth => {
                    let access = access_token.ok_or_else(|| {
                        anyhow::anyhow!("OAuth profile missing access_token: {id}")
                    })?;
                    Some(TokenSet {
                        access_token: access,
                        refresh_token,
                        id_token,
                        expires_at: parse_optional_datetime(p.expires_at.as_deref())?,
                        token_type: p.token_type.clone(),
                        scope: p.scope.clone(),
                    })
                }
                AuthProfileKind::Token => None,
            };

            profiles.insert(
                id.clone(),
                AuthProfile {
                    id: id.clone(),
                    provider: p.provider.clone(),
                    profile_name: p.profile_name.clone(),
                    kind,
                    account_id: p.account_id.clone(),
                    workspace_id: p.workspace_id.clone(),
                    token_set,
                    token,
                    metadata: p.metadata.clone(),
                    created_at: parse_datetime_with_fallback(&p.created_at),
                    updated_at: parse_datetime_with_fallback(&p.updated_at),
                },
            );
        }

        if migrated {
            self.write_persisted_locked(&persisted).await?;
        }

        Ok(AuthProfilesData {
            schema_version: persisted.schema_version,
            updated_at: parse_datetime_with_fallback(&persisted.updated_at),
            active_profiles: persisted.active_profiles,
            profiles,
        })
    }

    async fn save_locked(&self, data: &AuthProfilesData) -> Result<()> {
        let mut persisted = PersistedAuthProfiles {
            schema_version: CURRENT_SCHEMA_VERSION,
            updated_at: data.updated_at.to_rfc3339(),
            active_profiles: data.active_profiles.clone(),
            profiles: BTreeMap::new(),
        };

        for (id, profile) in &data.profiles {
            let (access_token, refresh_token, id_token, expires_at, token_type, scope) =
                match (&profile.kind, &profile.token_set) {
                    (AuthProfileKind::OAuth, Some(token_set)) => (
                        self.encrypt_optional(Some(&token_set.access_token))?,
                        self.encrypt_optional(token_set.refresh_token.as_deref())?,
                        self.encrypt_optional(token_set.id_token.as_deref())?,
                        token_set.expires_at.as_ref().map(DateTime::to_rfc3339),
                        token_set.token_type.clone(),
                        token_set.scope.clone(),
                    ),
                    _ => (None, None, None, None, None, None),
                };

            let token = self.encrypt_optional(profile.token.as_deref())?;

            persisted.profiles.insert(
                id.clone(),
                PersistedAuthProfile {
                    provider: profile.provider.clone(),
                    profile_name: profile.profile_name.clone(),
                    kind: profile_kind_to_string(profile.kind).to_string(),
                    account_id: profile.account_id.clone(),
                    workspace_id: profile.workspace_id.clone(),
                    access_token,
                    refresh_token,
                    id_token,
                    token,
                    expires_at,
                    token_type,
                    scope,
                    metadata: profile.metadata.clone(),
                    created_at: profile.created_at.to_rfc3339(),
                    updated_at: profile.updated_at.to_rfc3339(),
                },
            );
        }

        self.write_persisted_locked(&persisted).await
    }

    async fn read_persisted_locked(&self) -> Result<PersistedAuthProfiles> {
        if !self.path.exists() {
            return Ok(PersistedAuthProfiles::default());
        }

        let bytes = fs::read(&self.path).await.with_context(|| {
            format!(
                "Failed to read auth profile store at {}",
                self.path.display()
            )
        })?;

        if bytes.is_empty() {
            return Ok(PersistedAuthProfiles::default());
        }

        let mut persisted: PersistedAuthProfiles =
            serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "Failed to parse auth profile store at {}",
                    self.path.display()
                )
            })?;

        if persisted.schema_version == 0 {
            persisted.schema_version = CURRENT_SCHEMA_VERSION;
        }

        if persisted.schema_version > CURRENT_SCHEMA_VERSION {
            anyhow::bail!(
                "Unsupported auth profile schema version {} (max supported: {})",
                persisted.schema_version,
                CURRENT_SCHEMA_VERSION
            );
        }

        Ok(persisted)
    }

    async fn write_persisted_locked(&self, persisted: &PersistedAuthProfiles) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create auth profile directory at {}",
                    parent.display()
                )
            })?;
        }

        let json =
            serde_json::to_vec_pretty(persisted).context("Failed to serialize auth profiles")?;
        let tmp_name = format!(
            "{}.tmp.{}.{}",
            PROFILES_FILENAME,
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let tmp_path = self.path.with_file_name(tmp_name);

        fs::write(&tmp_path, &json).await.with_context(|| {
            format!(
                "Failed to write temporary auth profile file at {}",
                tmp_path.display()
            )
        })?;

        fs::rename(&tmp_path, &self.path).await.with_context(|| {
            format!(
                "Failed to replace auth profile store at {}",
                self.path.display()
            )
        })?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Internal: encryption / decryption helpers
    // -----------------------------------------------------------------------

    fn encrypt_optional(&self, value: Option<&str>) -> Result<Option<String>> {
        match value {
            Some(v) if !v.is_empty() => self.encrypt_value(v).map(Some),
            Some(_) | None => Ok(None),
        }
    }

    /// Decrypt and optionally migrate a stored value.
    ///
    /// Returns `(plaintext, Some(new_encrypted_value))` if the value was
    /// plaintext and encryption is enabled (triggering a migration), or
    /// `(plaintext, None)` if already encrypted or encryption is off.
    fn decrypt_optional(&self, value: Option<&str>) -> Result<(Option<String>, Option<String>)> {
        match value {
            Some(v) if !v.is_empty() => {
                let (plaintext, migrated) = self.decrypt_and_migrate(v)?;
                Ok((Some(plaintext), migrated))
            }
            Some(_) | None => Ok((None, None)),
        }
    }

    /// Encrypt `plaintext` to `enc2:<hex(nonce || ciphertext)>`.
    /// If `encrypt_secrets` is `false`, returns the plaintext unchanged.
    fn encrypt_value(&self, plaintext: &str) -> Result<String> {
        if !self.encrypt_secrets {
            return Ok(plaintext.to_string());
        }

        let key_bytes = self.load_or_create_key()?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

        let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        blob.extend_from_slice(&nonce_bytes);
        blob.extend_from_slice(&ciphertext);

        Ok(format!("enc2:{}", hex_encode(&blob)))
    }

    /// Decrypt a stored value and return plaintext.
    ///
    /// - `enc2:<hex>` — AES-256-GCM (current format)
    /// - No prefix — plaintext (returned as-is; returns migration value if encryption enabled)
    ///
    /// Returns `(plaintext, Option<new_encrypted_form>)`. The second element is
    /// `Some(...)` when the input was plaintext and encryption is now enabled
    /// (so the caller can persist the encrypted form).
    fn decrypt_and_migrate(&self, value: &str) -> Result<(String, Option<String>)> {
        if let Some(hex_str) = value.strip_prefix("enc2:") {
            let plaintext = self.decrypt_enc2(hex_str)?;
            Ok((plaintext, None))
        } else {
            // Plaintext — migrate to enc2 if encryption is enabled
            if self.encrypt_secrets {
                let encrypted = self.encrypt_value(value)?;
                Ok((value.to_string(), Some(encrypted)))
            } else {
                Ok((value.to_string(), None))
            }
        }
    }

    fn decrypt_enc2(&self, hex_str: &str) -> Result<String> {
        let blob = hex_decode(hex_str).context("Failed to decode enc2: prefix (corrupt hex)")?;
        anyhow::ensure!(blob.len() > NONCE_LEN, "Encrypted value too short (missing nonce)");

        let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let key_bytes = self.load_or_create_key()?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or tampered data"))?;

        String::from_utf8(plaintext_bytes)
            .context("Decrypted auth value is not valid UTF-8 — corrupt data")
    }

    /// Load the AES key from disk, creating it if it doesn't exist.
    fn load_or_create_key(&self) -> Result<Vec<u8>> {
        if self.key_path.exists() {
            let hex_key = std::fs::read_to_string(&self.key_path)
                .context("Failed to read auth secret key file")?;
            hex_decode(hex_key.trim()).context("Auth secret key file is corrupt")
        } else {
            let mut key = vec![0u8; KEY_LEN];
            OsRng.fill_bytes(&mut key);

            if let Some(parent) = self.key_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let key_hex = hex_encode(&key);
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.key_path)
            {
                Ok(mut key_file) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        key_file
                            .set_permissions(std::fs::Permissions::from_mode(0o600))
                            .context("Failed to set auth key file permissions")?;
                    }
                    write!(key_file, "{}", key_hex)
                        .context("Failed to write auth key file")?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Another process created it — read their key
                    let hex_key = std::fs::read_to_string(&self.key_path)
                        .context("Failed to read auth secret key file after race")?;
                    return hex_decode(hex_key.trim())
                        .context("Auth secret key file is corrupt (race)");
                }
                Err(e) => {
                    return Err(e).context("Failed to create auth key file");
                }
            }

            Ok(key)
        }
    }

    // -----------------------------------------------------------------------
    // Internal: file locking
    // -----------------------------------------------------------------------

    async fn acquire_lock(&self) -> Result<AuthProfileLockGuard> {
        if let Some(parent) = self.lock_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create lock directory at {}", parent.display())
            })?;
        }

        let mut waited = 0_u64;
        loop {
            match OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.lock_path)
                .await
            {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    writeln!(&mut buffer, "pid={}", std::process::id())?;
                    if let Err(e) = file.write_all(&buffer).await {
                        fs::remove_file(&self.lock_path)
                            .await
                            .inspect(|_| {
                                tracing::error!("Failed to remove auth profile lock file");
                            })
                            .ok();
                        return Err(e).with_context(|| {
                            format!(
                                "Failed to write auth profile lock at {}",
                                self.lock_path.display()
                            )
                        });
                    }
                    return Ok(AuthProfileLockGuard {
                        lock_path: self.lock_path.clone(),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if waited >= LOCK_TIMEOUT_MS {
                        anyhow::bail!(
                            "Timed out waiting for auth profile lock at {}",
                            self.lock_path.display()
                        );
                    }
                    sleep(Duration::from_millis(LOCK_WAIT_MS)).await;
                    waited = waited.saturating_add(LOCK_WAIT_MS);
                }
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!(
                            "Failed to create auth profile lock at {}",
                            self.lock_path.display()
                        )
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lock guard
// ---------------------------------------------------------------------------

struct AuthProfileLockGuard {
    lock_path: PathBuf,
}

impl Drop for AuthProfileLockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

// ---------------------------------------------------------------------------
// Persisted (on-disk) representations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAuthProfiles {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default = "default_now_rfc3339")]
    updated_at: String,
    #[serde(default)]
    active_profiles: BTreeMap<String, String>,
    #[serde(default)]
    profiles: BTreeMap<String, PersistedAuthProfile>,
}

impl Default for PersistedAuthProfiles {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            updated_at: default_now_rfc3339(),
            active_profiles: BTreeMap::new(),
            profiles: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedAuthProfile {
    provider: String,
    profile_name: String,
    kind: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default = "default_now_rfc3339")]
    created_at: String,
    #[serde(default = "default_now_rfc3339")]
    updated_at: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

fn default_now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn parse_profile_kind(value: &str) -> Result<AuthProfileKind> {
    match value {
        "oauth" => Ok(AuthProfileKind::OAuth),
        "token" => Ok(AuthProfileKind::Token),
        other => anyhow::bail!("Unsupported auth profile kind: {other}"),
    }
}

fn profile_kind_to_string(kind: AuthProfileKind) -> &'static str {
    match kind {
        AuthProfileKind::OAuth => "oauth",
        AuthProfileKind::Token => "token",
    }
}

fn parse_optional_datetime(value: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    value.map(parse_datetime).transpose()
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .with_context(|| format!("Invalid RFC3339 timestamp: {value}"))
}

fn parse_datetime_with_fallback(value: &str) -> DateTime<Utc> {
    parse_datetime(value).unwrap_or_else(|_| Utc::now())
}

// ---------------------------------------------------------------------------
// Profile ID helpers
// ---------------------------------------------------------------------------

/// Build a profile ID from a provider and profile name: `"{provider}:{profile_name}"`.
pub fn profile_id(provider: &str, profile_name: &str) -> String {
    format!("{}:{}", provider.trim(), profile_name.trim())
}

/// Select a profile ID from `data` for `provider`, following the priority chain:
///
/// 1. Explicit override → exact match or `{provider}:{override}`
/// 2. Active profile for the provider
/// 3. `"{provider}:default"`
/// 4. First profile for the provider
pub fn select_profile_id(
    data: &AuthProfilesData,
    provider: &str,
    profile_override: Option<&str>,
) -> Option<String> {
    if let Some(override_profile) = profile_override {
        let requested = if override_profile.contains(':') {
            override_profile.to_string()
        } else {
            profile_id(provider, override_profile)
        };
        if data.profiles.contains_key(&requested) {
            return Some(requested);
        }
        return None;
    }

    if let Some(active) = data.active_profiles.get(provider) {
        if data.profiles.contains_key(active) {
            return Some(active.clone());
        }
    }

    let default = profile_id(provider, "default");
    if data.profiles.contains_key(&default) {
        return Some(default);
    }

    data.profiles
        .iter()
        .find_map(|(id, profile)| (profile.provider == provider).then(|| id.clone()))
}

// ---------------------------------------------------------------------------
// Hex encode/decode helpers
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim();
    anyhow::ensure!(hex.len() % 2 == 0, "Hex string has odd length");
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .with_context(|| format!("Invalid hex byte at position {i}"))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// PkceState and OAuth common utilities
// ---------------------------------------------------------------------------

/// PKCE state container for OAuth2 authorization code flow.
///
/// Generated with [`generate_pkce_state`].
#[derive(Debug, Clone)]
pub struct PkceState {
    /// The raw code verifier (random base64url string, >= 43 chars).
    pub code_verifier: String,
    /// The S256 code challenge (SHA-256 of verifier, base64url-encoded).
    pub code_challenge: String,
    /// Random state parameter for CSRF protection.
    pub state: String,
}

/// Generate a new PKCE state with cryptographically random values.
///
/// The code verifier is a 64-byte random value base64url-encoded (87 chars).
/// The code challenge is `BASE64URL(SHA-256(code_verifier))` (S256 method).
/// The state is a 24-byte random value base64url-encoded (32 chars).
pub fn generate_pkce_state() -> PkceState {
    let code_verifier = random_base64url(64);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    PkceState {
        code_verifier,
        code_challenge,
        state: random_base64url(24),
    }
}

/// Generate a cryptographically random base64url-encoded string of `byte_len` random bytes.
pub fn random_base64url(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// URL-encode a string using RFC 3986 percent encoding.
///
/// Only unreserved characters (`A-Z`, `a-z`, `0-9`, `-`, `_`, `.`, `~`) are
/// left unencoded; all others are replaced with `%XX`.
pub fn url_encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect::<String>()
}

/// URL-decode a percent-encoded string.
///
/// Handles `%XX` sequences and `+` as space (as in `application/x-www-form-urlencoded`).
pub fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = bytes[i + 1] as char;
                let lo = bytes[i + 2] as char;
                if let (Some(h), Some(l)) = (hi.to_digit(16), lo.to_digit(16)) {
                    if let Ok(value) = u8::try_from(h * 16 + l) {
                        out.push(value);
                        i += 3;
                        continue;
                    }
                }
                out.push(bytes[i]);
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).to_string()
}

/// Parse URL query parameters into a `BTreeMap<String, String>`.
///
/// Keys and values are percent-decoded.
pub fn parse_query_params(input: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in input.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        out.insert(url_decode(key), url_decode(value));
    }
    out
}
