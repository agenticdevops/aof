# Testing Patterns

**Analysis Date:** 2026-02-11

## Test Framework

**Runner:**
- `tokio` test harness with `#[tokio::test]` macro
- Version: 1.35+ (from workspace Cargo.toml)
- Features: `["full"]` for comprehensive async/blocking support
- Test utilities: `test-util` feature enabled in dev-dependencies

**Assertion Library:**
- Rust's standard `assert!`, `assert_eq!`, `assert_ne!`
- Pattern matching with `assert!(matches!(value, pattern))`
- No external assertion library; keep tests idiomatic Rust

**Run Commands:**
```bash
cargo test --lib                    # Run all unit tests
cargo test --lib --all-features     # With all feature flags
cargo test --test '*'               # Run all integration tests
cargo test test_executor            # Single test file
cargo test -- --test-threads=1      # Serial execution
./scripts/test-pre-compile.sh        # Quick validation (5 seconds)
```

## Test File Organization

**Location:**
- Integration tests: `crates/{crate-name}/tests/*.rs` - separate from source
- Examples: Reference tests co-located with code in modules (internal `mod tests { }`)
- Patterns: Tests verify behavior without requiring external systems

**Naming:**
- Test files: Descriptive snake_case: `executor_tests.rs`, `mcp_initialization.rs`, `tool_executor.rs`, `command_parsing.rs`
- Test functions: Start with `test_`, describe what is being tested: `test_executor_simple_execution()`, `test_mcp_client_requires_initialization()`
- Helper functions: Action-based: `create_test_message()`, `create_test_task()`, `create_test_model()`

**Structure:**
```
crates/aof-runtime/
├── src/
│   ├── executor/
│   │   └── agent_executor.rs
│   └── lib.rs
└── tests/
    ├── executor_tests.rs       # Integration tests for AgentExecutor
    ├── mcp_initialization.rs    # MCP initialization tests
    ├── tool_executor.rs         # Tool executor flow tests
    └── orchestrator_tests.rs    # RuntimeOrchestrator tests
```

## Test Structure

**Suite Organization:**
```rust
#[tokio::test]
async fn test_name() {
    // Setup
    let executor = AgentExecutor::new(config, model, None, None);
    let mut context = AgentContext::new("Hello");

    // Act
    let result = executor.execute(&mut context).await.unwrap();

    // Assert
    assert_eq!(result, "Expected response");
}
```

**Patterns:**

1. **Setup-Act-Assert (AAA):**
   - Setup: Create mocks, fixtures, configuration
   - Act: Call the function being tested
   - Assert: Verify expected outcomes

2. **Async Testing with Tokio:**
```rust
#[tokio::test]
async fn test_executor_tool_calls() {
    let model = Box::new(MockModel::new(responses));
    let executor = AgentExecutor::new(config, model, tool_executor, None);
    let mut context = AgentContext::new("Do something");

    let result = executor.execute(&mut context).await.unwrap();
    assert_eq!(result, "Tool failed, but I'll continue");
}
```

3. **Error Handling Tests:**
```rust
#[tokio::test]
async fn test_executor_max_iterations() {
    let result = executor.execute(&mut context).await;
    assert!(result.is_err());  // Verify error occurred
}
```

4. **State Verification:**
```rust
#[tokio::test]
async fn test_executor_with_tool_calls() {
    let result = executor.execute(&mut context).await.unwrap();

    // Verify state changed
    assert_eq!(context.metadata.tool_calls, 1);
    assert_eq!(context.tool_results.len(), 1);
    assert!(!context.tool_results[0].success);
}
```

## Mocking

**Framework:** Custom mock implementations using `#[derive(Clone, Debug)]` structs

**Patterns:**

1. **Mock Model Implementation:**
```rust
struct MockModel {
    responses: Vec<ModelResponse>,
    current: Mutex<usize>,
    config: ModelConfig,
}

#[async_trait]
impl Model for MockModel {
    async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
        let mut current = self.current.lock().unwrap();
        let idx = *current;
        *current += 1;

        if idx < self.responses.len() {
            Ok(self.responses[idx].clone())
        } else {
            Ok(ModelResponse { /* default */ })
        }
    }
}
```

2. **Mock Tool Executor:**
```rust
struct MockToolExecutor {
    should_fail: bool,
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, name: &str, _input: ToolInput) -> AofResult<ToolResult> {
        if self.should_fail {
            return Ok(ToolResult::error(format!("Tool {} failed", name)));
        }
        Ok(ToolResult::success(serde_json::json!({
            "tool": name,
            "result": "success"
        })).with_execution_time(50))
    }
}
```

3. **Mock MCP Client:**
```rust
#[derive(Clone, Debug)]
struct MockMcpClient {
    initialized: bool,
    initialized_call_count: Arc<std::sync::Mutex<usize>>,
}

impl MockMcpClient {
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
        Ok(serde_json::json!({"status": "success", "tool": name}))
    }
}
```

**What to Mock:**
- External LLM models (OpenAI, Anthropic APIs)
- Tool executors and MCP clients
- Async operations that would cause test slowdown
- File system operations
- Network calls

**What NOT to Mock:**
- Core domain logic (AgentConfig, AgentContext)
- Error types and result handling
- Serialization/deserialization
- Simple struct constructors

## Fixtures and Factories

**Test Data:**
```rust
fn create_test_message(text: &str) -> TriggerMessage {
    let user = TriggerUser {
        id: "user123".to_string(),
        username: Some("testuser".to_string()),
        display_name: Some("Test User".to_string()),
        is_bot: false,
    };

    TriggerMessage::new(
        "msg123".to_string(),
        "telegram".to_string(),
        "chat456".to_string(),
        user,
        text.to_string(),
    )
}

fn create_test_task(id: &str, name: &str) -> Task {
    Task::new(
        id.to_string(),
        name.to_string(),
        "test-agent".to_string(),
        "Test input".to_string(),
    )
}
```

**Location:**
- Keep fixtures in test file at top level or in helper functions
- Define before test functions
- Name with `create_*` prefix for clarity

## Coverage

**Requirements:** Not enforced via CI, but high coverage expected

**View Coverage:**
```bash
# Generate coverage report (requires tarpaulin)
cargo tarpaulin --out Html

# Or with llvm-cov
cargo llvm-cov --html
```

## Test Types

**Unit Tests:**
- Scope: Single function or small module behavior
- Location: Usually within `tests/*.rs` files with `#[tokio::test]`
- Pattern: Quick, deterministic, no external dependencies
- Example: `test_parse_run_agent_command()` - tests command parsing logic
- Example: `test_executor_simple_execution()` - tests basic agent execution

**Integration Tests:**
- Scope: Multiple components working together
- Location: `tests/*.rs` files with full setup
- Pattern: Mock external systems, test integration points
- Example: `test_executor_with_tool_calls()` - tests executor + tool executor interaction
- Example: `test_orchestrator_submission()` - tests task submission through orchestrator

**E2E Tests:**
- Status: Not used - focus on unit + integration tests
- External systems: Mocked to avoid external dependencies

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

// With multiple async operations
#[tokio::test]
async fn test_multiple_async_calls() {
    let mut client = MockMcpClient::new();
    client.initialize().await.unwrap();

    let result = client.call_tool("test_tool", serde_json::json!({})).await;
    assert!(result.is_ok());
}
```

**Error Testing:**
```rust
#[tokio::test]
async fn test_error_cases() {
    // Test 1: Invalid state
    let client = MockMcpClient::new();
    let result = client.call_tool("test_tool", serde_json::json!({})).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "MCP client not initialized");

    // Test 2: Missing parameters
    let mut executor = ToolExecutorTest::new();
    executor.register_tool("kubectl", "Kubernetes commands", serde_json::json!({}));

    let result = executor.execute_tool("kubectl", serde_json::json!({})).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Missing 'command' argument for kubectl");
}
```

**Parameterized Testing:**
```rust
#[tokio::test]
async fn test_executor_stop_reasons() {
    let test_cases = vec![
        (StopReason::EndTurn, "Normal completion"),
        (StopReason::MaxTokens, "Max tokens reached"),
        (StopReason::StopSequence, "Stop sequence hit"),
    ];

    for (stop_reason, expected_content) in test_cases {
        let responses = vec![ModelResponse {
            content: expected_content.to_string(),
            tool_calls: vec![],
            stop_reason,
            usage: Usage::default(),
            metadata: HashMap::new(),
        }];

        let model = Box::new(MockModel::new(responses));
        let executor = AgentExecutor::new(config, model, None, None);
        let result = executor.execute(&mut context).await.unwrap();
        assert_eq!(result, expected_content);
    }
}
```

**Behavior-Driven Tests (Anti-Pattern Detection):**
```rust
// Pattern test: Ensures CORRECT initialization pattern
#[tokio::test]
async fn test_correct_initialization_pattern() {
    let mut client = MockMcpClient::new();

    // 1. Create client
    assert!(!client.is_initialized());

    // 2. Initialize BEFORE use
    client.initialize().await.expect("Failed to initialize");

    // 3. Use client
    let result = client.call_tool("kubectl", serde_json::json!({"command": "get pods"})).await;
    assert!(result.is_ok());
}

// Anti-pattern test: Shows bug we fixed
#[tokio::test]
async fn test_uninitialized_client_fails() {
    let client = MockMcpClient::new();

    // Bug: Using uninitialized client
    let result = client.call_tool("kubectl", serde_json::json!({"command": "get pods"})).await;

    // This SHOULD fail
    assert!(result.is_err(), "Uninitialized client should not be able to call tools");
}
```

---

*Testing analysis: 2026-02-11*
