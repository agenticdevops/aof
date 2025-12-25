# AOF Integration Architecture Analysis
## Adding New Relic, Splunk, and ServiceNow Integrations

**Date:** 2025-12-25
**Objective:** Understand AOF system architecture for adding platform integrations
**Platforms:** New Relic (observability), Splunk (logs/SIEM), ServiceNow (ITSM)

---

## 1. Crate Dependency Graph

```
aof (workspace root)
├── aof-core           # Core traits & types (FOUNDATION)
│   ├── Tool trait
│   ├── Agent trait
│   ├── AgentConfig
│   ├── AgentFleet
│   └── Error types
│
├── aof-tools          # Tool implementations (ADD INTEGRATIONS HERE)
│   ├── Features: observability, cicd, security, cloud
│   ├── tools/
│   │   ├── observability.rs (Prometheus, Loki, Elasticsearch, VictoriaMetrics)
│   │   ├── grafana.rs
│   │   ├── datadog.rs        <- REFERENCE IMPLEMENTATION
│   │   ├── [newrelic.rs]     <- NEW
│   │   ├── [splunk.rs]       <- NEW
│   │   └── [servicenow.rs]   <- NEW
│   └── registry.rs (Tool registration)
│
├── aof-llm            # LLM provider abstraction
├── aof-mcp            # MCP client
├── aof-memory         # State management
├── aof-runtime        # Agent execution engine
│   ├── executor/
│   │   └── agent_executor.rs (Uses ToolExecutor)
│   └── fleet/ (Multi-agent orchestration)
│
└── aofctl             # CLI binary
    └── Uses aof-runtime + aof-tools
```

**Key Dependencies:**
- **aof-core** → No dependencies (pure traits/types)
- **aof-tools** → Depends on aof-core only
- **aof-runtime** → Depends on aof-core, aof-tools, aof-llm, aof-mcp, aof-memory
- **aofctl** → Depends on everything

---

## 2. Core Trait Definitions to Implement

### 2.1 Tool Trait (aof-core/src/tool.rs)

**Primary trait for all integrations:**

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with input arguments
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;

    /// Tool configuration (name, description, parameters)
    fn config(&self) -> &ToolConfig;

    /// Validate tool input schema (optional override)
    fn validate_input(&self, input: &ToolInput) -> AofResult<()> {
        Ok(())
    }

    /// Tool definition for LLM (auto-generated from config)
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.config().name.clone(),
            description: self.config().description.clone(),
            parameters: self.config().parameters.clone(),
        }
    }
}
```

**Key Types:**

```rust
pub struct ToolConfig {
    pub name: String,              // e.g., "newrelic_query"
    pub description: String,       // Human-readable description
    pub parameters: serde_json::Value, // JSON Schema for LLM
    pub tool_type: ToolType,       // Mcp, Shell, Http, Custom
    pub timeout_secs: u64,         // Execution timeout
    pub extra: HashMap<String, serde_json::Value>, // Extension point
}

pub struct ToolInput {
    pub arguments: serde_json::Value,  // Tool arguments from LLM
    pub context: Option<HashMap<String, serde_json::Value>>, // Agent context
}

pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,  // Result payload
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
```

### 2.2 ToolExecutor Trait (aof-core/src/tool.rs)

**Runtime execution interface (already implemented in aof-tools/registry.rs):**

```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult>;

    /// Get all available tools
    fn list_tools(&self) -> Vec<ToolDefinition>;

    /// Get specific tool
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
}
```

**Implementation:** `BuiltinToolExecutor` in `aof-tools/src/registry.rs` handles registration.

---

## 3. Reference Implementation Pattern (Datadog)

**File:** `aof-tools/src/tools/datadog.rs`

### 3.1 Structure

```rust
// 1. Module documentation with prerequisites
//! Datadog Tools
//!
//! ## Available Tools
//! - `datadog_metric_query` - Query metrics
//! - `datadog_log_query` - Search logs
//! ...
//!
//! ## Authentication
//! - Requires DD-API-KEY and DD-APPLICATION-KEY headers
//! - Supports environment variables

// 2. Tool collection struct (for bulk registration)
pub struct DatadogTools;

impl DatadogTools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(DatadogMetricQueryTool::new()),
            Box::new(DatadogLogQueryTool::new()),
            Box::new(DatadogMonitorListTool::new()),
            // ... more tools
        ]
    }
}

// 3. Helper functions (authentication, error handling)
async fn create_datadog_client(api_key: &str, app_key: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("DD-API-KEY", ...);
    headers.insert("DD-APPLICATION-KEY", ...);
    Client::builder().default_headers(headers).build()
}

async fn handle_datadog_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    // Unified error handling
}

// 4. Individual tool implementations
pub struct DatadogMetricQueryTool {
    config: ToolConfig,
}

impl DatadogMetricQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": { "type": "string", ... },
                "api_key": { "type": "string", ... },
                "query": { "type": "string", ... },
                // ... more params
            }),
            vec!["api_key", "query"], // required fields
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_metric_query",
                "Query Datadog metrics using Datadog query language",
                parameters,
                60, // timeout in seconds
            ),
        }
    }
}

#[async_trait]
impl Tool for DatadogMetricQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        // 1. Extract arguments
        let endpoint: String = input.get_arg("endpoint")?;
        let api_key: String = input.get_arg("api_key")?;
        let query: String = input.get_arg("query")?;

        // 2. Create authenticated client
        let client = create_datadog_client(&api_key, &app_key).await?;

        // 3. Build API request
        let url = format!("{}/api/v1/query", endpoint.trim_end_matches('/'));
        let params = [("query", query.as_str())];

        // 4. Execute request
        let response = client.get(&url).query(&params).send().await
            .map_err(|e| /* error handling */)?;

        // 5. Handle response
        handle_datadog_response(response, "Datadog metric query").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 3.2 Key Patterns

1. **Authentication Headers:** Use `reqwest::header::HeaderMap` for custom auth
2. **Error Handling:** Centralized `handle_*_response()` functions
3. **JSON Schema:** Use `create_schema()` helper from `tools/common.rs`
4. **Timeout:** Set explicit timeouts (default 60s for API calls)
5. **Environment Variables:** Document in module comments, let users pass as args
6. **Time Parsing:** Helper functions for relative time (`-1h`, `now`, ISO 8601)

---

## 4. Module Structure for New Integrations

### 4.1 Feature Flags (Cargo.toml)

**Add to `aof-tools/Cargo.toml`:**

```toml
[features]
default = ["file", "shell", "git"]
observability = ["reqwest", "chrono"]
itsm = ["reqwest", "chrono", "base64"]  # NEW: For ServiceNow
siem = ["reqwest", "chrono"]            # NEW: For Splunk
all = ["file", "shell", "...", "observability", "itsm", "siem"]

[dependencies]
# Existing...
base64 = { version = "0.22", optional = true }
urlencoding = { version = "2.1", optional = true }
```

### 4.2 File Organization

```
aof-tools/src/tools/
├── mod.rs              # Module declarations
├── common.rs           # Shared helpers
├── observability.rs    # Prometheus, Loki, etc.
├── datadog.rs          # Reference implementation
├── newrelic.rs         # NEW: New Relic integration
├── splunk.rs           # NEW: Splunk integration
└── servicenow.rs       # NEW: ServiceNow integration
```

**Update `mod.rs`:**

```rust
#[cfg(feature = "observability")]
pub mod datadog;

#[cfg(feature = "observability")]
pub mod newrelic;

#[cfg(feature = "siem")]
pub mod splunk;

#[cfg(feature = "itsm")]
pub mod servicenow;
```

### 4.3 Tool Registration

**Update `aof-tools/src/lib.rs`:**

```rust
#[cfg(feature = "observability")]
pub use tools::newrelic::{NewRelicTools, NewRelicQueryTool, NewRelicAlertTool};

#[cfg(feature = "siem")]
pub use tools::splunk::{SplunkTools, SplunkSearchTool, SplunkAlertTool};

#[cfg(feature = "itsm")]
pub use tools::servicenow::{ServiceNowTools, ServiceNowIncidentTool, ServiceNowCMDBTool};
```

---

## 5. Integration Specifications

### 5.1 New Relic Integration

**Module:** `aof-tools/src/tools/newrelic.rs`
**Feature:** `observability`
**Authentication:** API Key (X-Api-Key header) or Personal Access Token
**Endpoints:** Regional (US, EU)

**Tools to Implement:**

1. **newrelic_nrql_query**
   - Query NRQL (New Relic Query Language)
   - Parameters: `account_id`, `query`, `api_key`, `region`
   - API: `POST /v1/accounts/{accountId}/query`

2. **newrelic_alert_list**
   - List alert violations
   - Parameters: `api_key`, `only_open`, `start_time`
   - API: `GET /v2/alerts_violations.json`

3. **newrelic_entity_search**
   - Search APM entities (apps, services)
   - Parameters: `query`, `entity_type`
   - API: GraphQL endpoint

4. **newrelic_metric_data**
   - Get metric timeseries data
   - Parameters: `names`, `values`, `from`, `to`
   - API: `GET /v2/applications/{id}/metrics/data.json`

5. **newrelic_incident_create**
   - Create incident (for automation)
   - Parameters: `title`, `priority`, `description`
   - API: `POST /v2/alerts_incidents.json`

**Authentication Pattern:**

```rust
async fn create_newrelic_client(api_key: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Api-Key", HeaderValue::from_str(api_key)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    Client::builder().default_headers(headers).build()
}
```

### 5.2 Splunk Integration

**Module:** `aof-tools/src/tools/splunk.rs`
**Feature:** `siem`
**Authentication:** Bearer token or basic auth
**Endpoints:** Self-hosted or Splunk Cloud

**Tools to Implement:**

1. **splunk_search**
   - Run SPL (Splunk Processing Language) search
   - Parameters: `search`, `earliest_time`, `latest_time`, `auth_token`
   - API: `POST /services/search/jobs` + `GET /services/search/jobs/{sid}/results`
   - Note: Async pattern - create job, poll for results

2. **splunk_saved_search_run**
   - Run saved search
   - Parameters: `search_name`, `args`
   - API: `POST /services/saved/searches/{name}/dispatch`

3. **splunk_alert_list**
   - List triggered alerts
   - Parameters: `earliest_time`, `count`
   - API: `GET /services/alerts/fired_alerts`

4. **splunk_event_submit**
   - Submit event to HEC (HTTP Event Collector)
   - Parameters: `event`, `source`, `sourcetype`, `index`, `hec_token`
   - API: `POST /services/collector/event`

5. **splunk_index_list**
   - List available indexes
   - API: `GET /services/data/indexes`

**Authentication Pattern:**

```rust
async fn create_splunk_client(endpoint: &str, token: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    // Note: Splunk self-signed certs require .danger_accept_invalid_certs(true)
    Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()
}
```

**Special Considerations:**
- Splunk search is **asynchronous** - create job, poll status, get results
- Need helper for polling: `poll_search_job(client, sid, max_attempts, interval)`
- Support both Splunk Enterprise (self-hosted) and Splunk Cloud

### 5.3 ServiceNow Integration

**Module:** `aof-tools/src/tools/servicenow.rs`
**Feature:** `itsm`
**Authentication:** Basic auth (username:password) or OAuth
**Endpoints:** `{instance}.service-now.com`

**Tools to Implement:**

1. **servicenow_incident_create**
   - Create incident ticket
   - Parameters: `short_description`, `description`, `urgency`, `impact`, `caller_id`
   - API: `POST /api/now/table/incident`

2. **servicenow_incident_update**
   - Update incident (e.g., add work notes, close)
   - Parameters: `sys_id`, `state`, `work_notes`, `close_notes`
   - API: `PATCH /api/now/table/incident/{sys_id}`

3. **servicenow_incident_query**
   - Query incidents with filters
   - Parameters: `query`, `sysparm_limit`, `sysparm_fields`
   - API: `GET /api/now/table/incident?sysparm_query=...`

4. **servicenow_cmdb_query**
   - Query Configuration Items (CMDB)
   - Parameters: `ci_class`, `query`
   - API: `GET /api/now/table/{ci_class}`

5. **servicenow_change_create**
   - Create change request
   - Parameters: `short_description`, `type`, `risk`, `start_date`
   - API: `POST /api/now/table/change_request`

6. **servicenow_problem_create**
   - Create problem ticket
   - Parameters: `short_description`, `impact`, `urgency`
   - API: `POST /api/now/table/problem`

**Authentication Pattern:**

```rust
async fn create_servicenow_client(
    instance: &str,
    username: &str,
    password: &str,
) -> AofResult<Client> {
    let auth = base64::encode(format!("{}:{}", username, password));
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", auth))?,
    );
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    Client::builder().default_headers(headers).build()
}
```

**Special Considerations:**
- ServiceNow uses **encoded queries** for filtering: `sysparm_query=active=true^priority=1`
- Support both **Table API** (`/api/now/table/*`) and **Aggregate API** for analytics
- Return `sys_id` in results for follow-up operations

---

## 6. Configuration Schema Patterns

### 6.1 YAML Agent Configuration

**Users configure tools in agent YAML:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: sre-incident-responder
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are an SRE incident responder. Use New Relic and Splunk to investigate
    issues, then create ServiceNow incidents for tracking.

  tools:
    # New Relic tools
    - name: newrelic_nrql_query
      source: builtin
      config:
        account_id: "123456"
        region: "us"  # or "eu"

    # Splunk tools
    - name: splunk_search
      source: builtin
      config:
        endpoint: "https://splunk.example.com:8089"

    # ServiceNow tools
    - name: servicenow_incident_create
      source: builtin
      config:
        instance: "dev12345"

    - name: servicenow_incident_query
      source: builtin
```

### 6.2 Fleet Configuration (Multi-Agent)

**RCA example with tiered execution:**

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: rca-investigation
spec:
  agents:
    # Tier 1: Data collectors (cheap models)
    - name: newrelic-collector
      tier: 1
      spec:
        model: google:gemini-2.0-flash
        tools:
          - newrelic_nrql_query
          - newrelic_metric_data

    - name: splunk-collector
      tier: 1
      spec:
        model: google:gemini-2.0-flash
        tools:
          - splunk_search
          - splunk_saved_search_run

    # Tier 2: Reasoners (powerful models)
    - name: rca-analyzer
      tier: 2
      weight: 2.0
      spec:
        model: anthropic:claude-sonnet-4
        instructions: "Analyze observability data to find root cause"

    # Tier 3: Ticket creator
    - name: incident-coordinator
      tier: 3
      role: manager
      spec:
        model: google:gemini-2.5-flash
        tools:
          - servicenow_incident_create
          - servicenow_incident_update

  coordination:
    mode: tiered
    tiered:
      pass_all_results: true
      final_aggregation: manager_synthesis
```

---

## 7. Implementation Roadmap

### Phase 1: New Relic (Observability)
1. Create `aof-tools/src/tools/newrelic.rs`
2. Implement 5 core tools (NRQL query, alerts, entities, metrics, incidents)
3. Add feature flag `observability` (already exists, extend it)
4. Update `mod.rs` and `lib.rs` exports
5. Write unit tests with mocked HTTP responses
6. Create example agent YAML in `examples/observability/newrelic-agent.yaml`
7. Document in `docs/tools/newrelic.md`

### Phase 2: Splunk (SIEM/Logs)
1. Create `aof-tools/src/tools/splunk.rs`
2. Implement async search pattern (create job → poll → get results)
3. Add feature flag `siem` to Cargo.toml
4. Implement 5 core tools (search, saved search, alerts, event submit, indexes)
5. Handle self-signed certificate option
6. Write unit tests
7. Create example fleet YAML for log investigation
8. Document in `docs/tools/splunk.md`

### Phase 3: ServiceNow (ITSM)
1. Create `aof-tools/src/tools/servicenow.rs`
2. Implement 6 core tools (incident CRUD, CMDB, change, problem)
3. Add feature flag `itsm` to Cargo.toml
4. Implement encoded query builder helper
5. Write unit tests
6. Create example incident automation workflow
7. Document in `docs/tools/servicenow.md`

### Phase 4: Integration Testing
1. Create end-to-end fleet example: **Incident Response Pipeline**
   - New Relic detects anomaly
   - Splunk searches correlated logs
   - ServiceNow creates incident with RCA summary
2. Performance benchmarks
3. Documentation updates (architecture docs, getting-started guide)

---

## 8. Architectural Decisions

### ADR-001: Built-in Tools vs MCP Servers

**Decision:** Implement as built-in tools in `aof-tools` crate, not MCP servers.

**Rationale:**
1. **Performance:** Direct HTTP calls faster than MCP protocol overhead
2. **Simplicity:** No external process management
3. **Consistency:** Matches existing observability tools (Datadog, Grafana)
4. **Packaging:** Ships with `aofctl` binary, no installation required
5. **Type Safety:** Rust's type system validates at compile time

**Drawbacks:**
- Less flexibility than MCP (can't update without recompiling)
- Can't use external MCP servers for these platforms

**Mitigation:** Users can still use MCP servers if they prefer. Built-in tools are for convenience.

### ADR-002: Feature Flag Organization

**Decision:** Use feature flags `observability`, `siem`, `itsm` instead of per-platform flags.

**Rationale:**
1. **Grouping:** Logical categorization by use case
2. **Dependencies:** Shared dependencies (reqwest, chrono) within category
3. **Binary Size:** Users enable categories they need (e.g., `--features observability`)
4. **Consistency:** Matches existing pattern (observability already exists)

**Implementation:**
- New Relic → `observability` (alongside Datadog, Grafana)
- Splunk → `siem` (new category for log/security platforms)
- ServiceNow → `itsm` (new category for IT service management)

### ADR-003: Authentication Pattern

**Decision:** Pass credentials as tool arguments, not environment variables.

**Rationale:**
1. **Flexibility:** Agent config can specify different credentials per tool
2. **Multi-tenancy:** Fleet can use different accounts per agent
3. **Explicit:** Clear in YAML what credentials are needed
4. **Security:** Credentials come from Context (can integrate with Vault, AWS Secrets Manager)

**Implementation:**
```yaml
tools:
  - name: newrelic_nrql_query
    config:
      api_key: "{{ secrets.newrelic_api_key }}"  # From context
      account_id: "123456"
```

### ADR-004: Async Search Pattern (Splunk)

**Decision:** Implement polling helper for async searches.

**Rationale:**
1. **Splunk API:** Search is inherently async (create job → poll → results)
2. **User Experience:** Hide complexity from LLM/user
3. **Timeout Handling:** Built-in retry and timeout logic
4. **Reusability:** Helper function used by multiple tools

**Implementation:**
```rust
async fn poll_search_job(
    client: &Client,
    endpoint: &str,
    sid: &str,
    max_attempts: u32,
    interval_ms: u64,
) -> AofResult<serde_json::Value> {
    // Poll /services/search/jobs/{sid} until isDone=true
}
```

---

## 9. Testing Strategy

### Unit Tests (Per Tool)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_newrelic_query_success() {
        // Use mockito to mock HTTP responses
        let mock_server = mockito::Server::new();
        let mock = mock_server.mock("POST", "/v1/accounts/123/query")
            .with_status(200)
            .with_body(r#"{"results": [...]}"#)
            .create();

        let tool = NewRelicNRQLQueryTool::new();
        let input = ToolInput::new(serde_json::json!({
            "endpoint": mock_server.url(),
            "account_id": "123",
            "query": "SELECT * FROM Transaction",
            "api_key": "test-key"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        mock.assert();
    }
}
```

### Integration Tests (Fleet)
```yaml
# tests/fixtures/incident-response.yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: test-incident-response
spec:
  agents:
    - name: newrelic-agent
      spec:
        model: google:gemini-2.0-flash
        tools: [newrelic_nrql_query]
    - name: servicenow-agent
      spec:
        model: google:gemini-2.0-flash
        tools: [servicenow_incident_create]
```

---

## 10. Documentation Requirements

### Tool Reference Docs
- `docs/tools/newrelic.md` - API reference, examples, auth setup
- `docs/tools/splunk.md` - Search patterns, SPL examples, async behavior
- `docs/tools/servicenow.md` - Table API, encoded queries, CMDB concepts

### Tutorial Docs
- `docs/tutorials/incident-response.md` - End-to-end RCA → ticket creation
- `docs/tutorials/observability-fleet.md` - Multi-platform monitoring

### Architecture Updates
- `docs/architecture/integrations.md` - Add new platforms to integration guide
- `docs/DOCUMENTATION_INDEX.md` - Update index with new tool docs

---

## 11. Key Files to Modify

### Core Implementation
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/Cargo.toml` - Add feature flags
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/lib.rs` - Export new tools
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/mod.rs` - Declare modules
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/newrelic.rs` - NEW
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/splunk.rs` - NEW
- `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/servicenow.rs` - NEW

### Examples
- `/Users/gshah/work/opsflow-sh/aof/examples/observability/newrelic-agent.yaml` - NEW
- `/Users/gshah/work/opsflow-sh/aof/examples/observability/splunk-investigation.yaml` - NEW
- `/Users/gshah/work/opsflow-sh/aof/examples/itsm/servicenow-automation.yaml` - NEW
- `/Users/gshah/work/opsflow-sh/aof/examples/fleets/incident-response-pipeline.yaml` - NEW

### Documentation
- `/Users/gshah/work/opsflow-sh/aof/docs/tools/newrelic.md` - NEW
- `/Users/gshah/work/opsflow-sh/aof/docs/tools/splunk.md` - NEW
- `/Users/gshah/work/opsflow-sh/aof/docs/tools/servicenow.md` - NEW
- `/Users/gshah/work/opsflow-sh/aof/docs/getting-started.md` - Add integration examples
- `/Users/gshah/work/opsflow-sh/aof/docs/DOCUMENTATION_INDEX.md` - Update index

---

## Summary

**AOF's architecture is designed for modular tool extensibility:**

1. **Trait-based abstractions** (`Tool`, `ToolExecutor`) enable zero-cost, type-safe integrations
2. **Feature flags** allow users to compile only what they need
3. **Registry pattern** centralizes tool discovery and execution
4. **Datadog implementation** serves as perfect reference for HTTP-based API tools
5. **Fleet coordination** enables multi-agent workflows with tiered execution

**Next Steps:**
1. Implement New Relic tools (Phase 1)
2. Implement Splunk tools with async search pattern (Phase 2)
3. Implement ServiceNow ITSM tools (Phase 3)
4. Create end-to-end incident response fleet example (Phase 4)

**Architecture is clean, consistent, and ready for extension.**
