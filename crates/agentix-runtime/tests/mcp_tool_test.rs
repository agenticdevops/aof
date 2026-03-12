//! Tests for the MCP tool executor and CompositeToolExecutor.
//! TDD: these tests define required behavior before implementation.

use agentix_core::{DirectoryToolType, McpServerConfig, ToolEntry};
use agentix_runtime::tools::{CliToolExecutor, CompositeToolExecutor, McpToolExecutor};
use agentix_runtime::executor::react_loop::ToolExecutor;

fn make_mcp_tool(server: &str) -> ToolEntry {
    ToolEntry {
        name: "mcp-tool".to_string(),
        tool_type: DirectoryToolType::Mcp,
        command: None,
        description: Some("MCP test tool".to_string()),
        server: Some(server.to_string()),
        args: vec![],
    }
}

fn make_shell_tool() -> ToolEntry {
    ToolEntry {
        name: "echo-tool".to_string(),
        tool_type: DirectoryToolType::Shell,
        command: Some("echo test".to_string()),
        description: None,
        server: None,
        args: vec![],
    }
}

#[tokio::test]
async fn test_mcp_executor_missing_server_returns_error() {
    // No servers configured — tool call should return descriptive error
    let executor = McpToolExecutor::new(vec![]);
    let tool = make_mcp_tool("nonexistent-server");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_err(), "Expected error for missing MCP server");
    let err = result.unwrap_err();
    assert!(
        err.contains("nonexistent-server") || err.to_lowercase().contains("not found") || err.to_lowercase().contains("not configured"),
        "Expected server name in error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_composite_executor_routes_shell_to_cli() {
    // CompositeToolExecutor should handle CLI/Shell tools via CliToolExecutor
    let cli = CliToolExecutor::new();
    let mcp = McpToolExecutor::new(vec![]);
    let executor = CompositeToolExecutor::new(cli, mcp);
    let tool = make_shell_tool();
    let result = executor.execute(&tool, serde_json::json!({})).await;
    assert!(result.is_ok(), "Shell tool via composite should work: {:?}", result.err());
    assert!(result.unwrap().contains("test"), "Should contain 'test' from echo");
}

#[tokio::test]
async fn test_composite_executor_routes_mcp_to_mcp_executor() {
    // MCP tools get routed to McpToolExecutor (returns error since no server configured)
    let cli = CliToolExecutor::new();
    let mcp = McpToolExecutor::new(vec![]);
    let executor = CompositeToolExecutor::new(cli, mcp);
    let tool = make_mcp_tool("some-server");
    let result = executor.execute(&tool, serde_json::json!({})).await;
    // Should get an error from McpToolExecutor (not from CliToolExecutor)
    assert!(result.is_err(), "Should error when MCP server not found");
}

#[tokio::test]
async fn test_mcp_executor_created_from_server_configs() {
    // Test that McpToolExecutor can be constructed from a Vec of McpServerConfig
    let configs = vec![
        McpServerConfig::stdio("test-server", "echo"),
    ];
    let executor = McpToolExecutor::new(configs);
    // Just confirm construction works
    let _ = executor;
}

#[tokio::test]
async fn test_mcp_executor_cli_tool_returns_wrong_type_error() {
    // McpToolExecutor should not handle CLI tools — return a type mismatch error
    let executor = McpToolExecutor::new(vec![]);
    let cli_tool = make_shell_tool();
    let result = executor.execute(&cli_tool, serde_json::json!({})).await;
    assert!(result.is_err(), "McpToolExecutor should not execute shell tools");
}
