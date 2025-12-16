use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::transport::{McpRequest, McpTransport, TransportType};
use aof_core::tool::ToolDefinition;
use aof_core::{AofError, AofResult};

/// MCP client
pub struct McpClient {
    transport: Arc<RwLock<Box<dyn McpTransport>>>,
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
    initialized: Arc<RwLock<bool>>,
}

impl McpClient {
    /// Create new MCP client with transport
    pub fn new(transport: Box<dyn McpTransport>) -> Self {
        Self {
            transport: Arc::new(RwLock::new(transport)),
            tools: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize MCP connection with optional server initialization options
    pub async fn initialize_with_options(&self, init_options: Option<serde_json::Value>) -> AofResult<()> {
        info!("Initializing MCP client");

        // Initialize transport
        let mut transport = self.transport.write().await;
        transport.init().await?;

        // Build initialize request with server options
        let mut init_payload = serde_json::json!({
            "protocolVersion": crate::MCP_VERSION,
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "aof",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        // Add server initialization options if provided
        if let Some(options) = init_options {
            if let Some(obj) = init_payload.as_object_mut() {
                obj.insert("initializationOptions".to_string(), options);
            }
        }

        let init_request = McpRequest::new("initialize", init_payload);

        let response = transport.request(&init_request).await?;
        debug!("Initialize response: {:?}", response);

        // List available tools
        let tools_request = McpRequest::new("tools/list", serde_json::json!({}));
        let tools_response = transport.request(&tools_request).await?;

        if let Some(result) = tools_response.result {
            if let Some(tools_array) = result.get("tools").and_then(|t| t.as_array()) {
                let mut tools = self.tools.write().await;
                for tool in tools_array {
                    if let Ok(tool_def) = serde_json::from_value::<ToolDefinition>(tool.clone()) {
                        info!("Registered MCP tool: {}", tool_def.name);
                        tools.insert(tool_def.name.clone(), tool_def);
                    }
                }
            }
        }

        *self.initialized.write().await = true;
        Ok(())
    }

    /// Initialize MCP connection (without server options)
    pub async fn initialize(&self) -> AofResult<()> {
        self.initialize_with_options(None).await
    }

    /// Call an MCP tool
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> AofResult<serde_json::Value> {
        if !*self.initialized.read().await {
            return Err(AofError::mcp("Client not initialized"));
        }

        debug!("Calling MCP tool: {} with args: {:?}", name, arguments);

        let request = McpRequest::new(
            "tools/call",
            serde_json::json!({
                "name": name,
                "arguments": arguments
            }),
        );

        let transport = self.transport.read().await;
        let response = transport.request(&request).await?;

        response
            .result
            .ok_or_else(|| AofError::mcp("No result in response"))
    }

    /// List available tools
    pub async fn list_tools(&self) -> AofResult<Vec<ToolDefinition>> {
        let tools = self.tools.read().await;
        Ok(tools.values().cloned().collect())
    }

    /// Shutdown client
    pub async fn shutdown(&self) -> AofResult<()> {
        info!("Shutting down MCP client");
        let mut transport = self.transport.write().await;
        transport.shutdown().await
    }
}

/// MCP client builder
pub struct McpClientBuilder {
    transport_type: TransportType,
    command: Option<String>,
    args: Vec<String>,
    endpoint: Option<String>,
    env_vars: HashMap<String, String>,
}

impl McpClientBuilder {
    pub fn new() -> Self {
        Self {
            transport_type: TransportType::Stdio,
            command: None,
            args: Vec::new(),
            endpoint: None,
            env_vars: HashMap::new(),
        }
    }

    pub fn stdio(mut self, command: impl Into<String>, args: Vec<String>) -> Self {
        self.transport_type = TransportType::Stdio;
        self.command = Some(command.into());
        self.args = args;
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    #[cfg(feature = "sse")]
    pub fn sse(mut self, endpoint: impl Into<String>) -> Self {
        self.transport_type = TransportType::Sse;
        self.endpoint = Some(endpoint.into());
        self
    }

    #[cfg(feature = "http")]
    pub fn http(mut self, endpoint: impl Into<String>) -> Self {
        self.transport_type = TransportType::Http;
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> AofResult<McpClient> {
        let transport: Box<dyn McpTransport> = match self.transport_type {
            TransportType::Stdio => {
                let command = self.command.ok_or_else(|| AofError::config("Command required for stdio transport"))?;
                let mut stdio_transport = crate::transport::stdio::StdioTransport::new(command, self.args);
                // Add environment variables if provided
                if !self.env_vars.is_empty() {
                    stdio_transport = stdio_transport.with_envs(self.env_vars);
                }
                Box::new(stdio_transport)
            }
            #[cfg(feature = "sse")]
            TransportType::Sse => {
                let endpoint = self.endpoint.ok_or_else(|| AofError::config("Endpoint required for SSE transport"))?;
                Box::new(crate::transport::sse::SseTransport::new(endpoint))
            }
            #[cfg(feature = "http")]
            TransportType::Http => {
                let endpoint = self.endpoint.ok_or_else(|| AofError::config("Endpoint required for HTTP transport"))?;
                Box::new(crate::transport::http::HttpTransport::new(endpoint))
            }
            #[cfg(not(feature = "sse"))]
            TransportType::Sse => {
                return Err(AofError::config("SSE transport not enabled"));
            }
            #[cfg(not(feature = "http"))]
            TransportType::Http => {
                return Err(AofError::config("HTTP transport not enabled"));
            }
        };

        Ok(McpClient::new(transport))
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
