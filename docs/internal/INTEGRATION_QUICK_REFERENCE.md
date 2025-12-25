# AOF Integration Quick Reference Card

**Purpose:** Fast lookup guide for implementing new platform integrations

---

## 📋 Checklist for Adding a New Platform

### Phase 1: Setup
- [ ] Choose feature flag category (`observability`, `siem`, `itsm`, etc.)
- [ ] Add feature to `aof-tools/Cargo.toml` if new category
- [ ] Create module file: `aof-tools/src/tools/{platform}.rs`
- [ ] Declare module in `aof-tools/src/tools/mod.rs`

### Phase 2: Implementation
- [ ] Write module documentation (platform overview, auth, tools list)
- [ ] Create tool collection struct (`{Platform}Tools`)
- [ ] Implement helper functions (`create_{platform}_client`, `handle_{platform}_response`)
- [ ] Implement each tool (struct + `Tool` trait)
- [ ] Export in `aof-tools/src/lib.rs`

### Phase 3: Testing
- [ ] Write unit tests with `mockito` (mocked HTTP)
- [ ] Test error handling (401, 403, 404, 429, 5xx)
- [ ] Add integration tests (optional, with real API)
- [ ] Test with example YAML configuration

### Phase 4: Documentation
- [ ] Create `docs/tools/{platform}.md` (user guide)
- [ ] Create example agent YAML in `examples/`
- [ ] Update `docs/DOCUMENTATION_INDEX.md`
- [ ] Add to `docs/getting-started.md`

---

## 🎯 File Template: Tool Module

**File:** `aof-tools/src/tools/{platform}.rs`

```rust
//! {Platform} Tools
//!
//! ## Available Tools
//! - `{tool_1}` - {description}
//! - `{tool_2}` - {description}
//!
//! ## Prerequisites
//! - Requires `{feature}` feature flag
//! - Valid API credentials
//!
//! ## Authentication
//! - {Auth method}
//! - {Header format}

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

// ============================================================================
// Tool Collection
// ============================================================================

pub struct {Platform}Tools;

impl {Platform}Tools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(Tool1::new()),
            Box::new(Tool2::new()),
        ]
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn create_{platform}_client(credentials) -> AofResult<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", ...);
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Client creation failed: {}", e)))
}

async fn handle_{platform}_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("Parse failed: {}", e)))?;

    match status {
        200 | 201 => Ok(ToolResult::success(body)),
        401 => Ok(ToolResult::error("Authentication failed")),
        403 => Ok(ToolResult::error("Insufficient permissions")),
        404 => Ok(ToolResult::error(format!("Not found: {:?}", body))),
        429 => Ok(ToolResult::error("Rate limited")),
        _ => Ok(ToolResult::error(format!("Status {}: {:?}", status, body))),
    }
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub struct {Tool}Tool {
    config: ToolConfig,
}

impl {Tool}Tool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "param1": {
                    "type": "string",
                    "description": "..."
                },
                // ... more params
            }),
            vec!["param1"], // required fields
        );

        Self {
            config: tool_config_with_timeout(
                "{tool_name}",
                "Tool description",
                parameters,
                60,
            ),
        }
    }
}

impl Default for {Tool}Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for {Tool}Tool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        // 1. Extract arguments
        let param1: String = input.get_arg("param1")?;

        debug!(param1 = %param1, "Executing {tool_name}");

        // 2. Create client
        let client = create_{platform}_client(...).await?;

        // 3. Build request
        let url = format!("{}/api/endpoint", endpoint);

        // 4. Execute request
        let response = client.get(&url).send().await
            .map_err(|e| aof_core::AofError::tool(format!("Request failed: {}", e)))?;

        // 5. Handle response
        handle_{platform}_response(response, "{tool_name}").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_{tool}_success() {
        let server = mockito::Server::new();
        let mock = server.mock("GET", "/api/endpoint")
            .with_status(200)
            .with_body(r#"{"result": "success"}"#)
            .create();

        let tool = {Tool}Tool::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": server.url(),
            "param1": "test"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        mock.assert();
    }
}
```

---

## 🔑 Common Patterns

### JSON Schema Parameters

```rust
let parameters = create_schema(
    serde_json::json!({
        "string_param": {
            "type": "string",
            "description": "A string parameter",
            "example": "example value"
        },
        "integer_param": {
            "type": "integer",
            "description": "An integer parameter",
            "default": 100
        },
        "enum_param": {
            "type": "string",
            "description": "Choice parameter",
            "enum": ["option1", "option2", "option3"],
            "default": "option1"
        },
        "array_param": {
            "type": "array",
            "description": "Array of strings",
            "items": {
                "type": "string"
            },
            "example": ["item1", "item2"]
        },
        "object_param": {
            "type": "object",
            "description": "Nested object",
            "properties": {
                "key1": {"type": "string"},
                "key2": {"type": "integer"}
            }
        }
    }),
    vec!["string_param"], // required fields only
);
```

### HTTP Client with Auth Headers

**API Key:**
```rust
let mut headers = reqwest::header::HeaderMap::new();
headers.insert("X-Api-Key", HeaderValue::from_str(api_key)?);
```

**Bearer Token:**
```rust
headers.insert(
    reqwest::header::AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {}", token))?
);
```

**Basic Auth:**
```rust
let auth = base64::encode(format!("{}:{}", username, password));
headers.insert(
    reqwest::header::AUTHORIZATION,
    HeaderValue::from_str(&format!("Basic {}", auth))?
);
```

### Extracting Arguments

```rust
// Required argument (returns error if missing)
let required: String = input.get_arg("param")?;

// Optional argument with default
let optional: String = input.get_arg("param").unwrap_or_else(|_| "default".to_string());

// Optional argument (None if missing)
let maybe: Option<String> = input.get_arg("param").ok();

// Integer argument
let count: i32 = input.get_arg("count")?;

// Boolean argument
let enabled: bool = input.get_arg("enabled")?;

// Array argument
let items: Vec<String> = input.get_arg("items")?;
```

### Error Handling

```rust
match response.status().as_u16() {
    200 | 201 => {
        let body: serde_json::Value = response.json().await?;
        Ok(ToolResult::success(body))
    }
    400 => {
        let error_msg = extract_error_message(&response).await;
        Ok(ToolResult::error(format!("Invalid request: {}", error_msg)))
    }
    401 => Ok(ToolResult::error("Authentication failed. Check credentials.")),
    403 => Ok(ToolResult::error("Insufficient permissions for this operation.")),
    404 => Ok(ToolResult::error("Resource not found")),
    429 => Ok(ToolResult::error("Rate limited. Retry later.")),
    500..=599 => Ok(ToolResult::error(format!("Server error ({})", status))),
    _ => Ok(ToolResult::error(format!("Unexpected status: {}", status))),
}
```

---

## 📦 Cargo.toml Changes

### Add Feature Flag

```toml
[features]
default = ["file", "shell", "git"]
observability = ["reqwest", "chrono"]
siem = ["reqwest", "chrono"]  # NEW
itsm = ["reqwest", "chrono", "base64"]  # NEW
all = ["file", "shell", "...", "observability", "siem", "itsm"]

[dependencies]
base64 = { version = "0.22", optional = true }  # If needed
urlencoding = { version = "2.1", optional = true }  # If needed
```

---

## 📝 Module Declaration

### In `aof-tools/src/tools/mod.rs`

```rust
#[cfg(feature = "observability")]
pub mod newrelic;

#[cfg(feature = "siem")]
pub mod splunk;

#[cfg(feature = "itsm")]
pub mod servicenow;
```

### In `aof-tools/src/lib.rs`

```rust
#[cfg(feature = "observability")]
pub use tools::newrelic::{NewRelicTools, NewRelicQueryTool, NewRelicAlertTool};

#[cfg(feature = "siem")]
pub use tools::splunk::{SplunkTools, SplunkSearchTool, SplunkAlertTool};

#[cfg(feature = "itsm")]
pub use tools::servicenow::{ServiceNowTools, ServiceNowIncidentTool};
```

---

## 🧪 Testing Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_success() {
        let server = mockito::Server::new();
        let mock = server.mock("POST", "/api/endpoint")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "success", "data": []}"#)
            .create();

        let tool = ToolName::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": server.url(),
            "api_key": "test-key",
            "param": "value"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert!(result.error.is_none());
        mock.assert();
    }

    #[tokio::test]
    async fn test_tool_auth_error() {
        let server = mockito::Server::new();
        let mock = server.mock("POST", "/api/endpoint")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        let tool = ToolName::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": server.url(),
            "api_key": "invalid-key"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        mock.assert();
    }

    #[tokio::test]
    async fn test_tool_missing_param() {
        let tool = ToolName::new();
        let input = ToolInput::new(serde_json::json!({}));

        let result = tool.execute(input).await;
        assert!(result.is_err()); // Missing required parameter
    }
}
```

---

## 📄 Example YAML Configuration

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: {platform}-agent
  labels:
    platform: {platform}
    environment: production
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a {purpose} agent. Use {platform} tools to {what}.

  tools:
    - name: {tool_name}
      source: builtin
      config:
        # Platform-specific config
        api_key: "{{ secrets.{platform}_api_key }}"
        endpoint: "https://api.{platform}.com"

  max_iterations: 10
  temperature: 0.7
```

---

## 🔍 Debugging Tips

### Enable Tracing
```bash
RUST_LOG=aof_tools=debug aofctl run agent path/to/agent.yaml
```

### Test Tool Individually
```rust
#[tokio::main]
async fn main() {
    let tool = ToolName::new();
    let input = ToolInput::new(serde_json::json!({
        "param": "value"
    }));

    match tool.execute(input).await {
        Ok(result) => println!("Success: {:?}", result.data),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Check JSON Schema
```rust
let tool = ToolName::new();
let definition = tool.definition();
println!("Schema: {}", serde_json::to_string_pretty(&definition.parameters).unwrap());
```

---

## 🚀 Build & Test Commands

```bash
# Check syntax
cargo check --features {feature}

# Run unit tests for module
cargo test --lib --features {feature} {module_name}

# Run all tests
cargo test --features all

# Build with feature
cargo build --release --features {feature}

# Lint
cargo clippy --features {feature}

# Format
cargo fmt

# Test with example
aofctl run agent examples/{platform}-agent.yaml
```

---

## 📚 Common Tool Types

### Query/Search Tool
- NRQL, SPL, SQL queries
- Parameters: `query`, `start_time`, `end_time`, `limit`
- Returns: Array of results

### Alert/Monitor Tool
- List alerts, violations, monitors
- Parameters: `state` (open/closed), `priority`, `time_range`
- Returns: Array of alerts with metadata

### Create/Update Tool
- Create incidents, tickets, events
- Parameters: `title`, `description`, `priority`, `assignment`
- Returns: Created resource with ID

### Metric Tool
- Get timeseries metrics
- Parameters: `metric_name`, `from`, `to`, `aggregation`
- Returns: Timeseries data points

---

## 🎯 Reference Implementations

- **Simple HTTP API:** `datadog.rs` (metric query, log query, monitors)
- **Async Polling:** `splunk.rs` (search job creation → poll → results)
- **GraphQL API:** `newrelic.rs` (GraphQL queries for NRQL)
- **Complex Auth:** `servicenow.rs` (Basic auth, encoded queries)

---

## 📞 Need Help?

- **Architecture docs:** `docs/internal/INTEGRATION_ARCHITECTURE.md`
- **API specs:** `docs/internal/INTEGRATION_API_SPEC.md`
- **Visual diagrams:** `docs/internal/INTEGRATION_DIAGRAMS.md`
- **Reference code:** `aof-tools/src/tools/datadog.rs`

---

**Quick Reference Version:** 1.0
**Last Updated:** 2025-12-25
**Maintained by:** AOF Architecture Team
