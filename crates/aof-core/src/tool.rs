use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AofError, AofResult};

/// Tool trait - executable capabilities for agents
///
/// Tools can be MCP tools, shell commands, HTTP APIs, etc.
/// Implementations should be lightweight and fast.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;

    /// Tool configuration
    fn config(&self) -> &ToolConfig;

    /// Validate tool input schema
    fn validate_input(&self, _input: &ToolInput) -> AofResult<()> {
        Ok(())
    }

    /// Tool definition for model
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.config().name.clone(),
            description: self.config().description.clone(),
            parameters: self.config().parameters.clone(),
        }
    }
}

/// Tool executor - manages tool execution lifecycle
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult>;

    /// Get all available tools
    fn list_tools(&self) -> Vec<ToolDefinition>;

    /// Get specific tool
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// JSON Schema for parameters
    pub parameters: serde_json::Value,

    /// Tool type (mcp, shell, http, etc.)
    #[serde(default)]
    pub tool_type: ToolType,

    /// Timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Extra configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_timeout() -> u64 {
    30
}

/// Tool type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    #[default]
    Mcp,
    Shell,
    Http,
    Custom,
}

/// Tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,

    /// Context from agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

impl ToolInput {
    /// Create new tool input
    pub fn new(arguments: serde_json::Value) -> Self {
        Self {
            arguments,
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(
        arguments: serde_json::Value,
        context: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            arguments,
            context: Some(context),
        }
    }

    /// Get argument value
    pub fn get_arg<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<T> {
        self.arguments
            .get(key)
            .ok_or_else(|| AofError::tool(format!("Missing argument: {}", key)))
            .and_then(|v| serde_json::from_value(v.clone()).map_err(Into::into))
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success status
    pub success: bool,

    /// Result data (JSON)
    pub data: serde_json::Value,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution time (ms)
    pub execution_time_ms: u64,
}

impl ToolResult {
    /// Create successful result
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            execution_time_ms: 0,
        }
    }

    /// Create error result
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.into()),
            execution_time_ms: 0,
        }
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }
}

/// Tool call (from model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID (for tracking)
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,
}

/// Tool definition (for model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(alias = "inputSchema")]
    pub parameters: serde_json::Value,
}

/// Reference-counted tool
pub type ToolRef = Arc<dyn Tool>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_input_new() {
        let args = serde_json::json!({"path": "/tmp/test.txt"});
        let input = ToolInput::new(args.clone());

        assert_eq!(input.arguments, args);
        assert!(input.context.is_none());
    }

    #[test]
    fn test_tool_input_with_context() {
        let args = serde_json::json!({"command": "ls"});
        let mut context = HashMap::new();
        context.insert("cwd".to_string(), serde_json::json!("/home/user"));

        let input = ToolInput::with_context(args.clone(), context);

        assert_eq!(input.arguments, args);
        assert!(input.context.is_some());
        let ctx = input.context.unwrap();
        assert_eq!(ctx.get("cwd"), Some(&serde_json::json!("/home/user")));
    }

    #[test]
    fn test_tool_input_get_arg() {
        let args = serde_json::json!({
            "name": "test",
            "count": 42,
            "enabled": true
        });
        let input = ToolInput::new(args);

        let name: String = input.get_arg("name").unwrap();
        assert_eq!(name, "test");

        let count: i32 = input.get_arg("count").unwrap();
        assert_eq!(count, 42);

        let enabled: bool = input.get_arg("enabled").unwrap();
        assert!(enabled);
    }

    #[test]
    fn test_tool_input_get_missing_arg() {
        let args = serde_json::json!({});
        let input = ToolInput::new(args);

        let result: AofResult<String> = input.get_arg("missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_result_success() {
        let data = serde_json::json!({"output": "file created"});
        let result = ToolResult::success(data.clone());

        assert!(result.success);
        assert_eq!(result.data, data);
        assert!(result.error.is_none());
        assert_eq!(result.execution_time_ms, 0);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("file not found");

        assert!(!result.success);
        assert_eq!(result.data, serde_json::Value::Null);
        assert_eq!(result.error, Some("file not found".to_string()));
    }

    #[test]
    fn test_tool_result_with_execution_time() {
        let result = ToolResult::success(serde_json::json!({}))
            .with_execution_time(150);

        assert_eq!(result.execution_time_ms, 150);
    }

    #[test]
    fn test_tool_config_serialization() {
        let config = ToolConfig {
            name: "read_file".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"]
            }),
            tool_type: ToolType::Shell,
            timeout_secs: 30,
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("read_file"));

        let deserialized: ToolConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "read_file");
        assert_eq!(deserialized.tool_type, ToolType::Shell);
    }

    #[test]
    fn test_tool_type_default() {
        let config: ToolConfig = serde_json::from_str(r#"{
            "name": "test",
            "description": "test tool",
            "parameters": {}
        }"#).unwrap();

        assert_eq!(config.tool_type, ToolType::Mcp); // default
    }

    #[test]
    fn test_tool_call_serialization() {
        let call = ToolCall {
            id: "call_123".to_string(),
            name: "write_file".to_string(),
            arguments: serde_json::json!({"path": "/tmp/out.txt", "content": "hello"}),
        };

        let json = serde_json::to_string(&call).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "call_123");
        assert_eq!(deserialized.name, "write_file");
    }

    #[test]
    fn test_tool_definition_serialization() {
        let def = ToolDefinition {
            name: "execute_shell".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("execute_shell"));
        assert!(json.contains("Execute a shell command"));
    }
}
