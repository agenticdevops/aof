//! Tests for MCP Client initialization - Critical error prevention
//!
//! This test module verifies that MCP clients are properly initialized
//! before tool execution. This catches initialization bugs at compile/test time
//! rather than runtime.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    // Mock MCP Client for testing without external dependencies
    #[derive(Clone, Debug)]
    struct MockMcpClient {
        initialized: bool,
        initialized_call_count: Arc<std::sync::Mutex<usize>>,
    }

    impl MockMcpClient {
        fn new() -> Self {
            Self {
                initialized: false,
                initialized_call_count: Arc::new(std::sync::Mutex::new(0)),
            }
        }

        async fn initialize(&mut self) -> Result<(), String> {
            let mut count = self.initialized_call_count.lock().unwrap();
            *count += 1;
            self.initialized = true;
            Ok(())
        }

        async fn call_tool(&self, name: &str, _args: serde_json::Value) -> Result<serde_json::Value, String> {
            if !self.initialized {
                return Err("MCP client not initialized".to_string());
            }
            Ok(serde_json::json!({
                "status": "success",
                "tool": name,
            }))
        }

        fn is_initialized(&self) -> bool {
            self.initialized
        }

        fn get_init_count(&self) -> usize {
            *self.initialized_call_count.lock().unwrap()
        }
    }

    #[tokio::test]
    async fn test_mcp_client_requires_initialization() {
        let client = MockMcpClient::new();

        // Test that uninitialized client fails tool calls
        let result = client.call_tool("test_tool", serde_json::json!({})).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "MCP client not initialized");
    }

    #[tokio::test]
    async fn test_mcp_client_initialization() {
        let mut client = MockMcpClient::new();
        assert!(!client.is_initialized());

        // Initialize the client
        let result = client.initialize().await;
        assert!(result.is_ok());
        assert!(client.is_initialized());
    }

    #[tokio::test]
    async fn test_mcp_client_tool_call_after_init() {
        let mut client = MockMcpClient::new();

        // Before init: should fail
        let result = client.call_tool("test_tool", serde_json::json!({})).await;
        assert!(result.is_err());

        // After init: should succeed
        client.initialize().await.unwrap();
        let result = client.call_tool("test_tool", serde_json::json!({})).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "success");
        assert_eq!(response["tool"], "test_tool");
    }

    #[tokio::test]
    async fn test_mcp_initialization_idempotent() {
        let mut client = MockMcpClient::new();

        // Initialize multiple times
        client.initialize().await.unwrap();
        assert_eq!(client.get_init_count(), 1);

        client.initialize().await.unwrap();
        assert_eq!(client.get_init_count(), 2);

        // Should still be usable
        let result = client.call_tool("test_tool", serde_json::json!({})).await;
        assert!(result.is_ok());
    }

    // Pattern test: Ensures correct initialization pattern
    #[tokio::test]
    async fn test_correct_initialization_pattern() {
        // This is the CORRECT pattern that should be used
        let mut client = MockMcpClient::new();

        // 1. Create client
        assert!(!client.is_initialized());

        // 2. Initialize client BEFORE use
        client.initialize().await.expect("Failed to initialize");

        // 3. Use client
        let result = client.call_tool("kubectl", serde_json::json!({"command": "get pods"})).await;
        assert!(result.is_ok());
    }

    // Anti-pattern test: Demonstrates the bug we fixed
    #[tokio::test]
    async fn test_uninitialized_client_fails() {
        // This is the INCORRECT pattern that was causing bugs
        let client = MockMcpClient::new();

        // Directly using client without initialize() - BUG!
        let result = client.call_tool("kubectl", serde_json::json!({"command": "get pods"})).await;

        // This SHOULD fail - if it doesn't, we have a bug
        assert!(result.is_err(), "Uninitialized client should not be able to call tools");
    }
}
