//! Unit tests for aof-core tool traits

use aof_core::{
    AofError, AofResult, Tool, ToolCall, ToolConfig, ToolDefinition, ToolExecutor, ToolInput,
    ToolResult, ToolType,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock tool for testing
struct MockTool {
    config: ToolConfig,
    should_fail: bool,
}

impl MockTool {
    fn new(name: &str, should_fail: bool) -> Self {
        Self {
            config: ToolConfig {
                name: name.to_string(),
                description: format!("Test tool: {}", name),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    }
                }),
                tool_type: ToolType::Custom,
                timeout_secs: 30,
                extra: HashMap::new(),
            },
            should_fail,
        }
    }
}

#[async_trait]
impl Tool for MockTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        if self.should_fail {
            return Ok(ToolResult::error("Mock tool failure"));
        }

        let input_str: String = input.get_arg("input").unwrap_or_else(|_| "default".to_string());

        Ok(ToolResult::success(serde_json::json!({
            "output": format!("Processed: {}", input_str)
        })).with_execution_time(100))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Mock tool executor for testing
struct MockToolExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl MockToolExecutor {
    fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.config().name.clone(), tool);
    }
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| AofError::tool(format!("Tool not found: {}", name)))?;

        tool.execute(input).await
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| t.definition())
            .collect()
    }

    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
}

#[tokio::test]
async fn test_tool_execute_success() {
    let tool = MockTool::new("test_tool", false);
    let input = ToolInput::new(serde_json::json!({"input": "hello"}));

    let result = tool.execute(input).await.unwrap();

    assert!(result.success);
    assert_eq!(result.data, serde_json::json!({"output": "Processed: hello"}));
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_tool_execute_failure() {
    let tool = MockTool::new("failing_tool", true);
    let input = ToolInput::new(serde_json::json!({}));

    let result = tool.execute(input).await.unwrap();

    assert!(!result.success);
    assert_eq!(result.error, Some("Mock tool failure".to_string()));
}

#[tokio::test]
async fn test_tool_input_get_arg() {
    let input = ToolInput::new(serde_json::json!({
        "name": "test",
        "count": 42,
        "enabled": true
    }));

    let name: String = input.get_arg("name").unwrap();
    assert_eq!(name, "test");

    let count: i32 = input.get_arg("count").unwrap();
    assert_eq!(count, 42);

    let enabled: bool = input.get_arg("enabled").unwrap();
    assert!(enabled);
}

#[tokio::test]
async fn test_tool_input_missing_arg() {
    let input = ToolInput::new(serde_json::json!({}));
    let result: AofResult<String> = input.get_arg("missing");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_executor_execute() {
    let mut executor = MockToolExecutor::new();
    let tool = Arc::new(MockTool::new("echo", false));
    executor.register_tool(tool);

    let input = ToolInput::new(serde_json::json!({"input": "test"}));
    let result = executor.execute_tool("echo", input).await.unwrap();

    assert!(result.success);
    assert_eq!(result.data, serde_json::json!({"output": "Processed: test"}));
}

#[tokio::test]
async fn test_tool_executor_tool_not_found() {
    let executor = MockToolExecutor::new();
    let input = ToolInput::new(serde_json::json!({}));

    let result = executor.execute_tool("nonexistent", input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_executor_list_tools() {
    let mut executor = MockToolExecutor::new();
    executor.register_tool(Arc::new(MockTool::new("tool1", false)));
    executor.register_tool(Arc::new(MockTool::new("tool2", false)));

    let tools = executor.list_tools();
    assert_eq!(tools.len(), 2);

    let names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(names.contains(&"tool1".to_string()));
    assert!(names.contains(&"tool2".to_string()));
}

#[tokio::test]
async fn test_tool_executor_get_tool() {
    let mut executor = MockToolExecutor::new();
    let tool = Arc::new(MockTool::new("get_test", false));
    executor.register_tool(tool);

    let retrieved = executor.get_tool("get_test");
    assert!(retrieved.is_some());

    let retrieved_tool = retrieved.unwrap();
    assert_eq!(retrieved_tool.config().name, "get_test");
}

#[test]
fn test_tool_result_success() {
    let result = ToolResult::success(serde_json::json!({"status": "ok"}));
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn test_tool_result_error() {
    let result = ToolResult::error("Something went wrong");
    assert!(!result.success);
    assert_eq!(result.error, Some("Something went wrong".to_string()));
    assert_eq!(result.data, serde_json::Value::Null);
}

#[test]
fn test_tool_result_with_execution_time() {
    let result = ToolResult::success(serde_json::json!({}))
        .with_execution_time(250);
    assert_eq!(result.execution_time_ms, 250);
}

#[test]
fn test_tool_call_serialization() {
    let call = ToolCall {
        id: "call_123".to_string(),
        name: "test_tool".to_string(),
        arguments: serde_json::json!({"arg": "value"}),
    };

    let json = serde_json::to_string(&call).unwrap();
    let deserialized: ToolCall = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "call_123");
    assert_eq!(deserialized.name, "test_tool");
}

#[test]
fn test_tool_config_serialization() {
    let config = ToolConfig {
        name: "serialize_test".to_string(),
        description: "Test tool".to_string(),
        parameters: serde_json::json!({"type": "object"}),
        tool_type: ToolType::Mcp,
        timeout_secs: 60,
        extra: HashMap::new(),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ToolConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "serialize_test");
    assert_eq!(deserialized.tool_type, ToolType::Mcp);
    assert_eq!(deserialized.timeout_secs, 60);
}

#[test]
fn test_tool_type_variants() {
    assert_eq!(ToolType::default(), ToolType::Mcp);

    let types = vec![ToolType::Mcp, ToolType::Shell, ToolType::Http, ToolType::Custom];
    for tool_type in types {
        let json = serde_json::to_string(&tool_type).unwrap();
        let deserialized: ToolType = serde_json::from_str(&json).unwrap();
        assert_eq!(tool_type, deserialized);
    }
}
