//! Serve command - starts the AOF trigger webhook server
//!
//! This command starts a long-running HTTP server that accepts webhooks
//! from messaging platforms (Slack, Discord, Telegram, WhatsApp) and
//! routes them to configured agents.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use aof_runtime::RuntimeOrchestrator;
use aof_triggers::{
    TriggerHandler, TriggerHandlerConfig, TriggerServer, TriggerServerConfig,
    SlackPlatform, SlackConfig,
    DiscordPlatform, PlatformConfig,
    TelegramPlatform, TelegramConfig,
    WhatsAppPlatform, WhatsAppConfig,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

/// Server configuration loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeConfig {
    /// API version (aof.dev/v1)
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,

    /// Kind (DaemonConfig)
    pub kind: Option<String>,

    /// Metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,

    /// Specification
    pub spec: ServeSpec,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub name: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeSpec {
    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Platform configurations
    #[serde(default)]
    pub platforms: PlatformConfigs,

    /// Agent directory (for auto-discovery)
    #[serde(default)]
    pub agents: AgentDiscoveryConfig,

    /// Runtime settings
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Enable CORS
    #[serde(default = "default_true")]
    pub cors: bool,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            cors: true,
            timeout_secs: default_timeout(),
        }
    }
}

fn default_port() -> u16 {
    8080
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformConfigs {
    /// Slack configuration
    pub slack: Option<SlackPlatformConfig>,

    /// Discord configuration
    pub discord: Option<DiscordPlatformConfig>,

    /// Telegram configuration
    pub telegram: Option<TelegramPlatformConfig>,

    /// WhatsApp configuration
    pub whatsapp: Option<WhatsAppPlatformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackPlatformConfig {
    /// Enable this platform
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Bot token (or env var name with _env suffix)
    pub bot_token: Option<String>,
    pub bot_token_env: Option<String>,

    /// Signing secret (or env var name)
    pub signing_secret: Option<String>,
    pub signing_secret_env: Option<String>,

    /// App ID
    pub app_id: Option<String>,

    /// Bot user ID (for mention detection)
    pub bot_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordPlatformConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub bot_token: Option<String>,
    pub bot_token_env: Option<String>,

    pub application_id: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramPlatformConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub bot_token: Option<String>,
    pub bot_token_env: Option<String>,

    pub webhook_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppPlatformConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub access_token: Option<String>,
    pub access_token_env: Option<String>,

    pub verify_token: Option<String>,
    pub phone_number_id: Option<String>,
    pub app_secret: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDiscoveryConfig {
    /// Directory containing agent YAML files
    pub directory: Option<PathBuf>,

    /// Watch for changes and hot-reload
    #[serde(default)]
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum concurrent tasks
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,

    /// Task timeout in seconds
    #[serde(default = "default_task_timeout")]
    pub task_timeout_secs: u64,

    /// Max tasks per user
    #[serde(default = "default_max_per_user")]
    pub max_tasks_per_user: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: default_max_concurrent(),
            task_timeout_secs: default_task_timeout(),
            max_tasks_per_user: default_max_per_user(),
        }
    }
}

fn default_max_concurrent() -> usize {
    10
}

fn default_task_timeout() -> u64 {
    300
}

fn default_max_per_user() -> usize {
    3
}

/// Resolve a value that can come from config or environment variable
fn resolve_env_value(direct: Option<&str>, env_name: Option<&str>) -> Option<String> {
    // First try direct value
    if let Some(val) = direct {
        return Some(val.to_string());
    }

    // Then try environment variable
    if let Some(env_var) = env_name {
        if let Ok(val) = std::env::var(env_var) {
            return Some(val);
        }
    }

    None
}

/// Execute the serve command
pub async fn execute(
    config_file: Option<&str>,
    port: Option<u16>,
    host: Option<&str>,
    agents_dir: Option<&str>,
) -> anyhow::Result<()> {
    // Load configuration
    let config = if let Some(config_path) = config_file {
        info!("Loading configuration from: {}", config_path);
        let content = std::fs::read_to_string(config_path)?;
        serde_yaml::from_str::<ServeConfig>(&content)?
    } else {
        // Use defaults with CLI overrides
        ServeConfig {
            api_version: Some("aof.dev/v1".to_string()),
            kind: Some("DaemonConfig".to_string()),
            metadata: ConfigMetadata::default(),
            spec: ServeSpec {
                server: ServerConfig {
                    port: port.unwrap_or(default_port()),
                    host: host.unwrap_or("0.0.0.0").to_string(),
                    ..Default::default()
                },
                platforms: PlatformConfigs::default(),
                agents: AgentDiscoveryConfig {
                    directory: agents_dir.map(PathBuf::from),
                    watch: false,
                },
                runtime: RuntimeConfig::default(),
            },
        }
    };

    // Apply CLI overrides
    let server_port = port.unwrap_or(config.spec.server.port);
    let server_host = host.unwrap_or(&config.spec.server.host);

    let bind_addr: SocketAddr = format!("{}:{}", server_host, server_port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

    info!("Starting AOF Trigger Server");
    info!("  Bind address: {}", bind_addr);

    // Create runtime orchestrator
    let orchestrator = Arc::new(
        RuntimeOrchestrator::with_max_concurrent(config.spec.runtime.max_concurrent_tasks)
    );

    // Create handler config
    let handler_config = TriggerHandlerConfig {
        verbose: true,
        auto_ack: true,
        max_tasks_per_user: config.spec.runtime.max_tasks_per_user,
        command_timeout_secs: config.spec.runtime.task_timeout_secs,
    };

    // Create trigger handler
    let mut handler = TriggerHandler::with_config(orchestrator, handler_config);

    // Register platforms
    let mut platforms_registered = 0;

    // Slack
    if let Some(slack_config) = &config.spec.platforms.slack {
        if slack_config.enabled {
            let bot_token = resolve_env_value(
                slack_config.bot_token.as_deref(),
                slack_config.bot_token_env.as_deref(),
            );
            let signing_secret = resolve_env_value(
                slack_config.signing_secret.as_deref(),
                slack_config.signing_secret_env.as_deref(),
            );

            if let (Some(token), Some(secret)) = (bot_token, signing_secret) {
                let platform_config = SlackConfig {
                    bot_token: token,
                    signing_secret: secret,
                    app_id: slack_config.app_id.clone().unwrap_or_default(),
                    bot_user_id: slack_config.bot_user_id.clone().unwrap_or_default(),
                    bot_name: "aofbot".to_string(),
                    allowed_workspaces: None,
                    allowed_channels: None,
                };
                match SlackPlatform::new(platform_config) {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        info!("  Registered platform: slack");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        warn!("  Failed to create Slack platform: {}", e);
                    }
                }
            } else {
                warn!("  Slack enabled but missing bot_token or signing_secret");
            }
        }
    }

    // Discord
    if let Some(discord_config) = &config.spec.platforms.discord {
        if discord_config.enabled {
            let bot_token = resolve_env_value(
                discord_config.bot_token.as_deref(),
                discord_config.bot_token_env.as_deref(),
            );

            if let Some(token) = bot_token {
                let platform_config = PlatformConfig {
                    platform: "discord".to_string(),
                    api_token: Some(token),
                    webhook_secret: discord_config.public_key.clone(),
                    webhook_url: None,
                };
                let platform = Arc::new(DiscordPlatform::new(platform_config));
                handler.register_platform(platform);
                info!("  Registered platform: discord");
                platforms_registered += 1;
            } else {
                warn!("  Discord enabled but missing bot_token");
            }
        }
    }

    // Telegram
    if let Some(telegram_config) = &config.spec.platforms.telegram {
        if telegram_config.enabled {
            let bot_token = resolve_env_value(
                telegram_config.bot_token.as_deref(),
                telegram_config.bot_token_env.as_deref(),
            );

            if let Some(token) = bot_token {
                let platform_config = TelegramConfig {
                    bot_token: token,
                    webhook_url: None,
                    webhook_secret: telegram_config.webhook_secret.clone(),
                    bot_name: "aofbot".to_string(),
                    allowed_users: None,
                    allowed_groups: None,
                };
                match TelegramPlatform::new(platform_config) {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        info!("  Registered platform: telegram");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        warn!("  Failed to create Telegram platform: {}", e);
                    }
                }
            } else {
                warn!("  Telegram enabled but missing bot_token");
            }
        }
    }

    // WhatsApp
    if let Some(whatsapp_config) = &config.spec.platforms.whatsapp {
        if whatsapp_config.enabled {
            let access_token = resolve_env_value(
                whatsapp_config.access_token.as_deref(),
                whatsapp_config.access_token_env.as_deref(),
            );

            if let Some(token) = access_token {
                let platform_config = WhatsAppConfig {
                    access_token: token,
                    verify_token: whatsapp_config.verify_token.clone().unwrap_or_default(),
                    phone_number_id: whatsapp_config.phone_number_id.clone().unwrap_or_default(),
                    app_secret: whatsapp_config.app_secret.clone().unwrap_or_default(),
                    business_account_id: None,
                    allowed_numbers: None,
                    api_version: "v18.0".to_string(),
                };
                match WhatsAppPlatform::new(platform_config) {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        info!("  Registered platform: whatsapp");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        warn!("  Failed to create WhatsApp platform: {}", e);
                    }
                }
            } else {
                warn!("  WhatsApp enabled but missing access_token");
            }
        }
    }

    if platforms_registered == 0 {
        warn!("No platforms registered! Server will start but won't process any webhooks.");
        warn!("Configure platforms in your config file or set environment variables.");
    }

    // Create server config
    let server_config = TriggerServerConfig {
        bind_addr,
        enable_cors: config.spec.server.cors,
        timeout_secs: config.spec.server.timeout_secs,
        max_body_size: 10 * 1024 * 1024, // 10MB
    };

    // Create and start server
    let server = TriggerServer::with_config(Arc::new(handler), server_config);

    info!("Server starting...");
    info!("  Health check: http://{}/health", bind_addr);
    info!("  Webhook endpoint: http://{}/webhook/{{platform}}", bind_addr);
    info!("Press Ctrl+C to stop");

    // Handle graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Shutdown signal received, stopping server...");
    };

    tokio::select! {
        result = server.serve() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                return Err(anyhow::anyhow!("Server error: {}", e));
            }
        }
        _ = shutdown_signal => {
            info!("Server stopped gracefully");
        }
    }

    Ok(())
}
