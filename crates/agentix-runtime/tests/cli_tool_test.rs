//! Tests for the CLI tool executor.
//! TDD: these tests define required behavior before implementation.

use agentix_core::{DirectoryToolType, ToolEntry};
use agentix_runtime::tools::CliToolExecutor;
use agentix_runtime::executor::react_loop::ToolExecutor;

fn make_echo_tool() -> ToolEntry {
    ToolEntry {
        name: "echo-test".to_string(),
        tool_type: DirectoryToolType::Shell,
        command: Some("echo".to_string()),
        description: Some("Echo test tool".to_string()),
        server: None,
        args: vec!["{{message}}".to_string()],
    }
}

fn make_cli_tool(cmd: &str) -> ToolEntry {
    ToolEntry {
        name: "cli-test".to_string(),
        tool_type: DirectoryToolType::Cli,
        command: Some(cmd.to_string()),
        description: Some("CLI test tool".to_string()),
        server: None,
        args: vec![],
    }
}

fn make_shell_tool(script: &str) -> ToolEntry {
    ToolEntry {
        name: "shell-test".to_string(),
        tool_type: DirectoryToolType::Shell,
        command: Some(script.to_string()),
        description: Some("Shell test tool".to_string()),
        server: None,
        args: vec![],
    }
}

#[tokio::test]
async fn test_echo_tool_substitutes_input() {
    let executor = CliToolExecutor::new();
    let tool = make_echo_tool();
    let input = serde_json::json!({"message": "hello-world"});
    let result = executor.execute(&tool, input).await;
    assert!(result.is_ok(), "echo should succeed: {:?}", result.err());
    let output = result.unwrap();
    assert!(
        output.contains("hello-world"),
        "Expected 'hello-world' in output, got: {}",
        output
    );
}

#[tokio::test]
async fn test_shell_tool_runs_command() {
    let executor = CliToolExecutor::new();
    let tool = make_shell_tool("echo 'tool-output'");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("tool-output"), "Got: {}", output);
}

#[tokio::test]
async fn test_cli_tool_success() {
    let executor = CliToolExecutor::new();
    // 'true' always succeeds
    let tool = make_cli_tool("true");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_ok(), "Expected Ok from 'true': {:?}", result.err());
}

#[tokio::test]
async fn test_cli_tool_failure_returns_err_string() {
    let executor = CliToolExecutor::new();
    // 'false' always exits non-zero
    let tool = make_cli_tool("false");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    // Tool failures return Err(String) — error message fed back as observation
    assert!(
        result.is_err(),
        "Expected Err from 'false' command, got Ok"
    );
}

#[tokio::test]
async fn test_missing_command_returns_err() {
    let executor = CliToolExecutor::new();
    let tool = ToolEntry {
        name: "no-command".to_string(),
        tool_type: DirectoryToolType::Cli,
        command: None,
        description: None,
        server: None,
        args: vec![],
    };
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("no command"), "Expected 'no command' in error, got: {}", err);
}

#[tokio::test]
async fn test_template_substitution_with_multiple_vars() {
    let executor = CliToolExecutor::new();
    let tool = ToolEntry {
        name: "multi-var".to_string(),
        tool_type: DirectoryToolType::Shell,
        command: Some("echo {{first}} {{second}}".to_string()),
        description: None,
        server: None,
        args: vec![],
    };
    let input = serde_json::json!({"first": "hello", "second": "world"});
    let result = executor.execute(&tool, input).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("hello"), "Got: {}", output);
    assert!(output.contains("world"), "Got: {}", output);
}

#[tokio::test]
async fn test_mcp_tool_returns_not_implemented_err() {
    let executor = CliToolExecutor::new();
    let tool = ToolEntry {
        name: "mcp-tool".to_string(),
        tool_type: DirectoryToolType::Mcp,
        command: None,
        description: None,
        server: Some("my-server".to_string()),
        args: vec![],
    };
    let result = executor.execute(&tool, serde_json::json!({})).await;
    // MCP not yet wired — should return a descriptive error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("mcp") || err.contains("Phase 14"),
        "Expected MCP-related error message, got: {}",
        err
    );
}

#[tokio::test]
async fn test_timeout_kills_long_running_command() {
    let executor = CliToolExecutor::with_timeout(1); // 1 second timeout
    let tool = make_shell_tool("sleep 10");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_err(), "Expected timeout error, got Ok");
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("timeout") || err.to_lowercase().contains("timed out"),
        "Expected timeout message, got: {}",
        err
    );
}
