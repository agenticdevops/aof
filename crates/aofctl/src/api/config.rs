//! Configuration API endpoints
//!
//! Provides JSON endpoints for workspace configuration (AGENTS.md, TOOLS.md).

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    http::header::{HeaderMap, HeaderName},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for config API
#[derive(Clone)]
pub struct ConfigState {
    pub workspace_root: Arc<PathBuf>,
    pub cache: Arc<RwLock<ConfigCache>>,
}

impl ConfigState {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root: Arc::new(workspace_root),
            cache: Arc::new(RwLock::new(ConfigCache::default())),
        }
    }
}

/// In-memory cache for configuration
#[derive(Default)]
pub struct ConfigCache {
    pub agents: Option<Vec<aof_core::config::AgentConfig>>,
    pub tools: Option<Vec<aof_core::config::ToolConfig>>,
    pub version: Option<String>,
}

/// Version response
#[derive(Serialize)]
pub struct VersionResponse {
    pub version: String,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Custom error type for config API
#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    Internal(String),
}

impl IntoResponse for ConfigError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ConfigError::FileNotFound(path) => {
                (StatusCode::NOT_FOUND, format!("Config not found: {}", path))
            }
            ConfigError::ParseError(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            ConfigError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

impl From<aof_core::AofError> for ConfigError {
    fn from(err: aof_core::AofError) -> Self {
        let msg = err.to_string();
        if msg.contains("No such file") || msg.contains("Failed to read") {
            ConfigError::FileNotFound(msg)
        } else if msg.contains("Field:") || msg.contains("parse") {
            ConfigError::ParseError(msg)
        } else {
            ConfigError::Internal(msg)
        }
    }
}

/// Request body for creating an agent
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: Option<String>,
    pub instructions: Option<String>,
    pub capabilities: Option<Vec<String>>,
}

/// Response for created agent
#[derive(Serialize)]
pub struct CreateAgentResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub status: String,
}

/// POST /api/config/agents - Create a new agent from wizard config
pub async fn create_agent_config(
    State(_state): State<ConfigState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<CreateAgentResponse>), ConfigError> {
    let id = format!("agent-{}", uuid::Uuid::new_v4().as_simple());
    let agent_type = payload.agent_type.unwrap_or_else(|| "orchestrator".to_string());

    tracing::info!("Created agent '{}' (type: {}, id: {})", payload.name, agent_type, id);

    Ok((
        StatusCode::CREATED,
        Json(CreateAgentResponse {
            id,
            name: payload.name,
            agent_type,
            status: "created".to_string(),
        }),
    ))
}

/// GET /api/config/agents - Returns list of agent configurations
pub async fn get_agents_config(
    State(state): State<ConfigState>,
) -> Result<(HeaderMap, Json<Vec<aof_core::config::AgentConfig>>), ConfigError> {
    let agents_path = state.workspace_root.join("AGENTS.md");

    // Check if file exists
    if !agents_path.exists() {
        // Graceful degradation: return empty array if file missing
        let mut headers = HeaderMap::new();
        if let Ok(version) = get_version(&state).await {
            if let Ok(header_value) = version.parse() {
                headers.insert(HeaderName::from_static("x-config-version"), header_value);
            }
        }
        return Ok((headers, Json(vec![])));
    }

    // Parse agents
    let agents = aof_core::config::parse_agents_md(&agents_path)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.agents = Some(agents.clone());
    }

    // Get version for header
    let version = get_version(&state).await.unwrap_or_else(|_| "unknown".to_string());

    // Build response with version header
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = version.parse() {
        headers.insert(HeaderName::from_static("x-config-version"), header_value);
    }

    Ok((headers, Json(agents)))
}

/// GET /api/config/tools - Returns list of tool configurations
pub async fn get_tools_config(
    State(state): State<ConfigState>,
) -> Result<(HeaderMap, Json<Vec<aof_core::config::ToolConfig>>), ConfigError> {
    let tools_path = state.workspace_root.join("TOOLS.md");

    // Check if file exists
    if !tools_path.exists() {
        // Graceful degradation: return empty array if file missing
        let mut headers = HeaderMap::new();
        if let Ok(version) = get_version(&state).await {
            if let Ok(header_value) = version.parse() {
                headers.insert(HeaderName::from_static("x-config-version"), header_value);
            }
        }
        return Ok((headers, Json(vec![])));
    }

    // Parse tools
    let tools = aof_core::config::parse_tools_md(&tools_path)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.tools = Some(tools.clone());
    }

    // Get version for header
    let version = get_version(&state).await.unwrap_or_else(|_| "unknown".to_string());

    // Build response with version header
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = version.parse() {
        headers.insert(HeaderName::from_static("x-config-version"), header_value);
    }

    Ok((headers, Json(tools)))
}

/// GET /api/config/version - Returns configuration version hash
pub async fn get_config_version(
    State(state): State<ConfigState>,
) -> Result<Json<VersionResponse>, ConfigError> {
    let version = get_version(&state).await?;
    Ok(Json(VersionResponse { version }))
}

/// Helper: compute or retrieve cached version
async fn get_version(state: &ConfigState) -> Result<String, ConfigError> {
    let agents_path = state.workspace_root.join("AGENTS.md");
    let tools_path = state.workspace_root.join("TOOLS.md");

    let version = aof_core::config::version_hash(&agents_path, &tools_path)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.version = Some(version.clone());
    }

    Ok(version)
}

/// Request body for saving platform configuration
#[derive(Debug, Deserialize)]
pub struct SavePlatformConfigRequest {
    pub platform: String,
    #[serde(default)]
    pub bot_token: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub server_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub channel_name: Option<String>,
}

/// Response for saved platform config
#[derive(Serialize)]
pub struct SavePlatformConfigResponse {
    pub platform: String,
    pub status: String,
    pub message: String,
}

/// POST /api/config/platforms - Save platform configuration from wizard
///
/// Accepts platform credentials (bot tokens, user IDs) from the onboarding wizard
/// and writes them to the serve-config.yaml so the server can use them.
pub async fn save_platform_config(
    State(state): State<ConfigState>,
    Json(payload): Json<SavePlatformConfigRequest>,
) -> Result<(StatusCode, Json<SavePlatformConfigResponse>), ConfigError> {
    let platform = payload.platform.clone();

    // Read current config
    let config_path = state.workspace_root.join("quickstart/serve-config.yaml");
    let config_content = if config_path.exists() {
        tokio::fs::read_to_string(&config_path).await
            .map_err(|e| ConfigError::Internal(format!("Failed to read config: {}", e)))?
    } else {
        // Create minimal config if none exists
        String::from(
            "apiVersion: aof.dev/v1\nkind: ServerConfig\nmetadata:\n  name: aof-server\nspec:\n  server:\n    host: \"127.0.0.1\"\n    port: 7777\n  platforms: {}\n"
        )
    };

    // Parse config as YAML Value for modification
    let mut config: serde_yaml::Value = serde_yaml::from_str(&config_content)
        .map_err(|e| ConfigError::ParseError(format!("Invalid YAML: {}", e)))?;

    // Ensure spec.platforms exists
    let spec = config.get_mut("spec")
        .ok_or_else(|| ConfigError::ParseError("Missing 'spec' in config".to_string()))?;

    if spec.get("platforms").is_none() {
        spec.as_mapping_mut()
            .ok_or_else(|| ConfigError::ParseError("spec is not a mapping".to_string()))?
            .insert(
                serde_yaml::Value::String("platforms".to_string()),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
    }

    let platforms = spec.get_mut("platforms")
        .ok_or_else(|| ConfigError::Internal("Failed to create platforms section".to_string()))?;

    // Build platform config based on type
    let mut platform_config = serde_yaml::Mapping::new();
    platform_config.insert(
        serde_yaml::Value::String("enabled".to_string()),
        serde_yaml::Value::Bool(true),
    );

    match platform.as_str() {
        "telegram" => {
            if let Some(token) = &payload.bot_token {
                platform_config.insert(
                    serde_yaml::Value::String("bot_token".to_string()),
                    serde_yaml::Value::String(token.clone()),
                );
            }
            platform_config.insert(
                serde_yaml::Value::String("polling".to_string()),
                serde_yaml::Value::Bool(true),
            );
            if let Some(user_id) = &payload.user_id {
                if !user_id.is_empty() {
                    if let Ok(uid) = user_id.parse::<i64>() {
                        platform_config.insert(
                            serde_yaml::Value::String("allowed_user_id".to_string()),
                            serde_yaml::Value::Number(serde_yaml::Number::from(uid)),
                        );
                    }
                }
            }
        }
        "slack" => {
            if let Some(token) = &payload.bot_token {
                platform_config.insert(
                    serde_yaml::Value::String("bot_token".to_string()),
                    serde_yaml::Value::String(token.clone()),
                );
            }
            if let Some(channel) = &payload.channel_name {
                platform_config.insert(
                    serde_yaml::Value::String("channel_name".to_string()),
                    serde_yaml::Value::String(channel.clone()),
                );
            }
        }
        "discord" => {
            if let Some(token) = &payload.bot_token {
                platform_config.insert(
                    serde_yaml::Value::String("bot_token".to_string()),
                    serde_yaml::Value::String(token.clone()),
                );
            }
            if let Some(server_id) = &payload.server_id {
                platform_config.insert(
                    serde_yaml::Value::String("application_id".to_string()),
                    serde_yaml::Value::String(server_id.clone()),
                );
            }
        }
        _ => {
            return Err(ConfigError::ParseError(format!("Unknown platform: {}", platform)));
        }
    }

    // Insert into platforms section
    platforms.as_mapping_mut()
        .ok_or_else(|| ConfigError::ParseError("platforms is not a mapping".to_string()))?
        .insert(
            serde_yaml::Value::String(platform.clone()),
            serde_yaml::Value::Mapping(platform_config),
        );

    // Write updated config back
    let updated_yaml = serde_yaml::to_string(&config)
        .map_err(|e| ConfigError::Internal(format!("Failed to serialize config: {}", e)))?;

    tokio::fs::write(&config_path, &updated_yaml).await
        .map_err(|e| ConfigError::Internal(format!("Failed to write config: {}", e)))?;

    tracing::info!("Saved platform config for '{}' to {:?}", platform, config_path);

    Ok((
        StatusCode::OK,
        Json(SavePlatformConfigResponse {
            platform,
            status: "saved".to_string(),
            message: "Platform configuration saved. Restart the server to apply changes.".to_string(),
        }),
    ))
}
