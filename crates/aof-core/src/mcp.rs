//! MCP (Model Context Protocol) configuration types
//!
//! This module provides configuration types for MCP servers that can be
//! specified in agent YAML configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server configuration
///
/// Supports three transport types:
/// - `stdio`: Local process communication via stdin/stdout
/// - `sse`: Server-Sent Events over HTTP
/// - `http`: HTTP request/response transport
///
/// # Example YAML (stdio)
/// ```yaml
/// mcp_servers:
///   - name: filesystem
///     transport: stdio
///     command: npx
///     args:
///       - "@anthropic-ai/mcp-server-fs"
///     env:
///       MCP_FS_ROOT: /workspace
///
/// # Example YAML (sse)
/// ```yaml
/// mcp_servers:
///   - name: remote-tools
///     transport: sse
///     endpoint: http://localhost:3000/mcp
///
/// # Example YAML (http)
/// ```yaml
/// mcp_servers:
///   - name: api-tools
///     transport: http
///     endpoint: http://localhost:8080/mcp/v1
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (for logging and identification)
    pub name: String,

    /// Transport type: stdio, sse, http
    #[serde(default = "default_transport")]
    pub transport: McpTransport,

    /// Command to execute (for stdio transport)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Arguments for the command (for stdio transport)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// Environment variables (for stdio transport)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Endpoint URL (for sse/http transport)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Optional tools filter - only use these tools from this server
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,

    /// Server initialization options (passed to MCP initialize)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_options: Option<serde_json::Value>,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Whether to automatically reconnect on failure
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,
}

/// MCP transport type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    /// Standard I/O transport (stdin/stdout with local process)
    #[default]
    Stdio,
    /// Server-Sent Events transport
    Sse,
    /// HTTP request/response transport
    Http,
}

fn default_transport() -> McpTransport {
    McpTransport::Stdio
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

impl McpServerConfig {
    /// Create a new stdio-based MCP server config
    pub fn stdio(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Stdio,
            command: Some(command.into()),
            args: Vec::new(),
            env: HashMap::new(),
            endpoint: None,
            tools: Vec::new(),
            init_options: None,
            timeout_secs: default_timeout(),
            auto_reconnect: true,
        }
    }

    /// Create a new SSE-based MCP server config
    pub fn sse(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Sse,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            endpoint: Some(endpoint.into()),
            tools: Vec::new(),
            init_options: None,
            timeout_secs: default_timeout(),
            auto_reconnect: true,
        }
    }

    /// Create a new HTTP-based MCP server config
    pub fn http(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Http,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            endpoint: Some(endpoint.into()),
            tools: Vec::new(),
            init_options: None,
            timeout_secs: default_timeout(),
            auto_reconnect: true,
        }
    }

    /// Add arguments to the config
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add environment variables to the config
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Filter to specific tools
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    /// Set initialization options
    pub fn with_init_options(mut self, options: serde_json::Value) -> Self {
        self.init_options = Some(options);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        match self.transport {
            McpTransport::Stdio => {
                if self.command.is_none() {
                    return Err("Stdio transport requires 'command' field".to_string());
                }
            }
            McpTransport::Sse | McpTransport::Http => {
                if self.endpoint.is_none() {
                    return Err(format!(
                        "{:?} transport requires 'endpoint' field",
                        self.transport
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_config_creation() {
        let config = McpServerConfig::stdio("test-server", "npx")
            .with_args(vec!["@anthropic-ai/mcp-server-fs".to_string()])
            .with_env_var("MCP_FS_ROOT", "/workspace");

        assert_eq!(config.name, "test-server");
        assert_eq!(config.transport, McpTransport::Stdio);
        assert_eq!(config.command, Some("npx".to_string()));
        assert_eq!(config.args.len(), 1);
        assert_eq!(config.env.get("MCP_FS_ROOT"), Some(&"/workspace".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sse_config_creation() {
        let config = McpServerConfig::sse("remote-server", "http://localhost:3000/mcp");

        assert_eq!(config.name, "remote-server");
        assert_eq!(config.transport, McpTransport::Sse);
        assert_eq!(config.endpoint, Some("http://localhost:3000/mcp".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_http_config_creation() {
        let config = McpServerConfig::http("api-server", "http://localhost:8080/mcp/v1");

        assert_eq!(config.name, "api-server");
        assert_eq!(config.transport, McpTransport::Http);
        assert_eq!(config.endpoint, Some("http://localhost:8080/mcp/v1".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_stdio_config_without_command_fails_validation() {
        let config = McpServerConfig {
            name: "test".to_string(),
            transport: McpTransport::Stdio,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            endpoint: None,
            tools: Vec::new(),
            init_options: None,
            timeout_secs: 30,
            auto_reconnect: true,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_yaml_deserialization_stdio() {
        let yaml = r#"
            name: filesystem
            transport: stdio
            command: npx
            args:
              - "@anthropic-ai/mcp-server-fs"
            env:
              MCP_FS_ROOT: /workspace
        "#;

        let config: McpServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "filesystem");
        assert_eq!(config.transport, McpTransport::Stdio);
        assert_eq!(config.command, Some("npx".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_yaml_deserialization_sse() {
        let yaml = r#"
            name: remote-tools
            transport: sse
            endpoint: http://localhost:3000/mcp
        "#;

        let config: McpServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "remote-tools");
        assert_eq!(config.transport, McpTransport::Sse);
        assert_eq!(config.endpoint, Some("http://localhost:3000/mcp".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_yaml_deserialization_with_init_options() {
        let yaml = r#"
            name: advanced-server
            transport: stdio
            command: ./mcp-server
            init_options:
              debug: true
              workspace: /home/user
        "#;

        let config: McpServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "advanced-server");
        assert!(config.init_options.is_some());
        let opts = config.init_options.unwrap();
        assert_eq!(opts.get("debug"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn test_default_transport() {
        let yaml = r#"
            name: default-transport
            command: ./server
        "#;

        let config: McpServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.transport, McpTransport::Stdio);
    }
}
