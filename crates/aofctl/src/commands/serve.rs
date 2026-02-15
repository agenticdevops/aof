//! Serve command - starts the AOF trigger webhook server
//!
//! This command starts a long-running HTTP server that accepts webhooks
//! from messaging platforms (Slack, Discord, Telegram, WhatsApp) and
//! routes them to configured agents.
//!
//! # Performance Profiling with tokio-console
//!
//! To profile async runtime performance using tokio-console:
//!
//! 1. Build with tokio-console feature and tokio_unstable flag:
//!    ```bash
//!    RUSTFLAGS="--cfg tokio_unstable" cargo build --features tokio-console
//!    ```
//!
//! 2. Run aofctl serve:
//!    ```bash
//!    RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio-console -- serve
//!    ```
//!
//! 3. In another terminal, launch tokio-console:
//!    ```bash
//!    tokio-console
//!    ```
//!
//! The console will connect to the running daemon and display:
//! - Task list with CPU/poll time
//! - Async resource usage (channels, mutexes)
//! - Task durations and wait times
//! - Waker churn and blocking detection
//!
//! Note: tokio-console adds overhead (~10-15%) and should only be used for profiling,
//! not in production deployments.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{CorsLayer, Any};
use aof_coordination::{EventBroadcaster, SessionPersistence, SessionState, AgentState};
use aof_coordination_protocols::{CoordinationManager, CoordinationConfig, CoordinationMode};
use aof_core::{TriggerRegistry, Registry, StandaloneTriggerType};
use aof_runtime::{Runtime, RuntimeOrchestrator};

// No-op model for when no API key is configured
struct NoOpModel {
    config: aof_core::ModelConfig,
}

impl NoOpModel {
    fn new() -> Self {
        Self {
            config: aof_core::ModelConfig {
                provider: aof_core::ModelProvider::Anthropic,
                model: "noop".to_string(),
                api_key: None,
                endpoint: None,
                max_tokens: Some(0),
                temperature: 0.0,
                timeout_secs: 60,
                headers: std::collections::HashMap::new(),
                extra: std::collections::HashMap::new(),
            }
        }
    }
}

#[async_trait::async_trait]
impl aof_core::Model for NoOpModel {
    async fn generate(&self, _request: &aof_core::ModelRequest) -> aof_core::AofResult<aof_core::ModelResponse> {
        Err(aof_core::AofError::config("No LLM API key configured. Set ANTHROPIC_API_KEY environment variable."))
    }

    async fn generate_stream(
        &self,
        _request: &aof_core::ModelRequest,
    ) -> aof_core::AofResult<std::pin::Pin<Box<dyn futures::Stream<Item = aof_core::AofResult<aof_core::StreamChunk>> + Send>>> {
        Err(aof_core::AofError::config("No LLM API key configured. Set ANTHROPIC_API_KEY environment variable."))
    }

    fn provider(&self) -> aof_core::ModelProvider {
        aof_core::ModelProvider::Anthropic
    }

    fn config(&self) -> &aof_core::ModelConfig {
        &self.config
    }
}
use aof_triggers::{
    TriggerHandler, TriggerHandlerConfig, TriggerServer, TriggerServerConfig,
    SlackPlatform, SlackConfig,
    DiscordPlatform, PlatformConfig,
    TelegramPlatform, TelegramConfig,
    WhatsAppPlatform, WhatsAppConfig,
    GitHubPlatform, GitHubConfig,
    JiraPlatform, JiraConfig,
    CommandBinding as HandlerCommandBinding,
    flow::{FlowRegistry, FlowRouter},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Config API
use crate::api::config::{ConfigState, get_agents_config, get_tools_config, get_config_version};
// Metrics API
use crate::api::metrics::{MetricsState, get_agent_metrics};

// Additional imports for inline handlers
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};

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

    /// Flows directory (for AgentFlow-based routing)
    #[serde(default)]
    pub flows: FlowsConfig,

    /// Triggers directory (for loading Trigger resources)
    #[serde(default)]
    pub triggers: TriggersConfig,

    /// Runtime settings
    #[serde(default)]
    pub runtime: RuntimeConfig,

    /// Decision logging settings
    #[serde(default)]
    pub decision_log: DecisionLogConfig,

    /// Coordination protocol settings
    #[serde(default)]
    pub coordination: Option<CoordinationServeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLogConfig {
    /// Enable decision logging
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Path to decision log file (default: ~/.aof/decisions.jsonl)
    pub path: Option<PathBuf>,
}

impl Default for DecisionLogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoordinationServeConfig {
    /// Enable coordination protocols
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Global coordination mode ("full", "standard", "reduced", "heartbeat_only", "disabled")
    #[serde(default)]
    pub mode: Option<String>,

    /// Heartbeat protocol settings
    #[serde(default)]
    pub heartbeat: Option<HeartbeatServeConfig>,

    /// Token limits and auto-degradation settings
    #[serde(default)]
    pub token_limits: Option<TokenLimitsServeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLimitsServeConfig {
    /// Maximum coordination overhead before degradation (default: 30%)
    #[serde(default = "default_30_percent")]
    pub max_overhead_percent: f64,

    /// Enable automatic degradation (default: true)
    #[serde(default = "default_true")]
    pub auto_degrade: bool,

    /// Recovery threshold for hysteresis (default: 20%)
    #[serde(default = "default_20_percent")]
    pub recovery_threshold: f64,
}

impl Default for TokenLimitsServeConfig {
    fn default() -> Self {
        Self {
            max_overhead_percent: default_30_percent(),
            auto_degrade: true,
            recovery_threshold: default_20_percent(),
        }
    }
}

fn default_30_percent() -> f64 {
    30.0
}

fn default_20_percent() -> f64 {
    20.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatServeConfig {
    /// Heartbeat frequency in seconds (default: 60)
    #[serde(default = "default_heartbeat_frequency")]
    pub frequency_secs: u64,

    /// Heartbeat timeout in seconds (default: 120)
    #[serde(default = "default_heartbeat_timeout")]
    pub timeout_secs: u64,
}

impl Default for HeartbeatServeConfig {
    fn default() -> Self {
        Self {
            frequency_secs: default_heartbeat_frequency(),
            timeout_secs: default_heartbeat_timeout(),
        }
    }
}

fn default_heartbeat_frequency() -> u64 {
    60 // 60 seconds (updated from 30s in plan)
}

fn default_heartbeat_timeout() -> u64 {
    120 // 120 seconds (2x interval)
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

    /// GitHub configuration
    pub github: Option<GitHubPlatformConfig>,

    /// Jira configuration
    pub jira: Option<JiraPlatformConfig>,
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

    /// User IDs allowed to approve destructive commands
    /// If not specified, anyone can approve
    /// Supports platform-agnostic IDs with prefixes:
    /// - Raw Slack user IDs: "U12345678"
    /// - Prefixed format: "slack:U12345678"
    /// - Email format (future): "email:user@company.com"
    #[serde(default)]
    pub approval_allowed_users: Option<Vec<String>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPlatformConfig {
    /// Enable this platform
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// GitHub token (or env var name with _env suffix)
    pub token: Option<String>,
    pub token_env: Option<String>,

    /// Webhook secret (or env var name)
    pub webhook_secret: Option<String>,
    pub webhook_secret_env: Option<String>,

    /// Bot/App name for identification
    pub bot_name: Option<String>,

    /// Allowed repository filter (optional whitelist)
    /// Format: ["owner/repo", "owner/*", "*"]
    #[serde(default)]
    pub allowed_repos: Option<Vec<String>>,

    /// Allowed GitHub organizations (optional whitelist)
    #[serde(default)]
    pub allowed_orgs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPlatformConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Jira Cloud ID (for cloud instances)
    pub cloud_id: Option<String>,
    pub cloud_id_env: Option<String>,

    /// Base URL (e.g., https://your-domain.atlassian.net)
    pub base_url: Option<String>,

    /// User email for API authentication
    pub user_email: Option<String>,
    pub user_email_env: Option<String>,

    /// API token for authentication
    pub api_token: Option<String>,
    pub api_token_env: Option<String>,

    /// Webhook secret for signature verification
    pub webhook_secret: Option<String>,
    pub webhook_secret_env: Option<String>,

    /// Bot name for identification in comments
    #[serde(default)]
    pub bot_name: Option<String>,

    /// Allowed project keys (whitelist)
    #[serde(default)]
    pub allowed_projects: Option<Vec<String>>,

    /// Allowed event types (whitelist)
    #[serde(default)]
    pub allowed_events: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDiscoveryConfig {
    /// Directory containing agent YAML files
    pub directory: Option<PathBuf>,

    /// Watch for changes and hot-reload
    #[serde(default)]
    pub watch: bool,
}

/// Flows configuration for AgentFlow-based routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowsConfig {
    /// Directory containing AgentFlow YAML files
    pub directory: Option<PathBuf>,

    /// Watch for changes and hot-reload
    #[serde(default)]
    pub watch: bool,

    /// Enable flow-based routing (takes priority over default_agent)
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Triggers configuration for loading Trigger resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggersConfig {
    /// Directory containing Trigger YAML files
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

    /// Default agent for natural language messages (non-slash-command)
    pub default_agent: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: default_max_concurrent(),
            task_timeout_secs: default_task_timeout(),
            max_tasks_per_user: default_max_per_user(),
            default_agent: None,
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

/// Create a channel adapter from configuration
fn create_adapter_from_config(
    config: &aof_gateway::config::AdapterConfig,
) -> Result<Box<dyn aof_gateway::ChannelAdapter>, aof_core::AofError> {
    use aof_gateway::Platform;

    match config.platform {
        Platform::Slack => {
            // For now, create minimal mock adapter - full implementation in 03-02 already exists
            // This is a placeholder until we export the adapter types properly
            Err(aof_core::AofError::config(
                "Slack adapter integration coming in final integration test".to_string()
            ))
        }
        Platform::Discord => {
            Err(aof_core::AofError::config(
                "Discord adapter integration coming in final integration test".to_string()
            ))
        }
        Platform::Telegram => {
            Err(aof_core::AofError::config(
                "Telegram adapter integration coming in final integration test".to_string()
            ))
        }
        _ => Err(aof_core::AofError::config(format!(
            "Unsupported platform: {:?}",
            config.platform
        ))),
    }
}

/// Execute the serve command
pub async fn execute(
    config_file: Option<&str>,
    port: Option<u16>,
    host: Option<&str>,
    agents_dir: Option<&str>,
    flows_dir: Option<&str>,
    triggers_dir: Option<&str>,
    gateway_config_file: Option<&str>,
    debug_gateway: bool,
    validate_config_only: bool,
    static_dir: Option<&str>,
    workspace_root: Option<&str>,
) -> anyhow::Result<()> {
    // Handle --validate-config flag
    if validate_config_only {
        if let Some(gw_config_path) = gateway_config_file {
            let config = aof_gateway::config::load_gateway_config(gw_config_path)?;
            println!("✓ Gateway config is valid");
            println!("  Adapters: {}", config.spec.adapters.len());
            println!("  Squads: {}", config.spec.squads.len());
            return Ok(());
        } else {
            anyhow::bail!("--validate-config requires --gateway-config");
        }
    }

    // Enable debug logging for gateway if requested
    if debug_gateway {
        std::env::set_var("RUST_LOG", "aof_gateway=debug");
    }
    // Load configuration
    let config = if let Some(config_path) = config_file {
        println!("Loading configuration from: {}", config_path);
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
                flows: FlowsConfig {
                    directory: flows_dir.map(PathBuf::from),
                    watch: false,
                    enabled: true,
                },
                triggers: TriggersConfig {
                    directory: triggers_dir.map(PathBuf::from),
                    watch: false,
                },
                runtime: RuntimeConfig::default(),
                decision_log: DecisionLogConfig::default(),
                coordination: None,
            },
        }
    };

    // Apply CLI overrides
    let server_port = port.unwrap_or(config.spec.server.port);
    let server_host = host.unwrap_or(&config.spec.server.host);

    let bind_addr: SocketAddr = format!("{}:{}", server_host, server_port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

    println!("Starting AOF Trigger Server");
    println!("  Bind address: {}", bind_addr);

    // Create event broadcaster for real-time event streaming
    let event_bus = Arc::new(EventBroadcaster::new(1000)); // 1000 event buffer
    println!("  Event bus: initialized (buffer: 1000)");

    // Create reliability metrics cache and subscribe to event bus
    let metrics_cache = Arc::new(aof_personas::ReliabilityCache::default_capacity());
    {
        let cache = Arc::clone(&metrics_cache);
        let mut event_rx = event_bus.subscribe();
        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = cache.update_with_event(&event).await {
                            tracing::warn!("Failed to update metrics cache: {}", e);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Metrics cache lagged, dropped {} events", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Event bus closed, stopping metrics cache updates");
                        break;
                    }
                }
            }
        });
    }
    println!("  Metrics cache: initialized (max 10000 events)");

    // Initialize coordination manager if enabled in config
    let coordination_manager = if let Some(coord_config) = &config.spec.coordination {
        if coord_config.enabled {
            // Parse coordination mode (default to Full if not specified)
            let mode = match coord_config.mode.as_deref() {
                Some("full") => CoordinationMode::Full,
                Some("standard") => CoordinationMode::Standard,
                Some("reduced") => CoordinationMode::Reduced,
                Some("heartbeat_only") => CoordinationMode::HeartbeatOnly,
                Some("disabled") => CoordinationMode::Disabled,
                _ => CoordinationMode::Full,
            };

            // Create heartbeat config
            let heartbeat_config = if let Some(hb_config) = &coord_config.heartbeat {
                aof_coordination_protocols::HeartbeatConfig {
                    frequency: std::time::Duration::from_secs(hb_config.frequency_secs),
                    timeout: std::time::Duration::from_secs(hb_config.timeout_secs),
                    enabled: true,
                }
            } else {
                aof_coordination_protocols::HeartbeatConfig::default()
            };

            // Parse token limits configuration
            let token_limits = if let Some(token_config) = &coord_config.token_limits {
                aof_coordination_protocols::DegradationConfig {
                    max_overhead_percent: token_config.max_overhead_percent,
                    auto_degrade: token_config.auto_degrade,
                    check_interval: std::time::Duration::from_secs(60),
                    recovery_threshold: token_config.recovery_threshold,
                }
            } else {
                aof_coordination_protocols::DegradationConfig::default()
            };

            // Create coordination config
            let coordination_config = CoordinationConfig {
                enabled: true,
                mode,
                heartbeat: heartbeat_config,
                token_limits,
                standup: aof_coordination_protocols::StandupConfig::default(),
            };

            // Generate session ID
            let session_id = uuid::Uuid::new_v4().to_string();

            // Create a dedicated broadcast channel for coordination events
            // The coordination manager will emit to this, and we'll forward to main event bus
            let (coord_event_tx, mut coord_event_rx) = tokio::sync::broadcast::channel(1000);

            // Create coordination manager
            let manager = Arc::new(CoordinationManager::new(
                coordination_config,
                coord_event_tx,
                session_id,
            ));

            // Forward coordination events to main event bus
            let event_bus_clone = Arc::clone(&event_bus);
            tokio::spawn(async move {
                loop {
                    match coord_event_rx.recv().await {
                        Ok(event) => {
                            event_bus_clone.emit(event);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Coordination event forwarder lagged, dropped {} events", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Coordination event channel closed, stopping forwarder");
                            break;
                        }
                    }
                }
            });

            println!("  Coordination: enabled (mode: {:?}, heartbeat: {}s/{}s)",
                mode,
                coord_config.heartbeat.as_ref().map(|c| c.frequency_secs).unwrap_or(60),
                coord_config.heartbeat.as_ref().map(|c| c.timeout_secs).unwrap_or(120)
            );

            // TODO: Register discovered agents with coordination manager
            // This will happen after agent discovery below

            Some(manager)
        } else {
            println!("  Coordination: disabled in config");
            None
        }
    } else {
        println!("  Coordination: not configured (disabled)");
        None
    };

    // Initialize gateway if config provided
    let gateway_handle = if let Some(gw_config_path) = gateway_config_file {
        tracing::info!("Loading gateway config from: {}", gw_config_path);

        let gw_config = aof_gateway::config::load_gateway_config(gw_config_path)?;

        tracing::info!(
            adapters = gw_config.spec.adapters.len(),
            squads = gw_config.spec.squads.len(),
            "Gateway config loaded"
        );

        // Create gateway hub with shutdown signal
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
        let mut hub = aof_gateway::GatewayHub::new(event_tx, shutdown_rx);
        hub.set_config(gw_config.clone());

        // Register adapters from config
        for adapter_config in &gw_config.spec.adapters {
            if !adapter_config.enabled {
                continue;
            }

            // Create adapter based on platform
            match create_adapter_from_config(adapter_config) {
                Ok(adapter) => {
                    let adapter_id = adapter.adapter_id().to_string();
                    hub.register_adapter(adapter);
                    tracing::info!("Registered gateway adapter: {}", adapter_id);
                }
                Err(e) => {
                    tracing::error!("Failed to create adapter for {:?}: {}", adapter_config.platform, e);
                }
            }
        }

        // Start gateway hub
        hub.start().await?;

        // Spawn gateway run loop
        let hub_handle = tokio::spawn(async move {
            if let Err(e) = hub.run().await {
                tracing::error!("Gateway hub error: {}", e);
            }
        });

        println!("  Gateway: initialized ({} adapters)", gw_config.spec.adapters.iter().filter(|a| a.enabled).count());

        Some((hub_handle, shutdown_tx))
    } else {
        None
    };

    // Create session persistence
    let persist_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aof")
        .join("sessions");
    tokio::fs::create_dir_all(&persist_dir).await?;
    let session_persistence = SessionPersistence::new(persist_dir.clone()).await?;

    // Create decision logger for agent decision tracking
    let decision_logger = if config.spec.decision_log.enabled {
        let decision_log_path = config.spec.decision_log.path.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("aof")
                .join("decisions.jsonl")
        });

        // Ensure parent directory exists
        if let Some(parent) = decision_log_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let logger = Arc::new(aof_coordination::DecisionLogger::new(
            decision_log_path.clone(),
            event_bus.clone(),
        ));
        println!("  Decision logger: enabled at {}", decision_log_path.display());
        Some(logger)
    } else {
        println!("  Decision logger: disabled");
        None
    };

    // Generate session ID (UUID v4, unique per daemon lifetime)
    let session_id = uuid::Uuid::new_v4().to_string();
    println!("  Session ID: {}", session_id);

    // Restore previous session if exists (for debugging/continuity)
    // In Phase 1, just log if previous session exists
    if let Ok(sessions) = session_persistence.list_sessions().await {
        if !sessions.is_empty() {
            println!("  Found {} previous session(s)", sessions.len());
        }
    }

    // Determine workspace root early (used for introductions and config API)
    let workspace_path = workspace_root
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    println!("  Workspace root: {}", workspace_path.display());

    // ========================================================================
    // Agent Introduction Events
    // ========================================================================
    // Load workspace persona files (AGENTS.md, SOUL.md) and emit introduction
    // events via the broadcast channel. These events flow to WebSocket clients
    // and messaging gateways. Introduction happens once per cold start.
    {
        let agents_md_path = workspace_path.join("AGENTS.md");
        let soul_md_path = workspace_path.join("SOUL.md");

        if agents_md_path.exists() {
            match aof_personas::AgentLoader::load_from_file(
                agents_md_path.to_str().unwrap_or("workspace/AGENTS.md"),
            )
            .await
            {
                Ok(agents) => {
                    let souls = aof_personas::SoulLoader::load_from_file(
                        soul_md_path.to_str().unwrap_or("workspace/SOUL.md"),
                    )
                    .await
                    .unwrap_or_default();

                    // Load optional squads.yaml for introduction overrides
                    let squads_path = workspace_path.join("squads.yaml");
                    let squad_overrides = load_squad_overrides(&squads_path).await;

                    let intro_events = aof_personas::build_introduction_event_batch(
                        &agents, &souls, &session_id,
                    );

                    tracing::info!(
                        agent_count = agents.len(),
                        "Emitting introduction events for {} agents",
                        agents.len()
                    );
                    println!(
                        "  Introductions: emitting for {} agents",
                        agents.len()
                    );

                    for mut event in intro_events {
                        // Apply squad override if present
                        if let Some(ref overrides) = squad_overrides {
                            if let Some(intro) = event.introduction.as_mut() {
                                if let Some(override_msg) = overrides.get(&intro.agent_id) {
                                    tracing::debug!(
                                        agent_id = %intro.agent_id,
                                        "Applying squad intro override"
                                    );
                                    intro.intro_message = override_msg.clone();
                                }
                            }
                        }

                        if let Some(ref intro) = event.introduction {
                            tracing::debug!(
                                agent = %intro.agent_name,
                                message = %intro.intro_message,
                                "Agent {} introduced",
                                intro.agent_name
                            );
                        }
                        event_bus.emit(event);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to load AGENTS.md for introductions, skipping"
                    );
                    println!(
                        "  Introductions: skipped (AGENTS.md load error: {})",
                        e
                    );
                }
            }
        } else {
            tracing::debug!("No AGENTS.md found at {:?}, skipping introductions", agents_md_path);
            println!("  Introductions: skipped (no AGENTS.md)");
        }
    }

    // Create runtime orchestrator
    let orchestrator = Arc::new(
        RuntimeOrchestrator::with_max_concurrent(config.spec.runtime.max_concurrent_tasks)
    );

    // Create handler config
    let handler_config = TriggerHandlerConfig {
        verbose: true,
        auto_ack: false, // Don't auto-ack for natural language - we handle it ourselves
        max_tasks_per_user: config.spec.runtime.max_tasks_per_user,
        command_timeout_secs: config.spec.runtime.task_timeout_secs,
        default_agent: config.spec.runtime.default_agent.clone(),
        command_bindings: std::collections::HashMap::new(), // Loaded from Trigger CRDs
        max_message_age_secs: 60, // Drop messages older than 1 minute (handles queued messages)
    };

    if let Some(ref agent) = config.spec.runtime.default_agent {
        println!("  Default agent for natural language: {}", agent);
    }

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
                    approval_allowed_users: slack_config.approval_allowed_users.clone(),
                    approval_allowed_roles: None,  // For future RBAC support
                };
                // Use async constructor to auto-detect bot_user_id from Slack API
                // This is critical for preventing self-approval of destructive commands
                match SlackPlatform::new_with_auto_detection(platform_config).await {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        println!("  Registered platform: slack");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        eprintln!("  Failed to create Slack platform: {}", e);
                    }
                }
            } else {
                eprintln!("  Slack enabled but missing bot_token or signing_secret");
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
                println!("  Registered platform: discord");
                platforms_registered += 1;
            } else {
                eprintln!("  Discord enabled but missing bot_token");
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
                        println!("  Registered platform: telegram");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        eprintln!("  Failed to create Telegram platform: {}", e);
                    }
                }
            } else {
                eprintln!("  Telegram enabled but missing bot_token");
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
                        println!("  Registered platform: whatsapp");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        eprintln!("  Failed to create WhatsApp platform: {}", e);
                    }
                }
            } else {
                eprintln!("  WhatsApp enabled but missing access_token");
            }
        }
    }

    // GitHub
    if let Some(github_config) = &config.spec.platforms.github {
        if github_config.enabled {
            let token = resolve_env_value(
                github_config.token.as_deref(),
                github_config.token_env.as_deref(),
            );
            let webhook_secret = resolve_env_value(
                github_config.webhook_secret.as_deref(),
                github_config.webhook_secret_env.as_deref(),
            );

            if let Some(secret) = webhook_secret {
                let platform_config = GitHubConfig {
                    token: token.unwrap_or_default(), // Token is optional, webhook_secret is required
                    webhook_secret: secret,
                    bot_name: github_config.bot_name.clone().unwrap_or_else(|| "aofbot".to_string()),
                    api_url: "https://api.github.com".to_string(),
                    allowed_repos: github_config.allowed_repos.clone(),
                    allowed_events: None,
                    allowed_users: None,
                    auto_approve_patterns: None,
                    enable_status_checks: true,
                    enable_reviews: true,
                    enable_comments: true,
                };

                if platform_config.token.is_empty() {
                    eprintln!("  GitHub: GITHUB_TOKEN not set, API features (posting comments) disabled");
                }

                match GitHubPlatform::new(platform_config) {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        println!("  Registered platform: github");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        eprintln!("  Failed to create GitHub platform: {}", e);
                    }
                }
            } else {
                eprintln!("  GitHub enabled but missing webhook_secret");
            }
        }
    }

    // Jira
    if let Some(jira_config) = &config.spec.platforms.jira {
        if jira_config.enabled {
            let api_token = resolve_env_value(
                jira_config.api_token.as_deref(),
                jira_config.api_token_env.as_deref(),
            );
            let user_email = resolve_env_value(
                jira_config.user_email.as_deref(),
                jira_config.user_email_env.as_deref(),
            );
            let webhook_secret = resolve_env_value(
                jira_config.webhook_secret.as_deref(),
                jira_config.webhook_secret_env.as_deref(),
            );

            // Build base URL from cloud_id or use provided base_url
            let base_url = if let Some(ref url) = jira_config.base_url {
                Some(url.clone())
            } else {
                let cloud_id = resolve_env_value(
                    jira_config.cloud_id.as_deref(),
                    jira_config.cloud_id_env.as_deref(),
                );
                cloud_id.map(|id| format!("https://api.atlassian.com/ex/jira/{}", id))
            };

            if let (Some(token), Some(email), Some(secret), Some(url)) =
                (api_token, user_email, webhook_secret, base_url)
            {
                let platform_config = JiraConfig {
                    base_url: url,
                    email,
                    api_token: token,
                    webhook_secret: secret,
                    bot_name: jira_config.bot_name.clone().unwrap_or_else(|| "aofbot".to_string()),
                    allowed_projects: jira_config.allowed_projects.clone(),
                    allowed_events: jira_config.allowed_events.clone(),
                    allowed_users: None,
                    enable_comments: true,
                    enable_updates: true,
                    enable_transitions: true,
                };
                match JiraPlatform::new(platform_config) {
                    Ok(platform) => {
                        handler.register_platform(Arc::new(platform));
                        println!("  Registered platform: jira");
                        platforms_registered += 1;
                    }
                    Err(e) => {
                        eprintln!("  Failed to create Jira platform: {}", e);
                    }
                }
            } else {
                eprintln!("  Jira enabled but missing required config (api_token, user_email, webhook_secret, and base_url or cloud_id)");
            }
        }
    }

    // Load Triggers from directory
    let triggers_dir_path = triggers_dir
        .map(PathBuf::from)
        .or_else(|| config.spec.triggers.directory.clone());

    if let Some(ref triggers_path) = triggers_dir_path {
        println!("Loading Triggers from: {}", triggers_path.display());
        let mut trigger_registry = TriggerRegistry::new();

        match trigger_registry.load_directory(triggers_path) {
            Ok(count) => {
                if count > 0 {
                    println!("  Loaded {} triggers: {:?}", count, trigger_registry.names());

                    // Register platforms for each trigger type
                    for trigger in trigger_registry.get_all() {
                        match trigger.spec.trigger_type {
                            StandaloneTriggerType::GitHub => {
                                // Register GitHub platform if we have a trigger for it
                                if let Some(ref secret) = trigger.spec.config.webhook_secret {
                                    // Get GitHub token from env or trigger config
                                    let token = std::env::var("GITHUB_TOKEN")
                                        .or_else(|_| std::env::var("GH_TOKEN"))
                                        .unwrap_or_default();

                                    if token.is_empty() {
                                        eprintln!("  GitHub trigger '{}': GITHUB_TOKEN not set, API features disabled", trigger.name());
                                    }

                                    let github_config = GitHubConfig {
                                        token,
                                        webhook_secret: secret.clone(),
                                        bot_name: "aof-bot".to_string(),
                                        api_url: "https://api.github.com".to_string(),
                                        allowed_repos: None,
                                        allowed_events: None,
                                        allowed_users: None,
                                        auto_approve_patterns: None,
                                        enable_status_checks: true,
                                        enable_reviews: true,
                                        enable_comments: true,
                                    };
                                    match GitHubPlatform::new(github_config) {
                                        Ok(platform) => {
                                            handler.register_platform(Arc::new(platform));
                                            println!("  Registered platform: github (from trigger '{}')", trigger.name());
                                            platforms_registered += 1;
                                        }
                                        Err(e) => {
                                            eprintln!("  Failed to create GitHub platform: {}", e);
                                        }
                                    }
                                } else {
                                    eprintln!("  GitHub trigger '{}' missing webhook_secret", trigger.name());
                                }
                            }
                            // Other trigger types use platforms registered from config
                            _ => {}
                        }

                        // Add command bindings from trigger
                        for (cmd, binding) in &trigger.spec.commands {
                            // Convert core CommandBinding to handler CommandBinding
                            let handler_binding = HandlerCommandBinding {
                                agent: binding.agent.clone(),
                                fleet: binding.fleet.clone(),
                                flow: binding.flow.clone(),
                                description: binding.description.clone(),
                            };

                            // Strip leading slash if present for consistent lookup
                            let cmd_name = cmd.trim_start_matches('/').to_string();
                            handler.register_command_binding(cmd_name.clone(), handler_binding);

                            if let Some(ref agent) = binding.agent {
                                println!("  Registered command '{}' -> agent '{}'", cmd, agent);
                            } else if let Some(ref fleet) = binding.fleet {
                                println!("  Registered command '{}' -> fleet '{}'", cmd, fleet);
                            } else if let Some(ref flow) = binding.flow {
                                println!("  Registered command '{}' -> flow '{}'", cmd, flow);
                            }
                        }

                        // Set default agent if specified in trigger
                        if let Some(ref default_agent) = trigger.spec.default_agent {
                            println!("  Default agent for trigger '{}': {}", trigger.name(), default_agent);
                        }
                    }
                } else {
                    println!("  No Trigger files found in {}", triggers_path.display());
                }
            }
            Err(e) => {
                eprintln!("  Failed to load triggers from {}: {}", triggers_path.display(), e);
            }
        }
    }

    if platforms_registered == 0 {
        eprintln!("Warning: No platforms registered! Server will start but won't process any webhooks.");
        eprintln!("Configure platforms in your config file or set environment variables.");
    }

    // Load AgentFlows and set up FlowRouter
    let flows_dir_path = flows_dir
        .map(PathBuf::from)
        .or_else(|| config.spec.flows.directory.clone());

    // Track if agents were actually loaded (not just if flows were configured)
    let mut agents_loaded = false;

    if config.spec.flows.enabled {
        if let Some(ref flows_path) = flows_dir_path {
            println!("Loading AgentFlows from: {}", flows_path.display());

            match FlowRegistry::from_directory(flows_path).await {
                Ok(registry) => {
                    let flow_count = registry.len();
                    let flow_names = registry.list();

                    if flow_count > 0 {
                        // Create FlowRouter from registry
                        let flow_router = Arc::new(FlowRouter::new(Arc::new(registry)));

                        // Create Runtime for agent execution
                        let runtime = Arc::new(RwLock::new(Runtime::new()));

                        // Get agents directory for flow executor
                        let agents_path = agents_dir
                            .map(PathBuf::from)
                            .or_else(|| config.spec.agents.directory.clone());

                        // Set up handler with FlowRouter
                        handler.set_flow_router(flow_router);
                        handler.set_runtime(runtime.clone());

                        // Pre-load all agents from directory (indexes by kind: Agent & metadata.name)
                        if let Some(ref ap) = agents_path {
                            match handler.load_agents_from_directory(ap).await {
                                Ok(count) => {
                                    println!("  Pre-loaded {} agents from {:?}", count, ap);
                                    agents_loaded = true;
                                }
                                Err(e) => eprintln!("  Failed to pre-load agents: {}", e),
                            }
                        }

                        println!("  Loaded {} AgentFlows: {:?}", flow_count, flow_names);
                    } else {
                        println!("  No AgentFlow files found in {}", flows_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("  Failed to load AgentFlows from {}: {}", flows_path.display(), e);
                }
            }
        } else {
            println!("  No flows directory configured - using default agent routing");
        }
    } else {
        println!("  Flow-based routing disabled");
    }

    // Pre-load agents from directory if not already done
    if !agents_loaded {
        let agents_path = agents_dir
            .map(PathBuf::from)
            .or_else(|| config.spec.agents.directory.clone());

        if let Some(ref ap) = agents_path {
            // Load agents now - create runtime and set it up
            let runtime = Arc::new(RwLock::new(Runtime::new()));
            handler.set_runtime(runtime);
            match handler.load_agents_from_directory(ap).await {
                Ok(count) => println!("  Pre-loaded {} agents from {:?}", count, ap),
                Err(e) => eprintln!("  Failed to pre-load agents: {}", e),
            }
        }
    }

    // ============================================================================
    // Custom Axum App Integration
    // ============================================================================
    // Build custom Axum router that combines:
    // 1. Trigger webhook routes (from TriggerHandler)
    // 2. Config API routes (/api/config/*)
    // 3. WebSocket route (/ws)
    // 4. Static file serving (React build)

    // Create config state for API endpoints
    let config_state = ConfigState::new(workspace_path.clone());

    // Create shared state for handlers
    #[derive(Clone)]
    struct AppState {
        handler: Arc<TriggerHandler>,
        event_bus: Option<Arc<EventBroadcaster>>,
    }

    let app_state = AppState {
        handler: Arc::new(handler),
        event_bus: Some(event_bus.clone()),
    };

    // Build API router (config endpoints)
    let config_router = Router::new()
        .route("/config/agents", get(get_agents_config))
        .route("/config/tools", get(get_tools_config))
        .route("/config/version", get(get_config_version))
        .with_state(config_state.clone());

    // Build tools discovery router
    use crate::api::tools::{discover_tools, ToolsState};
    let tools_state = ToolsState::new();
    let tools_router = Router::new()
        .route("/config/tools/discover", get(discover_tools))
        .with_state(tools_state);

    // Build metrics router
    let metrics_state = MetricsState::new(Arc::clone(&metrics_cache));
    let metrics_router = Router::new()
        .route("/agents/:id/metrics", get(get_agent_metrics))
        .with_state(metrics_state);

    // Build conversation router (conversational agent creation)
    // Initialize Orchestrator with specialists
    use aof_conversational::{Orchestrator, ConversationSessionStore, WorkspacePersistence};
    use crate::api::conversation::{
        ConversationState, create_session, get_session, conversation_message,
        conversation_confirm, conversation_cancel,
    };

    // Create a simple model for conversation (using Claude Opus via environment)
    // In production, this would use configuration from serve config
    let conversation_model: Box<dyn aof_core::Model> = {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .unwrap_or_else(|_| {
                eprintln!("Warning: ANTHROPIC_API_KEY not set, conversation API will not work");
                String::new()
            });

        if !api_key.is_empty() {
            let model_config = aof_core::ModelConfig {
                provider: aof_core::ModelProvider::Anthropic,
                model: "claude-opus-4-20250514".to_string(),
                api_key: Some(api_key),
                endpoint: None,
                max_tokens: Some(8192),
                temperature: 0.7,
                timeout_secs: 60,
                headers: std::collections::HashMap::new(),
                extra: std::collections::HashMap::new(),
            };
            aof_llm::create_model(model_config).await.unwrap_or_else(|e| {
                eprintln!("Failed to create conversation model: {}", e);
                Box::new(NoOpModel::new())
            })
        } else {
            // Create a no-op model if no API key
            Box::new(NoOpModel::new())
        }
    };

    let skill_registry = Arc::new(aof_skills::SkillRegistry::new(aof_skills::SkillConfig::default()));
    let session_store = ConversationSessionStore::new(100, std::time::Duration::from_secs(1800));

    // Create specialist models
    let specialist_model: Arc<dyn aof_llm::Model> = {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            let model_config = aof_core::ModelConfig {
                provider: aof_core::ModelProvider::Anthropic,
                model: "claude-opus-4-20250514".to_string(),
                api_key: Some(api_key),
                endpoint: None,
                max_tokens: Some(8192),
                temperature: 0.7,
                timeout_secs: 60,
                headers: std::collections::HashMap::new(),
                extra: std::collections::HashMap::new(),
            };
            match aof_llm::create_model(model_config).await {
                Ok(boxed_model) => Arc::from(boxed_model),
                Err(e) => {
                    eprintln!("Failed to create specialist model: {}", e);
                    Arc::new(NoOpModel::new())
                }
            }
        } else {
            Arc::new(NoOpModel::new())
        }
    };

    let orchestrator = Orchestrator::new(conversation_model, session_store)
        .with_agent_creator(
            specialist_model.clone(),
            workspace_path.clone(),
            skill_registry.clone(),
        )
        .with_squad_builder(
            specialist_model.clone(),
            workspace_path.clone(),
        )
        .with_skill_teacher(
            workspace_path.join("skills"),
        )
        .with_scheduler(
            specialist_model.clone(),
            workspace_path.clone(),
        );

    let persistence = Arc::new(WorkspacePersistence::new(
        workspace_path.clone(),
        workspace_path.join("skills"),
    ));

    let conversation_state = ConversationState {
        orchestrator: Arc::new(RwLock::new(orchestrator)),
        persistence,
    };

    let conversation_router = Router::new()
        .route("/conversation/session", post(create_session))
        .route("/conversation/session/:id", get(get_session))
        .route("/conversation/message", post(conversation_message))
        .route("/conversation/confirm", post(conversation_confirm))
        .route("/conversation/cancel", post(conversation_cancel))
        .with_state(conversation_state);

    // Build coordination router (heartbeat health status)
    let coordination_router = if let Some(manager) = &coordination_manager {
        #[derive(Clone)]
        struct CoordinationState {
            manager: Arc<CoordinationManager>,
        }

        async fn get_coordination_health(
            axum::extract::State(state): axum::extract::State<CoordinationState>,
        ) -> axum::response::Json<serde_json::Value> {
            use serde_json::json;

            let health_records = state.manager.health_snapshot().await;

            json!({
                "agents": health_records,
                "heartbeat_config": {
                    "frequency_secs": 60,  // TODO: Get from actual config
                    "timeout_secs": 120,
                }
            }).into()
        }

        async fn get_coordination_metrics(
            axum::extract::State(state): axum::extract::State<CoordinationState>,
        ) -> axum::response::Json<serde_json::Value> {
            use serde_json::json;

            let snapshot = state.manager.metrics_snapshot().await;

            json!({
                "coordination_tokens": snapshot.coordination_tokens,
                "production_tokens": snapshot.production_tokens,
                "overhead_percent": snapshot.overhead_percent,
                "heartbeat_tokens": snapshot.heartbeat_tokens,
                "standup_tokens": snapshot.standup_tokens,
                "current_mode": snapshot.current_mode,
                "window_start": snapshot.window_start.to_rfc3339(),
                "auto_degrade_enabled": true, // TODO: Get from config
                "max_overhead_percent": 30.0, // TODO: Get from config
            }).into()
        }

        async fn post_coordination_mode(
            axum::extract::State(state): axum::extract::State<CoordinationState>,
            axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
        ) -> axum::response::Json<serde_json::Value> {
            use serde_json::json;

            // Extract mode from payload
            let mode_str = payload.get("mode").and_then(|m| m.as_str()).unwrap_or("full");
            let mode = match mode_str {
                "full" => CoordinationMode::Full,
                "standard" => CoordinationMode::Standard,
                "reduced" => CoordinationMode::Reduced,
                "heartbeat_only" => CoordinationMode::HeartbeatOnly,
                "disabled" => CoordinationMode::Disabled,
                _ => CoordinationMode::Full,
            };

            // Apply mode change via manager
            state.manager.apply_mode_change(mode).await;

            json!({
                "success": true,
                "mode": format!("{:?}", mode),
                "message": "Coordination mode updated (note: full implementation requires scheduler control)"
            }).into()
        }

        let coord_state = CoordinationState {
            manager: Arc::clone(manager),
        };

        Router::new()
            .route("/coordination/health", get(get_coordination_health))
            .route("/coordination/metrics", get(get_coordination_metrics))
            .route("/coordination/mode", post(post_coordination_mode))
            .with_state(coord_state)
    } else {
        // Coordination disabled - return empty response
        async fn get_coordination_disabled() -> axum::response::Json<serde_json::Value> {
            use serde_json::json;
            json!({
                "agents": [],
                "coordination_enabled": false
            }).into()
        }

        Router::new()
            .route("/coordination/health", get(get_coordination_disabled))
            .route("/coordination/metrics", get(get_coordination_disabled))
            .route("/coordination/mode", post(get_coordination_disabled))
    };

    // Build chat router
    use crate::api::chat::{ChatState, get_messages, send_message};
    let chat_state = ChatState::new(Some(event_bus.clone()));
    let chat_router = Router::new()
        .route("/chat/messages", get(get_messages).post(send_message))
        .with_state(chat_state);

    // Merge all API sub-routers
    let api_router = config_router
        .merge(tools_router)
        .merge(metrics_router)
        .merge(conversation_router)
        .merge(coordination_router)
        .merge(chat_router);

    // Import handlers from aof-triggers server (inline to avoid duplicating logic)
    use axum::extract::State;
    use axum::extract::Path as AxumPath;
    use axum::http::HeaderMap as AxumHeaderMap;
    use axum::response::IntoResponse;
    use axum::Json as AxumJson;

    // Health check handler
    async fn health_handler() -> impl IntoResponse {
        AxumJson(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    // Webhook handler (reused from TriggerServer)
    async fn webhook_handler(
        State(state): State<AppState>,
        AxumPath(platform): AxumPath<String>,
        headers: AxumHeaderMap,
        body: Bytes,
    ) -> impl IntoResponse {
        use axum::http::StatusCode;

        // Extract headers (lowercase for consistent access)
        let mut header_map = std::collections::HashMap::new();
        for (key, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                header_map.insert(key.as_str().to_lowercase(), value_str.to_string());
            }
        }

        // Handle Slack URL verification challenge specially
        if platform == "slack" {
            if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&body) {
                if payload.get("type").and_then(|t| t.as_str()) == Some("url_verification") {
                    if let Some(challenge) = payload.get("challenge").and_then(|c| c.as_str()) {
                        return (
                            StatusCode::OK,
                            [("content-type", "text/plain")],
                            challenge.to_string(),
                        ).into_response();
                    }
                }
            }
        }

        // Get platform implementation
        let platform_impl = match state.handler.get_platform(&platform) {
            Some(p) => p,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    AxumJson(serde_json::json!({"error": format!("Unknown platform: {}", platform)}))
                ).into_response();
            }
        };

        // Parse message
        let message = match platform_impl.parse_message(&body, &header_map).await {
            Ok(m) => m,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    AxumJson(serde_json::json!({"error": format!("Parse error: {}", e)}))
                ).into_response();
            }
        };

        // Handle message asynchronously
        let handler = Arc::clone(&state.handler);
        let platform_name = platform.clone();
        tokio::spawn(async move {
            if let Err(e) = handler.handle_message(&platform_name, message).await {
                tracing::error!("Failed to handle message: {}", e);
            }
        });

        // Return immediate acknowledgment
        (StatusCode::OK, AxumJson(serde_json::json!({"status": "accepted"}))).into_response()
    }

    // WebSocket handler (reused from TriggerServer)
    use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};

    async fn handle_websocket_upgrade(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
    ) -> impl IntoResponse {
        let event_bus = state.event_bus.clone();
        ws.on_upgrade(move |socket| websocket_handler(socket, event_bus))
    }

    async fn websocket_handler(socket: WebSocket, event_bus: Option<Arc<EventBroadcaster>>) {
        let Some(bus) = event_bus else {
            return;
        };

        let (mut sender, mut receiver) = socket.split();
        let mut event_rx = bus.subscribe();

        // Spawn task to forward coordination events to WebSocket client
        let send_task = tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                if sender.send(WsMessage::Text(json)).await.is_err() {
                                    tracing::info!("WebSocket client disconnected");
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to serialize event: {}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, dropped {} events", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        // Listen for client messages (close frames, pings)
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                WsMessage::Close(_) => break,
                WsMessage::Ping(_) => {},
                _ => {}
            }
        }

        send_task.abort();
    }

    // Build main router with all routes
    // Note: Don't add route for "/" here - fallback_service will handle it
    let mut app = Router::new()
        .route("/health", get(health_handler))
        .route("/webhook/:platform", post(webhook_handler))
        .route("/ws", get(handle_websocket_upgrade))
        .nest("/api", api_router)
        .with_state(app_state.clone());

    // Add static file serving if static_dir provided
    if let Some(static_path) = static_dir {
        let static_dir_path = PathBuf::from(static_path);
        if static_dir_path.exists() {
            println!("  Static files: {}", static_dir_path.display());

            // Serve static files with SPA fallback
            // For any route not matching /api or /ws, serve from static dir
            // If file not found, serve index.html (React Router handles client-side routing)
            let serve_dir = ServeDir::new(&static_dir_path)
                .fallback(ServeFile::new(static_dir_path.join("index.html")));

            // Nest static serving at root, but it won't override /api or /ws routes
            app = app.fallback_service(serve_dir);
        } else {
            eprintln!("Warning: Static directory not found: {}", static_dir_path.display());
        }
    }

    // Add CORS if enabled
    if config.spec.server.cors {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );
    }

    println!("Server starting...");
    println!("  Health check: http://{}/health", bind_addr);
    println!("  WebSocket: ws://{}/ws", bind_addr);
    println!("  Webhook endpoint: http://{}/webhook/{{platform}}", bind_addr);
    println!("  Config API: http://{}/api/config/agents", bind_addr);
    println!("  Config API: http://{}/api/config/tools", bind_addr);
    println!("  Config API: http://{}/api/config/version", bind_addr);
    println!("  Metrics API: http://{}/api/agents/{{id}}/metrics", bind_addr);
    println!("  Coordination API: http://{}/api/coordination/health", bind_addr);
    println!("  Chat API: http://{}/api/chat/messages", bind_addr);
    if static_dir.is_some() {
        println!("  Web UI: http://{}/", bind_addr);
    }
    println!("Press Ctrl+C to stop");

    // Start custom Axum server
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", bind_addr, e))?;

    tracing::info!("Listening on {}", bind_addr);

    // Start coordination manager if initialized
    if let Some(manager) = &coordination_manager {
        match manager.start().await {
            Ok(handles) => {
                println!("  Coordination manager: started ({} background tasks)", handles.len());
            }
            Err(e) => {
                eprintln!("Failed to start coordination manager: {}", e);
                return Err(anyhow::anyhow!("Coordination manager start error: {}", e));
            }
        }
    }

    // Handle graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\nShutdown signal received, stopping server...");
    };

    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
                return Err(anyhow::anyhow!("Server error: {}", e));
            }
        }
        _ = shutdown_signal => {
            // Graceful shutdown: gateway first, then server
            if let Some((hub_handle, shutdown_tx)) = gateway_handle {
                println!("  Stopping gateway...");
                let _ = shutdown_tx.send(true);
                if let Err(e) = hub_handle.await {
                    eprintln!("Warning: Gateway shutdown error: {}", e);
                } else {
                    println!("  Gateway stopped");
                }
            }

            // Save session state on shutdown
            let final_state = SessionState {
                session_id: session_id.clone(),
                agent_states: std::collections::HashMap::new(), // TODO: Collect from runtime in Phase 2+
                task_queue: Vec::new(),
                created_at: chrono::Utc::now(),
                last_updated: chrono::Utc::now(),
            };
            if let Err(e) = session_persistence.save_session(&final_state).await {
                eprintln!("Warning: Failed to save session state: {}", e);
            } else {
                println!("  Session state saved");
            }
            println!("Server stopped gracefully");
        }
    }

    Ok(())
}

/// Load optional squad-specific introduction overrides from squads.yaml
///
/// Returns a map of agent_id -> intro_override. If the file doesn't exist
/// or can't be parsed, returns None (graceful degradation).
async fn load_squad_overrides(
    path: &std::path::Path,
) -> Option<HashMap<String, String>> {
    if !path.exists() {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct SquadsFile {
        squads: Vec<SquadConfig>,
    }

    #[derive(serde::Deserialize)]
    struct SquadConfig {
        #[allow(dead_code)]
        name: String,
        agents: Vec<SquadAgentConfig>,
    }

    #[derive(serde::Deserialize)]
    struct SquadAgentConfig {
        id: String,
        intro_override: Option<String>,
    }

    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            match serde_yaml::from_str::<SquadsFile>(&content) {
                Ok(file) => {
                    let mut overrides = HashMap::new();
                    for squad in file.squads {
                        for agent in squad.agents {
                            if let Some(intro) = agent.intro_override {
                                overrides.insert(agent.id, intro);
                            }
                        }
                    }
                    if overrides.is_empty() {
                        None
                    } else {
                        tracing::info!(
                            count = overrides.len(),
                            "Loaded {} squad intro overrides from {:?}",
                            overrides.len(),
                            path
                        );
                        Some(overrides)
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to parse squads.yaml, ignoring overrides"
                    );
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to read squads.yaml, ignoring overrides"
            );
            None
        }
    }
}
