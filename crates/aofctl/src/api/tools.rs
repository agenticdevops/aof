//! Tool discovery API endpoints
//!
//! Provides endpoints for discovering system tools (kubectl, terraform, docker, etc.)
//! on the local system.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use aof_tools::{ToolDiscovery, DiscoveredTool, ToolDiscoveryError};

/// Shared state for tools API
#[derive(Clone)]
pub struct ToolsState {
    pub discovery: Arc<RwLock<ToolDiscovery>>,
}

impl ToolsState {
    pub fn new() -> Self {
        Self {
            discovery: Arc::new(RwLock::new(ToolDiscovery::new(None))),
        }
    }
}

/// Tool response for API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub category: String,
    pub available: bool,
}

impl From<DiscoveredTool> for ToolResponse {
    fn from(tool: DiscoveredTool) -> Self {
        Self {
            id: tool.id,
            name: tool.name,
            path: tool.path.to_string_lossy().to_string(),
            version: tool.version,
            category: tool.category.to_string(),
            available: tool.available,
        }
    }
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Custom error type for tools API
#[derive(Debug)]
pub enum ToolsError {
    DiscoveryFailed(String),
    Internal(String),
}

impl IntoResponse for ToolsError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ToolsError::DiscoveryFailed(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            ToolsError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

impl From<ToolDiscoveryError> for ToolsError {
    fn from(err: ToolDiscoveryError) -> Self {
        ToolsError::DiscoveryFailed(err.to_string())
    }
}

/// GET /api/config/tools/discover - Discover available tools on the system
pub async fn discover_tools(
    State(state): State<ToolsState>,
) -> Result<Json<Vec<ToolResponse>>, ToolsError> {
    let discovery = state.discovery.read().await;

    let tools = discovery
        .scan_system_paths()
        .await
        .map_err(ToolsError::from)?;

    let responses: Vec<ToolResponse> = tools
        .into_iter()
        .map(|tool| ToolResponse::from(tool))
        .collect();

    Ok(Json(responses))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_response_from_discovered() {
        use aof_tools::DiscoveredTool;
        use std::path::PathBuf;

        let tool = DiscoveredTool {
            id: "kubectl".to_string(),
            name: "Kubernetes CLI".to_string(),
            path: PathBuf::from("/usr/local/bin/kubectl"),
            version: Some("v1.29.0".to_string()),
            category: aof_tools::ToolCategory::Kubectl,
            available: true,
            size_bytes: 45000000,
            detected_at: SystemTime::now(),
        };

        let response = ToolResponse::from(tool);
        assert_eq!(response.id, "kubectl");
        assert_eq!(response.category, "Kubectl");
        assert_eq!(response.version, Some("v1.29.0".to_string()));
    }

    #[test]
    fn test_tools_state_creation() {
        let state = ToolsState::new();
        assert!(state.discovery.try_read().is_ok());
    }
}
