use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use aof_core::AofResult;

/// MCP transport abstraction
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a request and receive response
    async fn request(&self, request: &McpRequest) -> AofResult<McpResponse>;

    /// Get transport type
    fn transport_type(&self) -> TransportType;

    /// Initialize transport
    async fn init(&mut self) -> AofResult<()>;

    /// Shutdown transport
    async fn shutdown(&mut self) -> AofResult<()>;
}

/// Transport type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    Http,
}

/// MCP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl McpRequest {
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: method.into(),
            params,
        }
    }
}

/// MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Transport implementations
pub mod stdio;
#[cfg(feature = "sse")]
pub mod sse;
#[cfg(feature = "http")]
pub mod http;
