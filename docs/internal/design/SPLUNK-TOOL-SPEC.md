# Splunk Tool Specification

## 1. Overview

The Splunk Tool provides integration with Splunk's observability and SIEM platform, enabling agents to run searches, query alerts, and send events programmatically. This tool follows the existing observability tool pattern in `aof-tools` and provides comprehensive access to Splunk's search and alerting capabilities.

### Splunk Platform Capabilities

Splunk is a leading platform for log management, SIEM, and observability:

- **Search**: SPL (Search Processing Language) for powerful data analysis
- **Alerts**: Saved searches with alert conditions
- **HEC (HTTP Event Collector)**: High-throughput event ingestion
- **Indexes**: Data organization and retention
- **Dashboards**: Visualization and reporting
- **Apps**: Extensible functionality
- **ITSI**: IT Service Intelligence

### API Architecture

Splunk uses a REST API for all operations:
- **Management Port**: 8089 (REST API)
- **HEC Port**: 8088 (Event ingestion)
- **Base URL**: `https://{splunk-host}:8089`

## 2. Tool Operations

### 2.1 Run Search (`splunk_search`)

Execute SPL (Search Processing Language) queries against Splunk data.

**Purpose**: Search and analyze log data, metrics, and events.

**API Endpoint**: `POST /services/search/v2/jobs`

**Parameters**:
- `base_url` (required): Splunk API base URL (e.g., `https://splunk.company.com:8089`)
- `token` (required): Splunk authentication token
- `query` (required): SPL search query
- `earliest_time` (optional): Start time (e.g., `-1h`, `2025-12-25T00:00:00`)
- `latest_time` (optional): End time (e.g., `now`, `2025-12-25T12:00:00`)
- `max_count` (optional): Maximum results (default: 1000)

**Example SPL Queries**:
```spl
# Error logs from web servers
index=web sourcetype=access_combined status>=500 | stats count by host

# Security events
index=security action=failure | timechart count by user

# Application metrics
index=metrics source="app_metrics" | stats avg(response_time) by endpoint
```

**Implementation Notes**:
- Splunk searches are asynchronous - create job, poll for completion, retrieve results
- Use search/jobs endpoint for complex queries
- Use export endpoint for streaming large result sets

### 2.2 Get Alerts (`splunk_alerts_list`)

List fired/triggered alerts.

**Purpose**: Retrieve alert information for incident response.

**API Endpoint**: `GET /servicesNS/-/-/alerts/fired_alerts`

**Parameters**:
- `base_url` (required): Splunk API base URL
- `token` (required): Splunk authentication token
- `count` (optional): Number of alerts to retrieve (default: 50)

**Response Format**:
```json
{
  "success": true,
  "data": {
    "alerts": [
      {
        "name": "High Error Rate",
        "triggered_time": "2025-12-25T07:30:00Z",
        "severity": "critical",
        "search_name": "errors_above_threshold",
        "result_count": 150
      }
    ]
  }
}
```

### 2.3 List Saved Searches (`splunk_saved_searches`)

List configured saved searches.

**Purpose**: Retrieve saved search configurations for management.

**API Endpoint**: `GET /servicesNS/-/-/saved/searches`

**Parameters**:
- `base_url` (required): Splunk API base URL
- `token` (required): Splunk authentication token
- `search` (optional): Filter by name pattern
- `count` (optional): Number of results (default: 50)

### 2.4 Run Saved Search (`splunk_saved_search_run`)

Execute a pre-configured saved search.

**Purpose**: Trigger existing saved searches for scheduled or on-demand analysis.

**API Endpoint**: `POST /servicesNS/-/-/saved/searches/{name}/dispatch`

**Parameters**:
- `base_url` (required): Splunk API base URL
- `token` (required): Splunk authentication token
- `name` (required): Saved search name
- `trigger_actions` (optional): Whether to trigger alert actions (default: false)

### 2.5 Send Event via HEC (`splunk_hec_send`)

Send events to Splunk via HTTP Event Collector.

**Purpose**: Ingest events from AOF agents into Splunk for analysis.

**API Endpoint**: `POST /services/collector/event` (port 8088)

**Parameters**:
- `hec_url` (required): HEC endpoint URL (e.g., `https://splunk.company.com:8088`)
- `hec_token` (required): HEC token
- `event` (required): Event data (JSON object)
- `source` (optional): Event source
- `sourcetype` (optional): Event source type
- `index` (optional): Target index
- `host` (optional): Host value

**Request Format**:
```json
{
  "event": {
    "message": "AOF agent completed task",
    "agent": "diagnostic-agent",
    "status": "success"
  },
  "source": "aof",
  "sourcetype": "aof:agent",
  "index": "main"
}
```

### 2.6 List Indexes (`splunk_indexes_list`)

List available Splunk indexes.

**Purpose**: Discover available data sources for querying.

**API Endpoint**: `GET /services/data/indexes`

**Parameters**:
- `base_url` (required): Splunk API base URL
- `token` (required): Splunk authentication token

## 3. Configuration

### 3.1 Authentication

Splunk supports multiple authentication methods:

**Bearer Token (Recommended)**:
```
Authorization: Bearer <token>
```

**Splunk Token**:
```
Authorization: Splunk <token>
```

**Basic Auth**:
```
Authorization: Basic <base64(username:password)>
```

### 3.2 Configuration Schema

```yaml
# Environment variables (recommended)
SPLUNK_BASE_URL: "https://splunk.company.com:8089"
SPLUNK_TOKEN: "eyJraWQiOiJzcGx..."
SPLUNK_HEC_URL: "https://splunk.company.com:8088"
SPLUNK_HEC_TOKEN: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

# Or in agent configuration
tools:
  - name: splunk_search
    config:
      base_url: "${SPLUNK_BASE_URL}"
      token: "${SPLUNK_TOKEN}"
```

## 4. Implementation Details

### 4.1 Tool Structure

```rust
// File: crates/aof-tools/src/tools/splunk.rs

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

/// Collection of all Splunk tools
pub struct SplunkTools;

impl SplunkTools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(SplunkSearchTool::new()),
            Box::new(SplunkAlertsListTool::new()),
            Box::new(SplunkSavedSearchesTool::new()),
            Box::new(SplunkSavedSearchRunTool::new()),
            Box::new(SplunkHecSendTool::new()),
            Box::new(SplunkIndexesListTool::new()),
        ]
    }
}
```

### 4.2 HTTP Client Setup

```rust
async fn create_splunk_client(token: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    // Splunk often uses self-signed certs
    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(120))
        .danger_accept_invalid_certs(false) // Set to true only in dev
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.3 Async Search Pattern

Splunk searches are asynchronous - you must:
1. Create a search job
2. Poll for job completion
3. Retrieve results

```rust
async fn run_search(
    client: &Client,
    base_url: &str,
    query: &str,
    earliest_time: Option<&str>,
    latest_time: Option<&str>,
    max_count: u32,
) -> AofResult<ToolResult> {
    // 1. Create search job
    let create_url = format!("{}/services/search/v2/jobs", base_url);
    let mut form_data = vec![
        ("search", query.to_string()),
        ("output_mode", "json".to_string()),
        ("exec_mode", "normal".to_string()),
    ];

    if let Some(earliest) = earliest_time {
        form_data.push(("earliest_time", earliest.to_string()));
    }
    if let Some(latest) = latest_time {
        form_data.push(("latest_time", latest.to_string()));
    }

    let create_response = client
        .post(&create_url)
        .form(&form_data)
        .send()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create search job: {}", e)))?;

    let job_response: serde_json::Value = create_response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to parse job response: {}", e)))?;

    let sid = job_response["sid"]
        .as_str()
        .ok_or_else(|| aof_core::AofError::tool("No SID in response".to_string()))?;

    // 2. Poll for completion
    let status_url = format!("{}/services/search/v2/jobs/{}", base_url, sid);
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let status_response = client
            .get(&status_url)
            .query(&[("output_mode", "json")])
            .send()
            .await?;

        let status: serde_json::Value = status_response.json().await?;
        let dispatch_state = status["entry"][0]["content"]["dispatchState"]
            .as_str()
            .unwrap_or("");

        if dispatch_state == "DONE" || dispatch_state == "FAILED" {
            break;
        }
    }

    // 3. Retrieve results
    let results_url = format!("{}/services/search/v2/jobs/{}/results", base_url, sid);
    let results_response = client
        .get(&results_url)
        .query(&[("output_mode", "json"), ("count", &max_count.to_string())])
        .send()
        .await?;

    let results: serde_json::Value = results_response.json().await?;

    Ok(ToolResult::success(results))
}
```

### 4.4 HEC Implementation

```rust
pub struct SplunkHecSendTool {
    config: ToolConfig,
}

impl SplunkHecSendTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "hec_url": {
                    "type": "string",
                    "description": "Splunk HEC endpoint URL (e.g., https://splunk:8088)"
                },
                "hec_token": {
                    "type": "string",
                    "description": "HEC token (GUID format)"
                },
                "event": {
                    "type": "object",
                    "description": "Event data to send"
                },
                "source": {
                    "type": "string",
                    "description": "Event source",
                    "default": "aof"
                },
                "sourcetype": {
                    "type": "string",
                    "description": "Event sourcetype",
                    "default": "aof:event"
                },
                "index": {
                    "type": "string",
                    "description": "Target index"
                }
            }),
            vec!["hec_url", "hec_token", "event"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_hec_send",
                "Send events to Splunk via HTTP Event Collector for ingestion.",
                parameters,
                30,
            ),
        }
    }
}

#[async_trait]
impl Tool for SplunkHecSendTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let hec_url: String = input.get_arg("hec_url")?;
        let hec_token: String = input.get_arg("hec_token")?;
        let event: serde_json::Value = input.get_arg("event")?;
        let source: String = input.get_arg("source").unwrap_or_else(|_| "aof".to_string());
        let sourcetype: String = input.get_arg("sourcetype").unwrap_or_else(|_| "aof:event".to_string());
        let index: Option<String> = input.get_arg("index").ok();

        let url = format!("{}/services/collector/event", hec_url.trim_end_matches('/'));

        let mut payload = json!({
            "event": event,
            "source": source,
            "sourcetype": sourcetype
        });

        if let Some(idx) = index {
            payload["index"] = json!(idx);
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create client: {}", e)))?;

        let response = client
            .post(&url)
            .header("Authorization", format!("Splunk {}", hec_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("HEC request failed: {}", e)))?;

        let status = response.status().as_u16();
        let body: serde_json::Value = response.json().await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to parse response: {}", e)))?;

        if status >= 400 {
            return Ok(ToolResult::error(format!("HEC error {}: {:?}", status, body)));
        }

        Ok(ToolResult::success(body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

## 5. Tool Parameters Schema

### 5.1 Common Parameters

```json
{
  "base_url": {
    "type": "string",
    "description": "Splunk REST API base URL (https://splunk:8089)"
  },
  "token": {
    "type": "string",
    "description": "Splunk authentication token"
  }
}
```

### 5.2 Search Schema

```json
{
  "type": "object",
  "properties": {
    "base_url": { "type": "string" },
    "token": { "type": "string" },
    "query": {
      "type": "string",
      "description": "SPL search query"
    },
    "earliest_time": {
      "type": "string",
      "description": "Start time (-1h, -1d@d, 2025-12-25T00:00:00)"
    },
    "latest_time": {
      "type": "string",
      "description": "End time (now, @d, 2025-12-25T12:00:00)"
    },
    "max_count": {
      "type": "integer",
      "description": "Maximum results to return",
      "default": 1000
    }
  },
  "required": ["base_url", "token", "query"]
}
```

### 5.3 HEC Send Schema

```json
{
  "type": "object",
  "properties": {
    "hec_url": {
      "type": "string",
      "description": "HEC endpoint URL (https://splunk:8088)"
    },
    "hec_token": {
      "type": "string",
      "description": "HEC token"
    },
    "event": {
      "type": "object",
      "description": "Event data"
    },
    "source": {
      "type": "string",
      "default": "aof"
    },
    "sourcetype": {
      "type": "string",
      "default": "aof:event"
    },
    "index": {
      "type": "string",
      "description": "Target index"
    }
  },
  "required": ["hec_url", "hec_token", "event"]
}
```

## 6. Error Handling

### 6.1 Common Error Scenarios

**Authentication Errors (401)**:
```json
{
  "messages": [
    {
      "type": "WARN",
      "text": "call not properly authenticated"
    }
  ]
}
```

**Search Syntax Errors (400)**:
```json
{
  "messages": [
    {
      "type": "ERROR",
      "text": "Error in 'search' command: Invalid search syntax"
    }
  ]
}
```

**Rate Limiting (429)**:
- Splunk Intelligence Management: 60 calls/min per user
- Core platform: Configurable per deployment

### 6.2 Error Handling Strategy

```rust
async fn handle_splunk_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("{} parse error: {}", operation, e)))?;

    // Check for Splunk error messages
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        let errors: Vec<String> = messages
            .iter()
            .filter(|m| m.get("type").and_then(|t| t.as_str()) == Some("ERROR"))
            .filter_map(|m| m.get("text").and_then(|t| t.as_str()))
            .map(String::from)
            .collect();

        if !errors.is_empty() {
            return Ok(ToolResult::error(format!("{} errors: {}", operation, errors.join("; "))));
        }
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!("{} HTTP {}: {:?}", operation, status, body)));
    }

    Ok(ToolResult::success(body))
}
```

## 7. Example Usage in Agent YAML

### 7.1 Log Analysis Agent

```yaml
name: splunk-log-analyst
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - splunk_search
  - splunk_indexes_list

config:
  splunk:
    base_url: "${SPLUNK_BASE_URL}"
    token: "${SPLUNK_TOKEN}"

system_prompt: |
  You are a Splunk log analysis agent.

  Use SPL queries to:
  - Search for errors and anomalies
  - Analyze access patterns
  - Investigate security events
  - Generate statistics and trends

  Common SPL patterns:
  - stats count by field
  - timechart span=1h count
  - rex field=_raw "pattern"
  - transaction startswith="start" endswith="end"

user_prompt: |
  Find all 5xx errors from the web index in the last hour and show the top 10 URLs.
```

### 7.2 Security Alert Agent

```yaml
name: splunk-security-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - splunk_search
  - splunk_alerts_list
  - splunk_saved_search_run

config:
  splunk:
    base_url: "${SPLUNK_BASE_URL}"
    token: "${SPLUNK_TOKEN}"

system_prompt: |
  You are a Splunk security analysis agent.

  Capabilities:
  1. List fired security alerts
  2. Run security-related searches
  3. Trigger saved security searches
  4. Analyze attack patterns

  Focus on:
  - Failed authentication attempts
  - Unusual access patterns
  - Privilege escalation
  - Data exfiltration indicators

user_prompt: |
  Check for any brute force login attempts in the last 24 hours.
```

### 7.3 Event Ingestion Agent

```yaml
name: splunk-event-sender
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - splunk_hec_send

config:
  splunk:
    hec_url: "${SPLUNK_HEC_URL}"
    hec_token: "${SPLUNK_HEC_TOKEN}"

system_prompt: |
  You are a Splunk event ingestion agent.

  Send structured events to Splunk for:
  - Agent activity logging
  - Task completion events
  - Error and exception events
  - Audit trail

  Always include:
  - Timestamp
  - Agent name
  - Action type
  - Status
  - Relevant context

user_prompt: |
  Log a successful task completion event for the diagnostic workflow.
```

## 8. Implementation Checklist

- [ ] Create `SplunkTools` struct with all 6 operations
- [ ] Implement Bearer token authentication
- [ ] Implement async search pattern (create → poll → retrieve)
- [ ] Implement HEC event sending
- [ ] Implement saved search listing and execution
- [ ] Implement alert listing
- [ ] Add SSL/TLS configuration options
- [ ] Add retry logic for async searches
- [ ] Create comprehensive error handling
- [ ] Write unit tests for each tool
- [ ] Add integration tests with mock Splunk API
- [ ] Document in `docs/tools/splunk.md`
- [ ] Add examples to `examples/agents/splunk-*.yaml`
- [ ] Add feature flag `siem` in `Cargo.toml`
- [ ] Export tools in `crates/aof-tools/src/lib.rs`

## 9. Rate Limits and Best Practices

### 9.1 Rate Limits

- **REST API**: No hard limits (deployment-specific)
- **Search concurrency**: Default 5-10 concurrent searches
- **Result limit**: 50,000 rows per search
- **HEC**: High throughput, batch recommended

### 9.2 Best Practices

1. **Always use time bounds**: Include `earliest_time` and `latest_time`
2. **Use export for large results**: `/jobs/{sid}/export` for streaming
3. **Limit fields**: Use `| fields` to reduce data transfer
4. **Paginate results**: Use `offset` and `count` parameters
5. **Batch HEC events**: Send multiple events in single request

## 10. Security Considerations

### 10.1 Token Management

- Store tokens in environment variables
- Use service accounts with minimal permissions
- Rotate tokens periodically
- Never log tokens

### 10.2 Network Security

- Always use HTTPS
- Validate SSL certificates in production
- Use network segmentation
- Implement firewall rules for API ports

---

**Document Version**: 1.0
**Last Updated**: 2025-12-25
**Author**: AOF Hive Mind Swarm
**Status**: Ready for Implementation
