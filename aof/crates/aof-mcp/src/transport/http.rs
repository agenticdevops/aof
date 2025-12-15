//! HTTP transport implementation for Model Context Protocol (MCP)
//!
//! This module provides a production-ready HTTP transport that supports:
//! - JSON-RPC 2.0 over HTTP POST
//! - Connection pooling with configurable limits
//! - Automatic endpoint validation
//! - Comprehensive error handling
//! - HTTP/2 support (optional)
//! - Graceful shutdown
//!
//! # Example
//!
//! ```no_run
//! use aof_mcp::transport::{http::HttpTransport, McpRequest, McpTransport};
//! use serde_json::json;
//!
//! # async fn example() -> aof_core::AofResult<()> {
//! // Create and initialize transport
//! let mut transport = HttpTransport::new("http://localhost:8080/mcp");
//! transport.init().await?;
//!
//! // Send request
//! let request = McpRequest::new("tools/list", json!({}));
//! let response = transport.request(&request).await?;
//!
//! // Cleanup
//! transport.shutdown().await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

use super::{McpRequest, McpResponse, McpTransport, TransportType};
use aof_core::{AofError, AofResult};

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Connection timeout in seconds
    pub timeout: Duration,
    /// Maximum number of connections per host
    pub pool_max_idle_per_host: usize,
    /// Connection keep-alive duration
    pub pool_idle_timeout: Duration,
    /// Enable HTTP/2
    pub http2_prior_knowledge: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
            http2_prior_knowledge: false,
        }
    }
}

/// HTTP transport for MCP using JSON-RPC over HTTP POST
pub struct HttpTransport {
    endpoint: String,
    client: Option<Arc<Client>>,
    config: HttpConfig,
}

impl HttpTransport {
    /// Create a new HTTP transport with default configuration
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self::with_config(endpoint, HttpConfig::default())
    }

    /// Create a new HTTP transport with custom configuration
    pub fn with_config(endpoint: impl Into<String>, config: HttpConfig) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: None,
            config,
        }
    }

    /// Validate the endpoint URL
    fn validate_endpoint(&self) -> AofResult<Url> {
        let url = Url::parse(&self.endpoint)
            .map_err(|e| AofError::mcp(format!("Invalid endpoint URL: {}", e)))?;

        match url.scheme() {
            "http" | "https" => Ok(url),
            scheme => Err(AofError::mcp(format!(
                "Invalid URL scheme '{}': only http and https are supported",
                scheme
            ))),
        }
    }

    /// Build the HTTP client with connection pooling
    fn build_client(&self) -> AofResult<Client> {
        debug!(
            "Building HTTP client with pool_max_idle_per_host={}, timeout={}s",
            self.config.pool_max_idle_per_host,
            self.config.timeout.as_secs()
        );

        let mut client_builder = Client::builder()
            .timeout(self.config.timeout)
            .pool_max_idle_per_host(self.config.pool_max_idle_per_host)
            .pool_idle_timeout(self.config.pool_idle_timeout)
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .http2_keep_alive_interval(Some(Duration::from_secs(30)))
            .http2_keep_alive_timeout(Duration::from_secs(10));

        if self.config.http2_prior_knowledge {
            client_builder = client_builder.http2_prior_knowledge();
        }

        client_builder
            .build()
            .map_err(|e| AofError::mcp(format!("Failed to build HTTP client: {}", e)))
    }

    /// Map HTTP status codes to appropriate errors
    fn handle_http_error(&self, status: StatusCode, body: String) -> AofError {
        match status {
            StatusCode::BAD_REQUEST => {
                AofError::mcp(format!("Bad request (400): {}", body))
            }
            StatusCode::UNAUTHORIZED => {
                AofError::mcp(format!("Unauthorized (401): {}", body))
            }
            StatusCode::FORBIDDEN => {
                AofError::mcp(format!("Forbidden (403): {}", body))
            }
            StatusCode::NOT_FOUND => {
                AofError::mcp(format!("Not found (404): {}", body))
            }
            StatusCode::REQUEST_TIMEOUT => {
                AofError::mcp(format!("Request timeout (408): {}", body))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                AofError::mcp(format!("Rate limited (429): {}", body))
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                AofError::mcp(format!("Server error (500): {}", body))
            }
            StatusCode::BAD_GATEWAY => {
                AofError::mcp(format!("Bad gateway (502): {}", body))
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                AofError::mcp(format!("Service unavailable (503): {}", body))
            }
            StatusCode::GATEWAY_TIMEOUT => {
                AofError::mcp(format!("Gateway timeout (504): {}", body))
            }
            _ => AofError::mcp(format!(
                "HTTP error {}: {}",
                status.as_u16(),
                body
            )),
        }
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn init(&mut self) -> AofResult<()> {
        debug!("Initializing HTTP transport: {}", self.endpoint);

        // Validate endpoint URL
        self.validate_endpoint()?;

        // Build HTTP client with connection pooling
        let client = self.build_client()?;
        self.client = Some(Arc::new(client));

        debug!("HTTP transport initialized successfully");
        Ok(())
    }

    async fn request(&self, request: &McpRequest) -> AofResult<McpResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| AofError::mcp("Transport not initialized"))?;

        // Serialize request to JSON
        let request_json = serde_json::to_string(request)
            .map_err(|e| AofError::mcp(format!("Failed to serialize request: {}", e)))?;

        debug!("Sending HTTP POST request to {}: {}", self.endpoint, request_json);

        // Send HTTP POST request with JSON-RPC payload
        let response = client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(request_json)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AofError::mcp(format!("Request timeout after {}s", self.config.timeout.as_secs()))
                } else if e.is_connect() {
                    AofError::mcp(format!("Connection failed: {}", e))
                } else {
                    AofError::mcp(format!("HTTP request failed: {}", e))
                }
            })?;

        let status = response.status();

        // Get response body
        let body = response
            .text()
            .await
            .map_err(|e| AofError::mcp(format!("Failed to read response body: {}", e)))?;

        debug!("Received HTTP response (status {}): {}", status, body);

        // Check HTTP status code
        if !status.is_success() {
            return Err(self.handle_http_error(status, body));
        }

        // Parse JSON-RPC response
        let mcp_response: McpResponse = serde_json::from_str(&body)
            .map_err(|e| AofError::mcp(format!("Failed to parse JSON-RPC response: {}", e)))?;

        // Check for JSON-RPC error
        if let Some(error) = &mcp_response.error {
            warn!(
                "MCP error response: code={}, message={}",
                error.code, error.message
            );
            return Err(AofError::mcp(format!(
                "MCP error {}: {}",
                error.code, error.message
            )));
        }

        Ok(mcp_response)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Http
    }

    async fn shutdown(&mut self) -> AofResult<()> {
        debug!("Shutting down HTTP transport");

        // Drop the client to close all connections
        self.client = None;

        debug!("HTTP transport shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_endpoint_valid_http() {
        let transport = HttpTransport::new("http://localhost:8080/mcp");
        assert!(transport.validate_endpoint().is_ok());
    }

    #[test]
    fn test_validate_endpoint_valid_https() {
        let transport = HttpTransport::new("https://api.example.com/mcp");
        assert!(transport.validate_endpoint().is_ok());
    }

    #[test]
    fn test_validate_endpoint_invalid_scheme() {
        let transport = HttpTransport::new("ftp://example.com");
        assert!(transport.validate_endpoint().is_err());
    }

    #[test]
    fn test_validate_endpoint_invalid_url() {
        let transport = HttpTransport::new("not a url");
        assert!(transport.validate_endpoint().is_err());
    }

    #[test]
    fn test_default_config() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.pool_max_idle_per_host, 10);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(90));
        assert!(!config.http2_prior_knowledge);
    }

    #[test]
    fn test_custom_config() {
        let config = HttpConfig {
            timeout: Duration::from_secs(60),
            pool_max_idle_per_host: 20,
            pool_idle_timeout: Duration::from_secs(120),
            http2_prior_knowledge: true,
        };
        let transport = HttpTransport::with_config("http://localhost:8080", config.clone());
        assert_eq!(transport.config.timeout, config.timeout);
        assert_eq!(transport.config.pool_max_idle_per_host, config.pool_max_idle_per_host);
    }

    #[tokio::test]
    async fn test_transport_not_initialized() {
        let transport = HttpTransport::new("http://localhost:8080");
        let request = McpRequest::new("test", json!({}));
        let result = transport.request(&request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_init_success() {
        let mut transport = HttpTransport::new("http://localhost:8080/mcp");
        let result = transport.init().await;
        assert!(result.is_ok());
        assert!(transport.client.is_some());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let mut transport = HttpTransport::new("http://localhost:8080/mcp");
        transport.init().await.unwrap();
        assert!(transport.client.is_some());

        transport.shutdown().await.unwrap();
        assert!(transport.client.is_none());
    }
}
