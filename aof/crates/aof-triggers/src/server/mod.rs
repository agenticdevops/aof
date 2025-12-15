//! Webhook server using axum
//!
//! This module provides the HTTP server for receiving webhooks
//! from various messaging platforms.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use crate::handler::TriggerHandler;
use crate::platforms::TriggerMessage;

/// Server configuration
#[derive(Debug, Clone)]
pub struct TriggerServerConfig {
    /// Bind address
    pub bind_addr: SocketAddr,

    /// Enable CORS
    pub enable_cors: bool,

    /// Request timeout seconds
    pub timeout_secs: u64,

    /// Maximum request body size
    pub max_body_size: usize,
}

impl Default for TriggerServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().unwrap(),
            enable_cors: true,
            timeout_secs: 30,
            max_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Server state
#[derive(Clone)]
struct AppState {
    handler: Arc<TriggerHandler>,
}

/// Webhook server
pub struct TriggerServer {
    config: TriggerServerConfig,
    handler: Arc<TriggerHandler>,
}

impl TriggerServer {
    /// Create a new trigger server
    pub fn new(handler: Arc<TriggerHandler>) -> Self {
        Self {
            config: TriggerServerConfig::default(),
            handler,
        }
    }

    /// Create server with custom configuration
    pub fn with_config(handler: Arc<TriggerHandler>, config: TriggerServerConfig) -> Self {
        Self { config, handler }
    }

    /// Create a builder for fluent configuration
    pub fn builder() -> TriggerServerBuilder {
        TriggerServerBuilder::new()
    }

    /// Start the server
    pub async fn serve(self) -> Result<(), ServerError> {
        let state = AppState {
            handler: self.handler,
        };

        let app = Router::new()
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .route("/webhook/:platform", post(webhook_handler))
            .route("/platforms", get(platforms_handler))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        info!("Starting webhook server on {}", self.config.bind_addr);

        let listener = tokio::net::TcpListener::bind(&self.config.bind_addr)
            .await
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| ServerError::ServerError(e.to_string()))?;

        Ok(())
    }
}

/// Server builder
pub struct TriggerServerBuilder {
    config: TriggerServerConfig,
    handler: Option<Arc<TriggerHandler>>,
}

impl TriggerServerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: TriggerServerConfig::default(),
            handler: None,
        }
    }

    /// Set the handler
    pub fn handler(mut self, handler: Arc<TriggerHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Set bind address
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.config.bind_addr = addr.into();
        self
    }

    /// Enable or disable CORS
    pub fn cors(mut self, enable: bool) -> Self {
        self.config.enable_cors = enable;
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Build the server
    pub fn build(self) -> Result<TriggerServer, ServerError> {
        let handler = self
            .handler
            .ok_or_else(|| ServerError::ConfigError("Handler not set".to_string()))?;

        Ok(TriggerServer {
            config: self.config,
            handler,
        })
    }
}

impl Default for TriggerServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Server errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Bind error: {0}")]
    BindError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Root handler
async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "aof-triggers",
        "version": crate::VERSION,
        "status": "running"
    }))
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Webhook handler
async fn webhook_handler(
    State(state): State<AppState>,
    Path(platform): Path<String>,
    headers: axum::http::HeaderMap,
    body: bytes::Bytes,
) -> Result<Response, WebhookError> {
    debug!("Received webhook for platform: {}", platform);

    // Extract headers
    let mut header_map = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(key.to_string(), value_str.to_string());
        }
    }

    // Get platform implementation
    let platform_impl = state
        .handler
        .get_platform(&platform)
        .ok_or_else(|| WebhookError::UnknownPlatform(platform.clone()))?;

    // Parse message
    let message = platform_impl
        .parse_message(&body, &header_map)
        .await
        .map_err(|e| WebhookError::ParseError(e.to_string()))?;

    // Handle message asynchronously (fire and forget)
    let handler = Arc::clone(&state.handler);
    let platform_name = platform.clone();
    tokio::spawn(async move {
        if let Err(e) = handler.handle_message(&platform_name, message).await {
            error!("Failed to handle message: {}", e);
        }
    });

    // Return immediate acknowledgment
    Ok(Json(serde_json::json!({
        "status": "accepted"
    }))
    .into_response())
}

/// List registered platforms
async fn platforms_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Note: This requires adding a method to TriggerHandler to list platforms
    // For now, return a simple response
    Json(serde_json::json!({
        "platforms": []
    }))
}

/// Webhook error type
#[derive(Debug)]
enum WebhookError {
    UnknownPlatform(String),
    ParseError(String),
}

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebhookError::UnknownPlatform(platform) => {
                (StatusCode::NOT_FOUND, format!("Unknown platform: {}", platform))
            }
            WebhookError::ParseError(msg) => {
                (StatusCode::BAD_REQUEST, format!("Parse error: {}", msg))
            }
        };

        (
            status,
            Json(serde_json::json!({
                "error": message
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_runtime::RuntimeOrchestrator;

    #[test]
    fn test_server_builder() {
        let orchestrator = Arc::new(RuntimeOrchestrator::new());
        let handler = Arc::new(TriggerHandler::new(orchestrator));

        let result = TriggerServer::builder()
            .handler(handler)
            .bind("127.0.0.1:8080".parse::<SocketAddr>().unwrap())
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = TriggerServerConfig::default();
        assert_eq!(config.bind_addr.port(), 8080);
        assert!(config.enable_cors);
        assert_eq!(config.timeout_secs, 30);
    }
}
