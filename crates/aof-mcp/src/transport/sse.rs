use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

use super::{McpRequest, McpResponse, McpTransport, TransportType};
use aof_core::{AofError, AofResult};

/// SSE transport for MCP (Server-Sent Events)
///
/// Uses HTTP POST for sending requests and Server-Sent Events for receiving responses.
/// This follows the MCP SSE transport specification where:
/// - Requests are sent via POST to the endpoint
/// - Responses are received via SSE stream from the endpoint/sse
pub struct SseTransport {
    endpoint: String,
    client: Arc<Mutex<Option<Client>>>,
    session_id: Arc<Mutex<Option<String>>>,
}

impl SseTransport {
    /// Create a new SSE transport with the given endpoint URL
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: Arc::new(Mutex::new(None)),
            session_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Parse an SSE event line into field name and value
    fn parse_sse_line(line: &str) -> Option<(&str, &str)> {
        if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let value = line[colon_pos + 1..].trim_start();
            Some((field, value))
        } else {
            None
        }
    }

    /// Parse SSE stream and extract JSON-RPC response
    async fn parse_sse_response(body: Bytes) -> AofResult<McpResponse> {
        let text = String::from_utf8(body.to_vec())
            .map_err(|e| AofError::mcp(format!("Invalid UTF-8 in SSE response: {}", e)))?;

        let mut event_type: Option<String> = None;
        let mut data_lines: Vec<String> = Vec::new();

        for line in text.lines() {
            let line = line.trim();

            // Empty line indicates end of event
            if line.is_empty() {
                if !data_lines.is_empty() {
                    let data = data_lines.join("\n");

                    // Parse the JSON-RPC response from the data field
                    let response: McpResponse = serde_json::from_str(&data)
                        .map_err(|e| AofError::mcp(format!("Failed to parse SSE data as JSON-RPC: {}", e)))?;

                    debug!("Parsed SSE event type: {:?}, response ID: {}", event_type, response.id);
                    return Ok(response);
                }

                // Reset for next event
                event_type = None;
                data_lines.clear();
                continue;
            }

            // Skip comment lines
            if line.starts_with(':') {
                continue;
            }

            // Parse field: value
            if let Some((field, value)) = Self::parse_sse_line(line) {
                match field {
                    "event" => {
                        event_type = Some(value.to_string());
                    }
                    "data" => {
                        data_lines.push(value.to_string());
                    }
                    "id" => {
                        debug!("SSE message ID: {}", value);
                    }
                    "retry" => {
                        debug!("SSE retry: {}", value);
                    }
                    _ => {
                        warn!("Unknown SSE field: {}", field);
                    }
                }
            }
        }

        // If we got here and have accumulated data, try to parse it
        if !data_lines.is_empty() {
            let data = data_lines.join("\n");
            let response: McpResponse = serde_json::from_str(&data)
                .map_err(|e| AofError::mcp(format!("Failed to parse final SSE data: {}", e)))?;
            return Ok(response);
        }

        Err(AofError::mcp("No valid SSE response received"))
    }
}

#[async_trait]
impl McpTransport for SseTransport {
    async fn init(&mut self) -> AofResult<()> {
        debug!("Initializing SSE transport: {}", self.endpoint);

        // Create HTTP client with appropriate timeouts
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AofError::mcp(format!("Failed to create HTTP client: {}", e)))?;

        *self.client.lock().await = Some(client);

        // Initialize session by sending an initialize request
        // The session ID will be extracted from the response headers if provided
        debug!("SSE transport initialized successfully");

        Ok(())
    }

    async fn request(&self, request: &McpRequest) -> AofResult<McpResponse> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AofError::mcp("Transport not initialized"))?;

        // Serialize request to JSON
        let request_json = serde_json::to_string(request)
            .map_err(|e| AofError::mcp(format!("Failed to serialize request: {}", e)))?;

        debug!("Sending SSE request to {}: {}", self.endpoint, request_json);

        // Send POST request with JSON-RPC payload
        let mut req_builder = client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream");

        // Add session ID if we have one
        if let Some(session_id) = self.session_id.lock().await.as_ref() {
            req_builder = req_builder.header("X-Session-ID", session_id);
        }

        let response = req_builder
            .body(request_json)
            .send()
            .await
            .map_err(|e| AofError::mcp(format!("Failed to send request: {}", e)))?;

        // Check status code
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AofError::mcp(format!(
                "HTTP error {}: {}",
                status, error_body
            )));
        }

        // Extract session ID from response headers if present
        if let Some(session_id) = response.headers().get("X-Session-ID") {
            if let Ok(session_id_str) = session_id.to_str() {
                *self.session_id.lock().await = Some(session_id_str.to_string());
                debug!("Session ID set: {}", session_id_str);
            }
        }

        // Check content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/event-stream") && !content_type.contains("application/json") {
            warn!("Unexpected content type: {}", content_type);
        }

        // Read response body
        let body = response
            .bytes()
            .await
            .map_err(|e| AofError::mcp(format!("Failed to read response body: {}", e)))?;

        debug!("Received SSE response: {} bytes", body.len());

        // Parse SSE response
        let mcp_response = Self::parse_sse_response(body).await?;

        // Check for MCP error in response
        if let Some(error) = &mcp_response.error {
            error!(
                "MCP error in response: code={}, message={}",
                error.code, error.message
            );
            return Err(AofError::mcp(format!(
                "MCP error {}: {}",
                error.code, error.message
            )));
        }

        debug!("SSE request completed successfully, response ID: {}", mcp_response.id);

        Ok(mcp_response)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Sse
    }

    async fn shutdown(&mut self) -> AofResult<()> {
        debug!("Shutting down SSE transport");

        // Clear client and session
        *self.client.lock().await = None;
        *self.session_id.lock().await = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line() {
        assert_eq!(
            SseTransport::parse_sse_line("event: message"),
            Some(("event", "message"))
        );
        assert_eq!(
            SseTransport::parse_sse_line("data: {\"test\":\"value\"}"),
            Some(("data", "{\"test\":\"value\"}"))
        );
        assert_eq!(
            SseTransport::parse_sse_line("data:no-space"),
            Some(("data", "no-space"))
        );
        assert_eq!(SseTransport::parse_sse_line("invalid"), None);
    }

    #[tokio::test]
    async fn test_parse_sse_response_single_line() {
        let sse_data = "data: {\"jsonrpc\":\"2.0\",\"id\":\"test-id\",\"result\":{\"success\":true}}\n\n";
        let bytes = Bytes::from(sse_data);

        let response = SseTransport::parse_sse_response(bytes).await.unwrap();
        assert_eq!(response.id, "test-id");
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_parse_sse_response_multi_line() {
        let sse_data = "event: response\ndata: {\"jsonrpc\":\"2.0\",\ndata: \"id\":\"multi-test\",\ndata: \"result\":{\"value\":42}}\n\n";
        let bytes = Bytes::from(sse_data);

        let response = SseTransport::parse_sse_response(bytes).await.unwrap();
        assert_eq!(response.id, "multi-test");
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_parse_sse_response_with_error() {
        let sse_data = "data: {\"jsonrpc\":\"2.0\",\"id\":\"error-test\",\"error\":{\"code\":-32600,\"message\":\"Invalid request\"}}\n\n";
        let bytes = Bytes::from(sse_data);

        let response = SseTransport::parse_sse_response(bytes).await.unwrap();
        assert_eq!(response.id, "error-test");
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[test]
    fn test_transport_type() {
        let transport = SseTransport::new("http://example.com");
        assert_eq!(transport.transport_type(), TransportType::Sse);
    }
}
