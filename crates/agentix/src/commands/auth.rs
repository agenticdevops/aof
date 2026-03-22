//! `agentix auth` — manage LLM subscription authentication.
//!
//! Commands:
//! - `agentix auth start <provider>`         # Initiate OAuth flow (opens browser)
//! - `agentix auth start <provider> --device-code`  # Headless device-code flow
//! - `agentix auth status`                   # Show auth status for all providers
//! - `agentix auth disconnect <provider>`    # Remove stored credentials

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use agentix_core::{
    AuthProfile, AuthProfileKind, AuthService, TokenSet,
    auth::{gemini_oauth, openai_oauth},
    auth::anthropic_token::detect_auth_kind,
    generate_pkce_state, normalize_provider,
};

// ---------------------------------------------------------------------------
// State directory
// ---------------------------------------------------------------------------

/// Returns `~/.agentix` as the default auth state directory.
pub fn default_state_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".agentix"))
        .unwrap_or_else(|| PathBuf::from(".agentix"))
}

/// Create an `AuthService` pointed at the default `~/.agentix` directory.
pub fn make_auth_service() -> AuthService {
    let state_dir = default_state_dir();
    AuthService::new(&state_dir, true)
}

// ---------------------------------------------------------------------------
// `agentix auth start <provider>`
// ---------------------------------------------------------------------------

/// Execute `agentix auth start <provider>`.
pub async fn start(provider: &str, device_code: bool) -> Result<()> {
    let normalized = normalize_provider(provider)?;

    match normalized.as_str() {
        "anthropic" => start_anthropic().await,
        "openai" => start_openai(device_code).await,
        "gemini" => start_gemini(device_code).await,
        other => anyhow::bail!(
            "Unknown provider '{}'. Use: anthropic, openai, gemini",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Anthropic: token paste
// ---------------------------------------------------------------------------

async fn start_anthropic() -> Result<()> {
    let service = make_auth_service();

    eprint!("Paste your Anthropic token (from `claude setup-token` or an API key): ");
    let token = read_secret_from_stdin()?;

    if token.is_empty() {
        anyhow::bail!("No token provided");
    }

    let kind = detect_auth_kind(&token, None);
    let mode = kind.as_metadata_value();

    let mut metadata = HashMap::new();
    metadata.insert("auth_kind".to_string(), mode.to_string());

    service
        .store_provider_token("anthropic", "default", &token, metadata, true)
        .await?;

    println!(
        "{} Anthropic authentication saved (mode: {})",
        "Success!".green().bold(),
        mode
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// OpenAI: OAuth PKCE or device code
// ---------------------------------------------------------------------------

async fn start_openai(device_code: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let service = make_auth_service();

    if device_code {
        return start_openai_device_code(&client, &service).await;
    }

    // PKCE browser flow
    let pkce = generate_pkce_state();
    let url = openai_oauth::build_authorize_url(&pkce);

    println!("Opening browser for OpenAI authentication...");
    open_browser(&url);
    println!("Waiting for callback at http://localhost:1455/auth/callback");
    println!("(Or pass --device-code for headless environments)");

    let timeout = Duration::from_secs(300); // 5 min
    let code = match openai_oauth::receive_loopback_code(&pkce.state, timeout).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Browser callback failed: {e}", "Error:".red());
            eprintln!("Hint: retry with --device-code for headless environments");
            return Err(e);
        }
    };

    let token_set = openai_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
    store_openai_tokens(&service, token_set).await
}

async fn start_openai_device_code(client: &reqwest::Client, service: &AuthService) -> Result<()> {
    println!("Starting OpenAI device code flow...");
    let device = openai_oauth::start_device_code_flow(client).await?;

    if let Some(msg) = &device.message {
        println!("{}", msg);
    } else {
        println!("Device code: {}", device.user_code.yellow().bold());
        println!("Visit: {}", device.verification_uri.cyan());
        if let Some(complete_url) = &device.verification_uri_complete {
            println!("Or open: {}", complete_url.cyan());
        }
    }
    println!("Waiting for authorization (expires in {}s)...", device.expires_in);

    let token_set = openai_oauth::poll_device_code_tokens(client, &device).await?;
    store_openai_tokens(service, token_set).await
}

async fn store_openai_tokens(service: &AuthService, token_set: TokenSet) -> Result<()> {
    let account_id = openai_oauth::extract_account_id_from_jwt(&token_set.access_token);

    let profile = service
        .store_openai_tokens("default", token_set, account_id.clone(), true)
        .await?;

    let account_display = profile
        .account_id
        .as_deref()
        .unwrap_or("-");

    println!(
        "{} OpenAI authentication successful! Account: {}",
        "Success!".green().bold(),
        account_display
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Gemini: OAuth PKCE or device code
// ---------------------------------------------------------------------------

async fn start_gemini(device_code: bool) -> Result<()> {
    // Check for required env vars before starting
    if gemini_oauth::gemini_oauth_client_id().is_none() {
        anyhow::bail!(
            "Set GEMINI_OAUTH_CLIENT_ID and GEMINI_OAUTH_CLIENT_SECRET environment variables\n\
             These are required for Gemini OAuth authentication."
        );
    }
    if gemini_oauth::gemini_oauth_client_secret().is_none() {
        anyhow::bail!(
            "Set GEMINI_OAUTH_CLIENT_SECRET environment variable\n\
             This is required for Gemini OAuth authentication."
        );
    }

    let client = reqwest::Client::new();
    let service = make_auth_service();

    if device_code {
        return start_gemini_device_code(&client, &service).await;
    }

    // PKCE browser flow
    let pkce = generate_pkce_state();
    let url = gemini_oauth::build_authorize_url(&pkce)?;

    println!("Opening browser for Gemini authentication...");
    open_browser(&url);
    println!("Waiting for callback at http://localhost:1456/auth/callback");

    let timeout = Duration::from_secs(300); // 5 min
    let code = match gemini_oauth::receive_loopback_code(&pkce.state, timeout).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Browser callback failed: {e}", "Error:".red());
            eprintln!("Hint: retry with --device-code for headless environments");
            return Err(e);
        }
    };

    let token_set = gemini_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
    store_gemini_tokens(&service, token_set).await
}

async fn start_gemini_device_code(client: &reqwest::Client, service: &AuthService) -> Result<()> {
    println!("Starting Gemini device code flow...");
    let device = gemini_oauth::start_device_code_flow(client).await?;

    println!("Device code: {}", device.user_code.yellow().bold());
    println!("Visit: {}", device.verification_uri.cyan());
    if let Some(complete_url) = &device.verification_uri_complete {
        println!("Or open: {}", complete_url.cyan());
    }
    println!("Waiting for authorization (expires in {}s)...", device.expires_in);

    let token_set = gemini_oauth::poll_device_code_tokens(client, &device).await?;
    store_gemini_tokens(service, token_set).await
}

async fn store_gemini_tokens(service: &AuthService, token_set: TokenSet) -> Result<()> {
    // Extract email from id_token if available
    let account_id = token_set
        .id_token
        .as_deref()
        .and_then(gemini_oauth::extract_account_email_from_id_token);

    let profile = service
        .store_gemini_tokens("default", token_set, account_id.clone(), true)
        .await?;

    let account_display = profile
        .account_id
        .as_deref()
        .unwrap_or("-");

    println!(
        "{} Gemini authentication successful! Account: {}",
        "Success!".green().bold(),
        account_display
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// `agentix auth status`
// ---------------------------------------------------------------------------

/// Execute `agentix auth status`.
pub async fn status() -> Result<()> {
    let service = make_auth_service();
    let profiles = service.load_profiles().await?;

    let providers = ["anthropic", "openai", "gemini"];

    // Collect status rows
    let rows: Vec<StatusRow> = providers
        .iter()
        .map(|&provider| {
            // Find active profile for this provider
            let profile = find_active_profile(&profiles.profiles, provider, &profiles.active_profiles);
            StatusRow::from_profile(provider, profile)
        })
        .collect();

    let any_connected = rows.iter().any(|r| r.connected);

    if !any_connected && rows.iter().all(|_| true) {
        // Check if there are zero profiles at all
        if profiles.profiles.is_empty() {
            println!(
                "No providers authenticated. Run `agentix auth start <provider>` to get started."
            );
            return Ok(());
        }
    }

    // Print table
    let col_provider = 10;
    let col_status = 16;
    let col_mode = 9;
    let col_account = 17;

    println!(
        "{:<col_provider$}  {:<col_status$}  {:<col_mode$}  {:<col_account$}  {}",
        "Provider",
        "Status",
        "Mode",
        "Account",
        "Expires",
        col_provider = col_provider,
        col_status = col_status,
        col_mode = col_mode,
        col_account = col_account,
    );
    println!(
        "{}",
        "-".repeat(col_provider + col_status + col_mode + col_account + 20)
    );

    for row in &rows {
        let status_display = if row.connected {
            "Connected".green().to_string()
        } else {
            "Not connected".dimmed().to_string()
        };

        println!(
            "{:<col_provider$}  {:<col_status$}  {:<col_mode$}  {:<col_account$}  {}",
            capitalize(row.provider),
            status_display,
            row.mode.as_deref().unwrap_or("-"),
            row.account.as_deref().unwrap_or("-"),
            row.expires.as_deref().unwrap_or("-"),
            col_provider = col_provider,
            col_status = col_status,
            col_mode = col_mode,
            col_account = col_account,
        );
    }

    Ok(())
}

struct StatusRow<'a> {
    provider: &'a str,
    connected: bool,
    mode: Option<String>,
    account: Option<String>,
    expires: Option<String>,
}

impl<'a> StatusRow<'a> {
    fn from_profile(provider: &'a str, profile: Option<&AuthProfile>) -> Self {
        let Some(p) = profile else {
            return Self { provider, connected: false, mode: None, account: None, expires: None };
        };

        let mode = match p.kind {
            AuthProfileKind::OAuth => Some("oauth".to_string()),
            AuthProfileKind::Token => Some("token".to_string()),
        };

        let expires = match &p.token_set {
            Some(ts) => {
                if let Some(exp) = ts.expires_at {
                    let now = chrono::Utc::now();
                    if exp <= now {
                        Some("Expired".red().to_string())
                    } else {
                        let remaining = exp - now;
                        let hours = remaining.num_hours();
                        let mins = remaining.num_minutes() % 60;
                        if hours > 0 {
                            Some(format!("{}h {}m", hours, mins))
                        } else {
                            Some(format!("{}m", mins))
                        }
                    }
                } else {
                    Some("Never".to_string())
                }
            }
            None => Some("Never".to_string()),
        };

        Self {
            provider,
            connected: true,
            mode,
            account: p.account_id.clone(),
            expires,
        }
    }
}

fn find_active_profile<'a>(
    profiles: &'a std::collections::BTreeMap<String, AuthProfile>,
    provider: &str,
    active_profiles: &std::collections::BTreeMap<String, String>,
) -> Option<&'a AuthProfile> {
    // 1. Check active profile
    if let Some(active_id) = active_profiles.get(provider) {
        if let Some(profile) = profiles.get(active_id) {
            return Some(profile);
        }
    }

    // 2. Try "default" profile for this provider
    let default_id = format!("{}:default", provider);
    if let Some(profile) = profiles.get(&default_id) {
        return Some(profile);
    }

    // 3. First profile for this provider
    profiles
        .values()
        .find(|p| p.provider == provider)
}

// ---------------------------------------------------------------------------
// `agentix auth disconnect <provider>`
// ---------------------------------------------------------------------------

/// Execute `agentix auth disconnect <provider>`.
pub async fn disconnect(provider: &str) -> Result<()> {
    let normalized = normalize_provider(provider)?;
    let service = make_auth_service();

    let removed = service.remove_profile(&normalized, "default").await?;

    if removed {
        println!("Disconnected from {}.", normalized);
    } else {
        println!("No {} credentials found.", normalized);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Browser open helper
// ---------------------------------------------------------------------------

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn();

    // If none of the above match, print the URL for manual opening
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    println!("Open this URL in your browser:\n{}", url);
}

// ---------------------------------------------------------------------------
// Stdin helper (reads without echo for tokens)
// ---------------------------------------------------------------------------

fn read_secret_from_stdin() -> Result<String> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

// ---------------------------------------------------------------------------
// Misc helpers
// ---------------------------------------------------------------------------

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
