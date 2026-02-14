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
