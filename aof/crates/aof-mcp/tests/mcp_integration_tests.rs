//! MCP Integration Tests
//!
//! Tests the MCP client with the smoke-test-mcp server.
//! Run the full test suite with: cargo test -p aof-mcp

use aof_mcp::McpClientBuilder;
use serde_json::json;
use std::path::PathBuf;

/// Find the smoke-test-mcp binary
fn find_smoke_test_mcp() -> Option<String> {
    let possible_paths = vec![
        // Release build
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/release/smoke-test-mcp"),
        // Debug build
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/debug/smoke-test-mcp"),
    ];

    for path in possible_paths {
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}

#[tokio::test]
async fn test_mcp_client_builder() {
    // Test builder patterns
    let result = McpClientBuilder::new()
        .stdio("echo", vec!["test".to_string()])
        .build();

    assert!(result.is_ok(), "Should create client with builder");
}

#[tokio::test]
async fn test_mcp_client_with_smoke_test_server() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    // Initialize
    let init_result = client.initialize().await;
    assert!(init_result.is_ok(), "Initialize should succeed: {:?}", init_result.err());

    // List tools
    let tools = client.list_tools().await.expect("list_tools should succeed");
    assert!(!tools.is_empty(), "Should have tools available");

    // Verify expected tools exist
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"echo"), "Should have echo tool");
    assert!(tool_names.contains(&"add"), "Should have add tool");
    assert!(tool_names.contains(&"kv_set"), "Should have kv_set tool");

    // Test echo tool
    let echo_result = client.call_tool("echo", json!({"message": "hello from test"})).await;
    assert!(echo_result.is_ok(), "Echo should succeed: {:?}", echo_result.err());

    // Test add tool
    let add_result = client.call_tool("add", json!({"a": 10, "b": 5})).await;
    assert!(add_result.is_ok(), "Add should succeed: {:?}", add_result.err());

    // Shutdown
    let shutdown_result = client.shutdown().await;
    assert!(shutdown_result.is_ok(), "Shutdown should succeed");
}

#[tokio::test]
async fn test_mcp_kv_operations() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // Set a key
    let set_result = client
        .call_tool("kv_set", json!({"key": "test_key", "value": "test_value"}))
        .await;
    assert!(set_result.is_ok(), "kv_set should succeed");

    // Get the key
    let get_result = client
        .call_tool("kv_get", json!({"key": "test_key"}))
        .await;
    assert!(get_result.is_ok(), "kv_get should succeed");

    // List keys
    let list_result = client.call_tool("kv_list", json!({})).await;
    assert!(list_result.is_ok(), "kv_list should succeed");

    // Delete the key
    let delete_result = client
        .call_tool("kv_delete", json!({"key": "test_key"}))
        .await;
    assert!(delete_result.is_ok(), "kv_delete should succeed");

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_file_operations() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // List existing files (pre-populated)
    let list_result = client.call_tool("file_list", json!({})).await;
    assert!(list_result.is_ok(), "file_list should succeed");

    // Read a pre-populated file
    let read_result = client
        .call_tool("file_read", json!({"path": "/tmp/test.txt"}))
        .await;
    assert!(read_result.is_ok(), "file_read should succeed");

    // Write a new file
    let write_result = client
        .call_tool(
            "file_write",
            json!({"path": "/tmp/new_file.txt", "content": "Hello from integration test!"}),
        )
        .await;
    assert!(write_result.is_ok(), "file_write should succeed");

    // Read it back
    let read_back = client
        .call_tool("file_read", json!({"path": "/tmp/new_file.txt"}))
        .await;
    assert!(read_back.is_ok(), "file_read should succeed for new file");

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_batch_processing() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // Test uppercase batch
    let upper_result = client
        .call_tool(
            "batch_process",
            json!({
                "items": ["hello", "world", "test"],
                "operation": "uppercase"
            }),
        )
        .await;
    assert!(upper_result.is_ok(), "batch_process uppercase should succeed");

    // Test reverse batch
    let reverse_result = client
        .call_tool(
            "batch_process",
            json!({
                "items": ["abc", "def"],
                "operation": "reverse"
            }),
        )
        .await;
    assert!(reverse_result.is_ok(), "batch_process reverse should succeed");

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_json_transform() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // Test keys operation
    let keys_result = client
        .call_tool(
            "json_transform",
            json!({
                "data": {"name": "test", "value": 42, "nested": {"a": 1}},
                "operation": "keys"
            }),
        )
        .await;
    assert!(keys_result.is_ok(), "json_transform keys should succeed");

    // Test flatten operation
    let flatten_result = client
        .call_tool(
            "json_transform",
            json!({
                "data": {"level1": {"level2": {"value": "deep"}}},
                "operation": "flatten"
            }),
        )
        .await;
    assert!(flatten_result.is_ok(), "json_transform flatten should succeed");

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_delay_tool() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // Test short delay
    let start = std::time::Instant::now();
    let delay_result = client.call_tool("delay", json!({"ms": 100})).await;
    let elapsed = start.elapsed();

    assert!(delay_result.is_ok(), "delay should succeed");
    assert!(
        elapsed.as_millis() >= 100,
        "Should wait at least 100ms, elapsed: {}ms",
        elapsed.as_millis()
    );

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_system_info() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    let result = client.call_tool("get_system_info", json!({})).await;
    assert!(result.is_ok(), "get_system_info should succeed");

    client.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_mcp_tool_not_found() {
    let mcp_path = match find_smoke_test_mcp() {
        Some(p) => p,
        None => {
            eprintln!("smoke-test-mcp binary not found, skipping integration test");
            return;
        }
    };

    let client = McpClientBuilder::new()
        .stdio(&mcp_path, vec![])
        .build()
        .expect("Failed to create MCP client");

    client.initialize().await.expect("Initialize failed");

    // Try to call non-existent tool
    let result = client
        .call_tool("nonexistent_tool", json!({"arg": "value"}))
        .await;
    assert!(result.is_err(), "Non-existent tool should fail");

    client.shutdown().await.expect("Shutdown failed");
}
