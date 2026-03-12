//! Integration tests for tool executor flow
//!
//! Tests the complete flow of tool definition, tool execution,
//! and error handling without requiring actual MCP server.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    struct ToolExecutorTest {
        tool_definitions: HashMap<String, ToolDef>,
        execution_log: Vec<ExecutionRecord>,
    }

    #[derive(Clone, Debug)]
    struct ToolDef {
        name: String,
        description: String,
        parameters: serde_json::Value,
    }

    #[derive(Clone, Debug)]
    struct ExecutionRecord {
        tool_name: String,
        arguments: serde_json::Value,
        success: bool,
        error_message: Option<String>,
    }

    impl ToolExecutorTest {
        fn new() -> Self {
            Self {
                tool_definitions: HashMap::new(),
                execution_log: Vec::new(),
            }
        }

        fn register_tool(&mut self, name: &str, description: &str, parameters: serde_json::Value) {
            self.tool_definitions.insert(
                name.to_string(),
                ToolDef {
                    name: name.to_string(),
                    description: description.to_string(),
                    parameters,
                },
            );
        }

        async fn execute_tool(
            &mut self,
            name: &str,
            arguments: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            // Check if tool is registered
            if !self.tool_definitions.contains_key(name) {
                let error = format!("Tool '{}' not found", name);
                self.execution_log.push(ExecutionRecord {
                    tool_name: name.to_string(),
                    arguments,
                    success: false,
                    error_message: Some(error.clone()),
                });
                return Err(error);
            }

            // Execute tool
            let result = self.execute_tool_impl(name, &arguments).await;

            let (success, error_message) = match &result {
                Ok(_) => (true, None),
                Err(e) => (false, Some(e.clone())),
            };

            self.execution_log.push(ExecutionRecord {
                tool_name: name.to_string(),
                arguments,
                success,
                error_message,
            });

            result
        }

        async fn execute_tool_impl(
            &self,
            name: &str,
            arguments: &serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            match name {
                "shell" => Ok(serde_json::json!({
                    "output": "Command executed successfully",
                    "exit_code": 0,
                    "args": arguments
                })),
                "kubectl" => {
                    if arguments.get("command").is_some() {
                        Ok(serde_json::json!({
                            "output": "kubectl command executed",
                            "args": arguments
                        }))
                    } else {
                        Err("Missing 'command' argument for kubectl".to_string())
                    }
                }
                _ => Err(format!("Tool '{}' execution not implemented", name)),
            }
        }

        fn get_execution_log(&self) -> &[ExecutionRecord] {
            &self.execution_log
        }
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let mut executor = ToolExecutorTest::new();

        executor.register_tool(
            "kubectl",
            "Execute Kubernetes commands",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                }
            }),
        );

        assert!(executor.tool_definitions.contains_key("kubectl"));
    }

    #[tokio::test]
    async fn test_unregistered_tool_fails() {
        let mut executor = ToolExecutorTest::new();

        let result = executor
            .execute_tool(
                "nonexistent_tool",
                serde_json::json!({"arg": "value"}),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tool 'nonexistent_tool' not found");
    }

    #[tokio::test]
    async fn test_tool_execution_success() {
        let mut executor = ToolExecutorTest::new();

        executor.register_tool(
            "shell",
            "Execute shell commands",
            serde_json::json!({"type": "object"}),
        );

        let result = executor
            .execute_tool("shell", serde_json::json!({"command": "echo hello"}))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["output"], "Command executed successfully");
    }

    #[tokio::test]
    async fn test_tool_execution_with_missing_params() {
        let mut executor = ToolExecutorTest::new();

        executor.register_tool(
            "kubectl",
            "Execute Kubernetes commands",
            serde_json::json!({
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": {"type": "string"}
                }
            }),
        );

        let result = executor
            .execute_tool("kubectl", serde_json::json!({})) // Missing command
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing 'command' argument for kubectl");
    }

    #[tokio::test]
    async fn test_execution_log_tracking() {
        let mut executor = ToolExecutorTest::new();

        executor.register_tool("shell", "Execute shell commands", serde_json::json!({}));
        executor.register_tool("kubectl", "Kubernetes commands", serde_json::json!({}));

        // Execute some tools
        let _ = executor.execute_tool("shell", serde_json::json!({"cmd": "ls"})).await;
        let _ = executor
            .execute_tool("kubectl", serde_json::json!({"command": "get pods"}))
            .await;
        let _ = executor.execute_tool("nonexistent", serde_json::json!({})).await;

        let log = executor.get_execution_log();
        assert_eq!(log.len(), 3);

        // Verify log entries
        assert_eq!(log[0].tool_name, "shell");
        assert!(log[0].success);

        assert_eq!(log[1].tool_name, "kubectl");
        assert!(log[1].success);

        assert_eq!(log[2].tool_name, "nonexistent");
        assert!(!log[2].success);
    }

    #[tokio::test]
    async fn test_tool_execution_error_handling() {
        let mut executor = ToolExecutorTest::new();

        executor.register_tool("kubectl", "Kubernetes commands", serde_json::json!({}));

        // Test success case
        let result = executor
            .execute_tool("kubectl", serde_json::json!({"command": "get pods"}))
            .await;
        assert!(result.is_ok());

        // Test failure case
        let result = executor
            .execute_tool("kubectl", serde_json::json!({"wrong_param": "value"}))
            .await;
        assert!(result.is_err());

        // Verify both are logged
        let log = executor.get_execution_log();
        assert_eq!(log.len(), 2);
        assert!(log[0].success);
        assert!(!log[1].success);
    }
}
