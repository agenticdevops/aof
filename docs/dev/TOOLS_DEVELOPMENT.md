# Tools Development Guide

This guide explains how to develop and extend the AOF tool system.

## Overview

AOF's tool system is modular and extensible. Tools can be:
1. **Built-in** - Rust implementations in `aof-tools`
2. **MCP** - External MCP server tools
3. **Custom** - User-defined tools

## Creating a Built-in Tool

### 1. Create the Tool Struct

```rust
use aof_core::{Tool, ToolConfig, ToolInput, ToolResult, AofResult, ToolType};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MyTool {
    config: ToolConfig,
}

impl MyTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig {
                name: "my_tool".to_string(),
                description: "Description of what this tool does".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": {
                            "type": "string",
                            "description": "First parameter"
                        },
                        "param2": {
                            "type": "integer",
                            "description": "Second parameter"
                        }
                    },
                    "required": ["param1"]
                }),
                tool_type: ToolType::Custom,
                timeout_secs: 30,
                extra: HashMap::new(),
            },
        }
    }
}
```

### 2. Implement the Tool Trait

```rust
#[async_trait]
impl Tool for MyTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        // Extract parameters
        let param1: String = input.get_arg("param1")?;
        let param2: Option<i32> = input.get_arg("param2").ok();

        // Execute tool logic
        let result = do_something(param1, param2).await?;

        // Return result
        Ok(ToolResult::success(serde_json::json!({
            "output": result
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 3. Add to Tool Category Module

In `src/tools/mod.rs`:

```rust
#[cfg(feature = "my_category")]
pub mod my_category;

#[cfg(feature = "my_category")]
pub use my_category::*;
```

### 4. Add Feature Flag

In `Cargo.toml`:

```toml
[features]
my_category = []
all = ["file", "shell", ..., "my_category"]
```

## CLI Wrapper Tools

For tools that wrap system CLIs (kubectl, docker, git, etc.):

### Common Pattern

```rust
use super::common::{execute_command, CommandOutput};

pub struct KubectlGetTool {
    config: ToolConfig,
}

#[async_trait]
impl Tool for KubectlGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let resource: String = input.get_arg("resource")?;
        let namespace: Option<String> = input.get_arg("namespace").ok();

        // Build command arguments
        let mut args = vec!["get", &resource];
        if let Some(ns) = &namespace {
            args.push("-n");
            args.push(ns);
        }

        // Execute command
        let output = execute_command("kubectl", &args, None, 60).await?;

        if output.success {
            Ok(ToolResult::success(serde_json::json!({
                "stdout": output.stdout,
                "stderr": output.stderr
            })))
        } else {
            Ok(ToolResult::error(format!(
                "kubectl failed: {}",
                output.stderr
            )))
        }
    }
}
```

### Command Execution Helper

```rust
pub async fn execute_command(
    program: &str,
    args: &[&str],
    working_dir: Option<&str>,
    timeout_secs: u64,
) -> Result<CommandOutput, String> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn: {}", e))?;

    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        child.wait_with_output()
    ).await
        .map_err(|_| "Command timed out".to_string())?
        .map_err(|e| format!("Command failed: {}", e))?;

    Ok(CommandOutput {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
```

## Tool Registry

### Registering Tools

```rust
use aof_tools::{ToolRegistry, BuiltinToolExecutor};

// Create registry with default tools
let registry = ToolRegistry::with_defaults();

// Or create custom registry
let mut registry = ToolRegistry::new();
registry.register(MyTool::new());
registry.register(AnotherTool::new());

// Convert to executor
let executor: Arc<dyn ToolExecutor> = Arc::new(registry.into_executor());
```

### Category Registration

```rust
// Register by category
registry.register_with_category(MyTool::new(), ToolCategory::Custom);

// List tools by category
let tools = registry.list_by_category(ToolCategory::Kubectl);
```

## Composite Executor

Combine multiple executors (built-in + MCP):

```rust
use aof_tools::CompositeToolExecutor;

let builtin_executor = registry.into_executor();
let mcp_executor = mcp_client.into_executor();

let composite = CompositeToolExecutor::new(vec![
    Box::new(builtin_executor),
    Box::new(mcp_executor),
]);
```

## Testing Tools

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_tool() {
        let tool = MyTool::new();

        let input = ToolInput::new(serde_json::json!({
            "param1": "test_value"
        }));

        let result = tool.execute(input).await.unwrap();

        assert!(result.success);
        assert!(result.data.get("output").is_some());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_tool_with_registry() {
    let mut registry = ToolRegistry::new();
    registry.register(MyTool::new());

    let executor = registry.into_executor();

    let input = ToolInput::new(serde_json::json!({
        "param1": "test"
    }));

    let result = executor.execute_tool("my_tool", input).await.unwrap();
    assert!(result.success);
}
```

## Best Practices

### 1. Parameter Validation

```rust
fn validate_input(&self, input: &ToolInput) -> AofResult<()> {
    // Validate required parameters
    let _param: String = input.get_arg("required_param")?;

    // Validate constraints
    let value: i32 = input.get_arg("number").unwrap_or(10);
    if value < 0 || value > 100 {
        return Err(AofError::tool("number must be 0-100"));
    }

    Ok(())
}
```

### 2. Timeout Handling

```rust
async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
    let timeout = input.get_arg("timeout_secs")
        .unwrap_or(self.config.timeout_secs);

    tokio::time::timeout(
        Duration::from_secs(timeout),
        self.do_work(&input)
    ).await
        .map_err(|_| AofError::tool("Operation timed out"))?
}
```

### 3. Error Messages

```rust
// Good - specific and actionable
Ok(ToolResult::error(format!(
    "kubectl get failed: {} (namespace: {}, resource: {})",
    stderr, namespace, resource
)))

// Bad - vague
Ok(ToolResult::error("Command failed"))
```

### 4. JSON Schema

Use proper JSON Schema for parameters:

```rust
parameters: serde_json::json!({
    "type": "object",
    "properties": {
        "path": {
            "type": "string",
            "description": "File path to read"
        },
        "encoding": {
            "type": "string",
            "description": "File encoding",
            "default": "utf-8",
            "enum": ["utf-8", "ascii", "latin1"]
        }
    },
    "required": ["path"],
    "additionalProperties": false
})
```

### 5. Security Considerations

```rust
// Validate paths
fn validate_path(path: &str) -> AofResult<()> {
    let path = Path::new(path);
    if path.components().any(|c| c == Component::ParentDir) {
        return Err(AofError::tool("Path traversal not allowed"));
    }
    Ok(())
}

// Block dangerous commands
const BLOCKED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "dd if=/dev",
];

fn is_safe_command(cmd: &str) -> bool {
    !BLOCKED_COMMANDS.iter().any(|b| cmd.contains(b))
}
```

## Adding New Tool Categories

1. Create module in `src/tools/`
2. Implement tools following patterns above
3. Add feature flag in `Cargo.toml`
4. Export from `src/tools/mod.rs`
5. Add to `ToolCategory` enum if needed
6. Update documentation in `docs/user/tools/index.md`
7. Create example agents demonstrating usage

## Documentation Requirements

Every tool should have:
1. Clear description
2. Parameter documentation with types
3. Example usage in JSON
4. MCP alternative (if applicable)
5. Tests covering happy path and error cases
