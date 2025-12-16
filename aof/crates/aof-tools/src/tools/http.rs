//! HTTP Tool
//!
//! Tool for making HTTP requests.
//!
//! ## Features
//!
//! - GET, POST, PUT, DELETE, PATCH methods
//! - Custom headers
//! - JSON body support
//! - Timeout control
//!
//! ## Prerequisites
//!
//! - Requires `http` feature flag
//!
//! ## MCP Alternative
//!
//! For MCP-based HTTP operations, use the fetch MCP server.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// HTTP request tool
pub struct HttpTool {
    config: ToolConfig,
}

impl HttpTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "url": {
                    "type": "string",
                    "description": "URL to request"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP method",
                    "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"],
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "Request headers",
                    "additionalProperties": { "type": "string" }
                },
                "body": {
                    "type": "string",
                    "description": "Request body (for POST/PUT/PATCH)"
                },
                "json": {
                    "type": "object",
                    "description": "JSON body (alternative to body string)"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "default": 30
                },
                "follow_redirects": {
                    "type": "boolean",
                    "description": "Follow HTTP redirects",
                    "default": true
                }
            }),
            vec!["url"],
        );

        Self {
            config: tool_config_with_timeout(
                "http_request",
                "Make an HTTP request. Returns status, headers, and body.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for HttpTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for HttpTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let url: String = input.get_arg("url")?;
        let method: String = input.get_arg("method").unwrap_or_else(|_| "GET".to_string());
        let headers: HashMap<String, String> = input.get_arg("headers").unwrap_or_default();
        let body: Option<String> = input.get_arg("body").ok();
        let json_body: Option<serde_json::Value> = input.get_arg("json").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(30);
        let follow_redirects: bool = input.get_arg("follow_redirects").unwrap_or(true);

        debug!(url = %url, method = %method, "Making HTTP request");

        // Build client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .redirect(if follow_redirects {
                reqwest::redirect::Policy::default()
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        // Build request
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            "HEAD" => client.head(&url),
            _ => return Ok(ToolResult::error(format!("Unsupported method: {}", method))),
        };

        // Add headers
        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        // Add body
        if let Some(json) = json_body {
            request = request.json(&json);
        } else if let Some(body_str) = body {
            request = request.body(body_str);
        }

        // Execute request
        let start = std::time::Instant::now();
        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("HTTP request failed: {}", e)));
            }
        };
        let elapsed = start.elapsed().as_millis() as u64;

        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown");

        // Get response headers
        let resp_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Get response body
        let body_text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to read response body: {}", e)));
            }
        };

        // Try to parse as JSON
        let body_json: Option<serde_json::Value> = serde_json::from_str(&body_text).ok();

        Ok(ToolResult::success(serde_json::json!({
            "status": status,
            "status_text": status_text,
            "headers": resp_headers,
            "body": if body_json.is_some() { body_json } else { Some(serde_json::json!(body_text)) },
            "elapsed_ms": elapsed,
            "url": url
        })).with_execution_time(elapsed))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_get() {
        let tool = HttpTool::new();
        let input = ToolInput::new(serde_json::json!({
            "url": "https://httpbin.org/get",
            "timeout_secs": 10
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["status"], 200);
    }
}
