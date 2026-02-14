# Coding Conventions

**Analysis Date:** 2026-02-11

## Naming Patterns

**Files:**
- Snake case: `agent_executor.rs`, `tool_executor.rs`, `fleet.rs`
- Module files: Single word or snake_case (e.g., `mod.rs`, `executor.rs`)
- Test files: Descriptive snake_case (e.g., `executor_tests.rs`, `mcp_initialization.rs`, `command_parsing.rs`)
- Crate names: Kebab case with `aof-` prefix (e.g., `aof-runtime`, `aof-core`, `aof-memory`)

**Functions:**
- Verb-first naming for actions: `execute()`, `initialize()`, `generate()`, `validate_input()`
- Constructor: Always `new()` for standard constructor (e.g., `MockModel::new()`, `Task::new()`)
- Builder pattern: `with_*()` methods (e.g., `with_context()`, `with_max_concurrent()`)
- Getter pattern: No `get_` prefix for simple accessors (e.g., `config()`, `provider()`, `status()`)
- Query pattern: Prefix with `is_`, `has_`, `list_` for boolean/collection returns (e.g., `is_initialized()`, `list_tools()`, `list_tasks()`)
- Helper functions: Lowercase with descriptive names (e.g., `default_timeout()`, `default_temperature()`, `create_test_message()`)

**Variables:**
- Snake case throughout (e.g., `max_concurrent`, `execution_time_ms`, `tool_executor`)
- Boolean prefixes: `is_`, `should_`, `has_` (e.g., `is_initialized`, `should_fail`, `has_context`)
- Collection suffix clarity: Plural for vecs (e.g., `responses`, `tools`, `tool_results`)
- Temporal variables: Suffix with unit (e.g., `timeout_secs`, `execution_time_ms`)

**Types:**
- PascalCase for structs and enums: `AgentExecutor`, `ModelResponse`, `ToolResult`
- Acronyms in PascalCase: `AofError`, `AofResult`, `HttpToolConfig`
- Type aliases: PascalCase (e.g., `AofResult<T>`)
- Enum variants: PascalCase (e.g., `StopReason::EndTurn`, `StopReason::ToolUse`)
- Trait names: PascalCase, often action-based (e.g., `Tool`, `ToolExecutor`, `Model`)

## Code Style

**Formatting:**
- Rust edition: 2021
- Minimum Rust version: 1.75
- Use standard `rustfmt` defaults (4-space indentation)
- Line length: Follow rustfmt defaults
- Module organization: Alphabetical within files

**Linting:**
- Use `cargo clippy` for static analysis
- Lint checks integrated into test suite via `./scripts/test-pre-compile.sh`
- Common patterns checked: MCP initialization, tool executor patterns, configuration consistency

**Async Patterns:**
- Use `tokio` runtime for async tasks
- Mark async functions with `#[tokio::test]` in tests
- Use `async fn` for trait methods with `#[async_trait]` macro
- Use `Pin<Box<dyn futures::Stream<Item = ...> + Send>>` for streaming returns

## Import Organization

**Order:**
1. External crates (e.g., `use async_trait`, `use serde`)
2. Workspace crates (e.g., `use aof_core`, `use aof_memory`)
3. Standard library (e.g., `use std::collections::HashMap`, `use std::sync::Arc`)
4. Internal module imports
5. Conditional imports (e.g., `#[cfg(test)]`)

**Path Aliases:**
- Re-export core types in `lib.rs`: Makes public API clear and imports shorter
- Example from `aof-core/src/lib.rs`: Re-exports `Agent`, `AgentConfig`, `AofError`, etc.
- Crates use full paths in imports: `use aof_core::{ ... }` from workspace dependencies

## Error Handling

**Patterns:**
- Use `AofError` enum for all fallible operations (defined in `aof_core::error`)
- Return `AofResult<T> = Result<T, AofError>` from public APIs
- Use `.into()` for automatic error conversion from compatible types (`serde_json::Error`, `serde_yaml::Error`, `std::io::Error`)
- Create errors with helper methods: `AofError::agent()`, `AofError::tool()`, `AofError::config()`
- Use `serde_path_to_error` for detailed field path errors on YAML/JSON parsing
- Propagate errors with `?` operator in async functions

**Example:**
```rust
// Define error in error.rs
#[derive(Error, Debug)]
pub enum AofError {
    #[error("Tool execution error: {0}")]
    Tool(String),
}

impl AofError {
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }
}

// Use in functions
fn validate_input(&self, _input: &ToolInput) -> AofResult<()> {
    Ok(())
}

// With serde_path_to_error for config
let deserializer = serde_yaml::Deserializer::from_str(&content);
let config: Config = serde_path_to_error::deserialize(deserializer)
    .map_err(|e| anyhow!("Field: {}\nError: {}", e.path(), e.inner()))?;
```

## Logging

**Framework:** `tracing` crate with `tracing-subscriber`

**Patterns:**
- Import: `use tracing::{debug, info, warn, error};`
- Standard levels used: `debug`, `info`, `warn`, `error`
- Log at key lifecycle points: initialization, state transitions, errors
- Include structured data where relevant (e.g., iteration count, tool name, status)

**Example from `agent_executor.rs`:**
```rust
use tracing::{debug, error, info, warn};

debug!("Starting agent execution");
info!("Tool execution completed: {}", tool_name);
warn!("Max iterations reached");
error!("Execution failed: {}", err);
```

## Comments

**When to Comment:**
- Explain complex logic or non-obvious decisions
- Document state machine transitions
- Mark workarounds or temporary solutions with TODO/FIXME
- Explain why, not what (code already shows what)
- Module-level comments: Describe purpose and usage patterns

**JSDoc/Rustdoc:**
- Use `///` for public items
- First line is summary (shown in quick help)
- Blank line before longer descriptions
- Include `#` headings for Examples, Panics, Errors, Safety sections
- Use markdown code blocks with language hints

**Example:**
```rust
/// Tool executor - manages tool execution lifecycle
///
/// This trait defines the interface for executing tools registered with an agent.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    ///
    /// # Arguments
    /// * `name` - Tool identifier
    /// * `input` - Tool arguments
    ///
    /// # Returns
    /// Tool result with execution time and status
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult>;
}
```

## Function Design

**Size:** Keep functions under 200 lines where possible. Larger functions should be broken into helper functions.

**Parameters:**
- Use builder pattern for struct creation instead of many parameters: `Task::new(...).with_priority(10)`
- Accept references for large types: `&AgentConfig` instead of `AgentConfig`
- Use type aliases for common patterns: `AofResult<T>` instead of `Result<T, AofError>`

**Return Values:**
- Return `AofResult<T>` for all fallible operations
- Use tuple returns for multiple related values: `(status, count)`
- Streaming returns use: `Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>`
- Avoid returning raw `Option<T>` from public APIs; prefer `AofResult<T>`

**Example from `tool.rs`:**
```rust
impl ToolInput {
    pub fn new(arguments: serde_json::Value) -> Self {
        Self {
            arguments,
            context: None,
        }
    }

    pub fn with_context(
        arguments: serde_json::Value,
        context: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            arguments,
            context: Some(context),
        }
    }

    pub fn get_arg<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<T> {
        self.arguments
            .get(key)
            .ok_or_else(|| AofError::tool(format!("Missing argument: {}", key)))
            .and_then(|v| serde_json::from_value(v.clone()).map_err(Into::into))
    }
}
```

## Module Design

**Exports:**
- Use `pub use` in `lib.rs` to re-export important types
- Keep internal types private with `pub(crate)`
- Structure: trait definitions, then struct/enum definitions, then impl blocks
- Order: Public types first, then private helper types

**Barrel Files:**
- Use `mod.rs` for re-exporting submodule types
- Example: `crates/aof-core/src/lib.rs` re-exports all public types from submodules

**Workspace Dependencies:**
- Define in `Cargo.toml` workspace section with version and features
- Path resolution: `path = "crates/..."` for local development
- Feature gating: Use `features = ["all"]` for comprehensive capability crates

---

*Convention analysis: 2026-02-11*
