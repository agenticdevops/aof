# Grafana Tool Specification

## 1. Overview

The Grafana Tool provides programmatic access to Grafana's REST API for querying data sources, managing dashboards, alerts, and annotations. This tool enables AOF agents to interact with Grafana for observability workflows, incident response, and operational intelligence.

### 1.1 Purpose

- **Query Data Sources**: Execute queries against Prometheus, Loki, Elasticsearch, and other data sources via Grafana
- **Dashboard Management**: Retrieve and inspect dashboard configurations
- **Alert Management**: List alert rules and create silences
- **Annotation Support**: Create annotations for marking significant events
- **Multi-Org Support**: Work with multiple Grafana organizations

### 1.2 Grafana API Capabilities

Grafana provides a comprehensive HTTP API for:
- **Data Source Queries**: Query any configured data source using its native query language
- **Dashboard Operations**: Search, retrieve, and inspect dashboards
- **Alerting**: List alert rules, check alert states, create silences
- **Annotations**: Add event markers to graphs
- **Organization Management**: Switch between organizations

### 1.3 Feature Flag

```toml
[features]
observability = ["reqwest", "serde_json"]
```

Grafana tools are included in the `observability` feature category alongside Prometheus, Loki, and Elasticsearch tools.

## 2. Tool Operations

### 2.1 grafana_query

Query data sources through Grafana's unified query API.

**Purpose**: Execute queries against Prometheus, Loki, or other data sources configured in Grafana.

**Parameters**:
- `endpoint` (required): Grafana server URL (e.g., `https://grafana.example.com`)
- `datasource_uid` (required): Data source UID from Grafana
- `query` (required): Query in data source's native language (PromQL, LogQL, etc.)
- `from` (optional): Start time (RFC3339 or Unix timestamp in milliseconds)
- `to` (optional): End time (RFC3339 or Unix timestamp in milliseconds)
- `max_data_points` (optional): Maximum number of data points, default: 1000
- `interval_ms` (optional): Interval in milliseconds
- `api_key` (required): Grafana API key or service account token

**Response**:
```json
{
  "success": true,
  "results": {
    "A": {
      "frames": [...],
      "status": "success"
    }
  },
  "datasource_uid": "...",
  "query": "..."
}
```

### 2.2 grafana_dashboard_get

Retrieve a dashboard by UID.

**Purpose**: Get complete dashboard JSON for inspection or analysis.

**Parameters**:
- `endpoint` (required): Grafana server URL
- `dashboard_uid` (required): Dashboard UID
- `api_key` (required): Grafana API key

**Response**:
```json
{
  "success": true,
  "dashboard": {
    "uid": "...",
    "title": "...",
    "panels": [...],
    "templating": {...}
  },
  "meta": {
    "created": "...",
    "updated": "..."
  }
}
```

### 2.3 grafana_dashboard_list

Search and list dashboards.

**Purpose**: Find dashboards by query, tags, or folder.

**Parameters**:
- `endpoint` (required): Grafana server URL
- `api_key` (required): Grafana API key
- `query` (optional): Search query string
- `tags` (optional): Array of tags to filter by
- `folder_ids` (optional): Array of folder IDs
- `limit` (optional): Maximum number of results, default: 100

**Response**:
```json
{
  "success": true,
  "dashboards": [
    {
      "uid": "...",
      "title": "...",
      "tags": ["production", "kubernetes"],
      "url": "/d/..."
    }
  ],
  "count": 42
}
```

### 2.4 grafana_alert_list

List alert rules and their current states.

**Purpose**: Query active alerts for monitoring and incident response.

**Parameters**:
- `endpoint` (required): Grafana server URL
- `api_key` (required): Grafana API key
- `dashboard_uid` (optional): Filter by dashboard UID
- `panel_id` (optional): Filter by panel ID
- `state` (optional): Filter by state: `alerting`, `ok`, `no_data`, `paused`
- `folder_id` (optional): Filter by folder ID

**Response**:
```json
{
  "success": true,
  "alerts": [
    {
      "id": 1,
      "name": "High CPU Usage",
      "state": "alerting",
      "dashboard_uid": "...",
      "panel_id": 2,
      "eval_date": "2025-01-15T10:30:00Z",
      "eval_data": {...}
    }
  ],
  "count": 5
}
```

### 2.5 grafana_alert_silence

Create an alert silence (mute).

**Purpose**: Temporarily suppress alert notifications during maintenance or known issues.

**Parameters**:
- `endpoint` (required): Grafana server URL
- `api_key` (required): Grafana API key
- `matchers` (required): Array of label matchers `[{"name": "alertname", "value": "HighCPU", "isRegex": false}]`
- `starts_at` (optional): Start time (RFC3339), default: now
- `ends_at` (required): End time (RFC3339)
- `comment` (required): Reason for silence
- `created_by` (optional): Username

**Response**:
```json
{
  "success": true,
  "silence_id": "abc123",
  "starts_at": "2025-01-15T10:00:00Z",
  "ends_at": "2025-01-15T12:00:00Z"
}
```

### 2.6 grafana_annotation_create

Create an annotation on dashboards.

**Purpose**: Mark significant events (deployments, incidents, scaling events) on graphs.

**Parameters**:
- `endpoint` (required): Grafana server URL
- `api_key` (required): Grafana API key
- `time` (optional): Event time in Unix milliseconds, default: now
- `time_end` (optional): End time for region annotations
- `text` (required): Annotation text
- `tags` (optional): Array of tags
- `dashboard_uid` (optional): Associate with specific dashboard
- `panel_id` (optional): Associate with specific panel

**Response**:
```json
{
  "success": true,
  "annotation_id": 123,
  "message": "Annotation added"
}
```

## 3. Configuration

### 3.1 Environment Variables

```bash
# Grafana endpoint
GRAFANA_ENDPOINT=https://grafana.example.com

# Authentication
GRAFANA_API_KEY=glsa_xxxxxxxxxxxxxxxxxxxxx

# Optional: Organization ID for multi-org setups
GRAFANA_ORG_ID=1
```

### 3.2 Agent YAML Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: grafana-ops
spec:
  model: google:gemini-2.5-flash

  tools:
    - grafana_query
    - grafana_dashboard_get
    - grafana_dashboard_list
    - grafana_alert_list
    - grafana_alert_silence
    - grafana_annotation_create

  environment:
    GRAFANA_ENDPOINT: "${GRAFANA_ENDPOINT}"
    GRAFANA_API_KEY: "${GRAFANA_API_KEY}"
    GRAFANA_ORG_ID: "${GRAFANA_ORG_ID:-1}"

  system_prompt: |
    You are a Grafana operations agent. You can query metrics and logs,
    inspect dashboards, manage alerts, and create annotations.

    Available data sources:
    - Prometheus (datasource_uid: prometheus-prod)
    - Loki (datasource_uid: loki-prod)
    - Elasticsearch (datasource_uid: elasticsearch-logs)
```

### 3.3 API Key Generation

```bash
# Create service account (Grafana 9.0+)
curl -X POST https://grafana.example.com/api/serviceaccounts \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "aof-agent",
    "role": "Viewer"
  }'

# Create service account token
curl -X POST https://grafana.example.com/api/serviceaccounts/1/tokens \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "aof-token"
  }'
```

## 4. Implementation Details

### 4.1 Module Structure

```rust
// crates/aof-tools/src/tools/grafana.rs

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Grafana tools
pub struct GrafanaTools;

impl GrafanaTools {
    /// Get all Grafana tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GrafanaQueryTool::new()),
            Box::new(GrafanaDashboardGetTool::new()),
            Box::new(GrafanaDashboardListTool::new()),
            Box::new(GrafanaAlertListTool::new()),
            Box::new(GrafanaAlertSilenceTool::new()),
            Box::new(GrafanaAnnotationCreateTool::new()),
        ]
    }
}
```

### 4.2 HTTP Client Setup

```rust
fn create_grafana_client(
    api_key: &str,
    org_id: Option<u64>,
) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // API Key authentication
    let auth_value = format!("Bearer {}", api_key);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
    );

    // Organization ID header (optional)
    if let Some(org) = org_id {
        headers.insert(
            "X-Grafana-Org-Id",
            reqwest::header::HeaderValue::from_str(&org.to_string())
                .map_err(|e| aof_core::AofError::tool(format!("Invalid org ID: {}", e)))?,
        );
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.3 Query API Implementation

```rust
#[async_trait]
impl Tool for GrafanaQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let datasource_uid: String = input.get_arg("datasource_uid")?;
        let query: String = input.get_arg("query")?;
        let api_key: String = input.get_arg("api_key")?;

        let from: Option<String> = input.get_arg("from").ok();
        let to: Option<String> = input.get_arg("to").ok();
        let max_data_points: i32 = input.get_arg("max_data_points").unwrap_or(1000);
        let interval_ms: Option<i32> = input.get_arg("interval_ms").ok();
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, datasource_uid = %datasource_uid, "Querying Grafana");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/ds/query", endpoint.trim_end_matches('/'));

        // Build query payload
        let mut query_payload = serde_json::json!({
            "queries": [{
                "datasource": {
                    "uid": datasource_uid
                },
                "expr": query,
                "refId": "A",
                "maxDataPoints": max_data_points
            }]
        });

        if let Some(f) = from {
            query_payload["from"] = serde_json::json!(f);
        }
        if let Some(t) = to {
            query_payload["to"] = serde_json::json!(t);
        }
        if let Some(interval) = interval_ms {
            query_payload["queries"][0]["intervalMs"] = serde_json::json!(interval);
        }

        let response = match client.post(&url).json(&query_payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Grafana query failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status != 200 {
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "results": body.get("results"),
            "datasource_uid": datasource_uid,
            "query": query
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 4.4 Authentication Headers

All requests must include:
```
Authorization: Bearer glsa_xxxxxxxxxxxxxxxxxxxxx
X-Grafana-Org-Id: 1  # Optional, for multi-org setups
Content-Type: application/json
```

### 4.5 Response Parsing

Grafana API returns consistent JSON structures:
- Success: HTTP 200 with data payload
- Error: HTTP 4xx/5xx with `{"message": "error description"}`
- Query results: `{"results": {"A": {"frames": [...]}}}` format

## 5. Tool Parameters Schema

### 5.1 grafana_query Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL (e.g., https://grafana.example.com)"
    },
    "datasource_uid": {
      "type": "string",
      "description": "Data source UID from Grafana"
    },
    "query": {
      "type": "string",
      "description": "Query in data source's native language (PromQL, LogQL, etc.)"
    },
    "from": {
      "type": "string",
      "description": "Start time (RFC3339 or Unix timestamp in milliseconds)"
    },
    "to": {
      "type": "string",
      "description": "End time (RFC3339 or Unix timestamp in milliseconds)"
    },
    "max_data_points": {
      "type": "integer",
      "description": "Maximum number of data points to return",
      "default": 1000
    },
    "interval_ms": {
      "type": "integer",
      "description": "Interval in milliseconds between data points"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key or service account token"
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID for multi-org setups"
    }
  },
  "required": ["endpoint", "datasource_uid", "query", "api_key"]
}
```

### 5.2 grafana_dashboard_get Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL"
    },
    "dashboard_uid": {
      "type": "string",
      "description": "Dashboard UID"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key"
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID"
    }
  },
  "required": ["endpoint", "dashboard_uid", "api_key"]
}
```

### 5.3 grafana_dashboard_list Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key"
    },
    "query": {
      "type": "string",
      "description": "Search query string"
    },
    "tags": {
      "type": "array",
      "description": "Tags to filter by",
      "items": {
        "type": "string"
      }
    },
    "folder_ids": {
      "type": "array",
      "description": "Folder IDs to filter by",
      "items": {
        "type": "integer"
      }
    },
    "limit": {
      "type": "integer",
      "description": "Maximum number of results",
      "default": 100
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID"
    }
  },
  "required": ["endpoint", "api_key"]
}
```

### 5.4 grafana_alert_list Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key"
    },
    "dashboard_uid": {
      "type": "string",
      "description": "Filter by dashboard UID"
    },
    "panel_id": {
      "type": "integer",
      "description": "Filter by panel ID"
    },
    "state": {
      "type": "string",
      "description": "Filter by state",
      "enum": ["alerting", "ok", "no_data", "paused"]
    },
    "folder_id": {
      "type": "integer",
      "description": "Filter by folder ID"
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID"
    }
  },
  "required": ["endpoint", "api_key"]
}
```

### 5.5 grafana_alert_silence Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key"
    },
    "matchers": {
      "type": "array",
      "description": "Label matchers for silence",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string"
          },
          "value": {
            "type": "string"
          },
          "isRegex": {
            "type": "boolean",
            "default": false
          }
        },
        "required": ["name", "value"]
      }
    },
    "starts_at": {
      "type": "string",
      "description": "Start time (RFC3339), default: now"
    },
    "ends_at": {
      "type": "string",
      "description": "End time (RFC3339)"
    },
    "comment": {
      "type": "string",
      "description": "Reason for silence"
    },
    "created_by": {
      "type": "string",
      "description": "Username"
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID"
    }
  },
  "required": ["endpoint", "api_key", "matchers", "ends_at", "comment"]
}
```

### 5.6 grafana_annotation_create Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "Grafana server URL"
    },
    "api_key": {
      "type": "string",
      "description": "Grafana API key"
    },
    "time": {
      "type": "integer",
      "description": "Event time in Unix milliseconds, default: now"
    },
    "time_end": {
      "type": "integer",
      "description": "End time for region annotations"
    },
    "text": {
      "type": "string",
      "description": "Annotation text"
    },
    "tags": {
      "type": "array",
      "description": "Tags for annotation",
      "items": {
        "type": "string"
      }
    },
    "dashboard_uid": {
      "type": "string",
      "description": "Associate with specific dashboard"
    },
    "panel_id": {
      "type": "integer",
      "description": "Associate with specific panel"
    },
    "org_id": {
      "type": "integer",
      "description": "Organization ID"
    }
  },
  "required": ["endpoint", "api_key", "text"]
}
```

## 6. Error Handling

### 6.1 Error Categories

**Authentication Errors** (HTTP 401):
```rust
if status == 401 {
    return Ok(ToolResult::error(
        "Authentication failed. Check API key and permissions.".to_string()
    ));
}
```

**Not Found** (HTTP 404):
```rust
if status == 404 {
    return Ok(ToolResult::error(format!(
        "Resource not found: {}",
        body.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
    )));
}
```

**Rate Limiting** (HTTP 429):
```rust
if status == 429 {
    let retry_after = response
        .headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    return Ok(ToolResult::error(format!(
        "Rate limited. Retry after: {}",
        retry_after
    )));
}
```

**Server Errors** (HTTP 5xx):
```rust
if status >= 500 {
    return Ok(ToolResult::error(format!(
        "Grafana server error ({}): {:?}",
        status,
        body.get("message")
    )));
}
```

### 6.2 Timeout Handling

```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()?;

// Query timeout is separate and configurable
let response = tokio::time::timeout(
    std::time::Duration::from_secs(30),
    client.post(&url).json(&payload).send()
).await
.map_err(|_| aof_core::AofError::tool("Query timeout after 30s"))?;
```

### 6.3 Network Errors

```rust
let response = match client.post(&url).json(&payload).send().await {
    Ok(r) => r,
    Err(e) => {
        if e.is_timeout() {
            return Ok(ToolResult::error("Request timeout".to_string()));
        } else if e.is_connect() {
            return Ok(ToolResult::error(format!(
                "Connection failed: {}. Check endpoint URL.",
                e
            )));
        } else {
            return Ok(ToolResult::error(format!("Network error: {}", e)));
        }
    }
};
```

### 6.4 JSON Parsing Errors

```rust
let body: serde_json::Value = match response.json().await {
    Ok(b) => b,
    Err(e) => {
        return Ok(ToolResult::error(format!(
            "Failed to parse Grafana response: {}",
            e
        )));
    }
};
```

## 7. Example Usage in Agent YAML

### 7.1 Incident Response Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: incident-responder
spec:
  model: google:gemini-2.5-flash

  tools:
    - grafana_query
    - grafana_dashboard_get
    - grafana_alert_list
    - grafana_annotation_create

  environment:
    GRAFANA_ENDPOINT: "https://grafana.prod.example.com"
    GRAFANA_API_KEY: "${GRAFANA_API_KEY}"
    PROMETHEUS_UID: "prometheus-prod"
    LOKI_UID: "loki-prod"

  system_prompt: |
    You are an incident response agent with access to Grafana.

    When investigating incidents:
    1. Query metrics from Prometheus to identify anomalies
    2. Query logs from Loki to find error patterns
    3. Create annotations marking when the incident occurred
    4. Retrieve relevant dashboards for context

    Example workflow:
    - Use grafana_query with prometheus-prod to check error rates
    - Use grafana_query with loki-prod to search for errors
    - Use grafana_annotation_create to mark incident start/end
```

### 7.2 Alert Management Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: alert-manager
spec:
  model: google:gemini-2.5-flash

  tools:
    - grafana_alert_list
    - grafana_alert_silence
    - grafana_query

  environment:
    GRAFANA_ENDPOINT: "${GRAFANA_ENDPOINT}"
    GRAFANA_API_KEY: "${GRAFANA_API_KEY}"

  system_prompt: |
    You are an alert management agent.

    Responsibilities:
    - List current firing alerts
    - Analyze alert patterns to identify noise
    - Create silences during maintenance windows
    - Verify alerts are valid before escalating

    When asked to silence alerts:
    1. Verify the alert is actually firing
    2. Query metrics to confirm it's a false positive or maintenance
    3. Create a time-bounded silence with clear comments
```

### 7.3 Dashboard Inspector Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dashboard-inspector
spec:
  model: google:gemini-2.5-flash

  tools:
    - grafana_dashboard_list
    - grafana_dashboard_get
    - grafana_query

  environment:
    GRAFANA_ENDPOINT: "${GRAFANA_ENDPOINT}"
    GRAFANA_API_KEY: "${GRAFANA_API_KEY}"

  system_prompt: |
    You are a dashboard analysis agent.

    Capabilities:
    - Search for dashboards by tags or keywords
    - Retrieve dashboard JSON to inspect panel configurations
    - Execute queries from dashboard panels to reproduce visualizations
    - Identify which data sources are used in dashboards

    Example tasks:
    - "Find all dashboards tagged with 'kubernetes'"
    - "Show me the queries used in the 'Node Exporter' dashboard"
    - "Execute the CPU usage query from panel 3 of dashboard xyz"
```

### 7.4 Deployment Annotation Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: deployment-annotator
spec:
  model: google:gemini-2.5-flash

  tools:
    - grafana_annotation_create

  environment:
    GRAFANA_ENDPOINT: "${GRAFANA_ENDPOINT}"
    GRAFANA_API_KEY: "${GRAFANA_API_KEY}"

  triggers:
    - type: webhook
      endpoint: /deployment/complete

  system_prompt: |
    You are a deployment tracking agent.

    When a deployment completes:
    1. Create an annotation marking the deployment time
    2. Include tags: deployment, version, service
    3. Add deployment metadata in the annotation text

    This helps correlate metrics changes with deployments.
```

## 8. Integration Patterns

### 8.1 Multi-Data Source Queries

```yaml
# Agent can query different data sources through Grafana
tools:
  - grafana_query

workflow:
  - step: query_prometheus
    tool: grafana_query
    args:
      datasource_uid: "prometheus-prod"
      query: "rate(http_requests_total[5m])"

  - step: query_loki
    tool: grafana_query
    args:
      datasource_uid: "loki-prod"
      query: '{job="api"} |= "error"'

  - step: query_elasticsearch
    tool: grafana_query
    args:
      datasource_uid: "elasticsearch-logs"
      query: "status:500"
```

### 8.2 Alert to Annotation Pipeline

```yaml
# When an alert fires, create annotation and query context
workflow:
  - step: list_firing_alerts
    tool: grafana_alert_list
    args:
      state: "alerting"

  - step: query_alert_metric
    tool: grafana_query
    args:
      datasource_uid: "prometheus-prod"
      query: "${alert.expr}"

  - step: mark_incident
    tool: grafana_annotation_create
    args:
      text: "Alert: ${alert.name} - ${alert.state}"
      tags: ["alert", "incident"]
```

### 8.3 Dashboard-Driven RCA

```yaml
# Use dashboard configuration to guide RCA
workflow:
  - step: find_dashboard
    tool: grafana_dashboard_list
    args:
      tags: ["${service}", "production"]

  - step: get_dashboard
    tool: grafana_dashboard_get
    args:
      dashboard_uid: "${dashboard.uid}"

  - step: execute_panel_queries
    tool: grafana_query
    for_each: "${dashboard.panels}"
    args:
      datasource_uid: "${panel.datasource.uid}"
      query: "${panel.targets[0].expr}"
```

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grafana_query_prometheus() {
        // Test with mock Grafana server
    }

    #[tokio::test]
    async fn test_grafana_dashboard_get() {
        // Test dashboard retrieval
    }

    #[tokio::test]
    async fn test_authentication_error() {
        // Test 401 handling
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        // Test 429 handling
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires real Grafana instance
async fn test_end_to_end_query() {
    let endpoint = std::env::var("GRAFANA_ENDPOINT")
        .expect("GRAFANA_ENDPOINT not set");
    let api_key = std::env::var("GRAFANA_API_KEY")
        .expect("GRAFANA_API_KEY not set");

    let tool = GrafanaQueryTool::new();
    let input = ToolInput::new(serde_json::json!({
        "endpoint": endpoint,
        "datasource_uid": "prometheus-test",
        "query": "up",
        "api_key": api_key
    }));

    let result = tool.execute(input).await.unwrap();
    assert!(result.success);
}
```

## 10. Security Considerations

### 10.1 API Key Storage

- **NEVER hardcode API keys** in agent YAML
- Use environment variables: `${GRAFANA_API_KEY}`
- Rotate keys regularly
- Use service account tokens with minimal permissions

### 10.2 Permissions

Recommended Grafana role for AOF agents:

```json
{
  "name": "AOF Agent",
  "permissions": [
    "datasources:query",
    "dashboards:read",
    "alerts:read",
    "alerts:write",  // Only for silence operations
    "annotations:create"
  ]
}
```

### 10.3 Network Security

- Use HTTPS endpoints only
- Verify TLS certificates
- Consider IP allowlisting for Grafana API access
- Use dedicated service accounts per agent

### 10.4 Rate Limiting

- Respect Grafana rate limits
- Implement exponential backoff on 429 errors
- Cache dashboard metadata when possible
- Batch queries when appropriate

## 11. Performance Optimization

### 11.1 Query Optimization

- Use appropriate time ranges
- Limit `max_data_points` to reduce response size
- Use downsampling via `interval_ms`
- Cache frequently accessed dashboards

### 11.2 Connection Pooling

```rust
// Reuse HTTP client across requests
lazy_static! {
    static ref GRAFANA_CLIENT_POOL: DashMap<String, reqwest::Client> = DashMap::new();
}

fn get_or_create_client(api_key: &str) -> reqwest::Client {
    // Connection pooling implementation
}
```

## 12. Future Enhancements

### 12.1 Planned Features

- **Dashboard Creation**: Create/update dashboards programmatically
- **Data Source Management**: List and configure data sources
- **User Management**: Manage teams and permissions
- **Folder Operations**: Organize dashboards in folders
- **Snapshot Support**: Create and share dashboard snapshots
- **Playlist Management**: Control dashboard playlists

### 12.2 Advanced Integrations

- **Alerting Rules**: Create and modify alert rules via unified alerting API
- **Recording Rules**: Manage Prometheus recording rules through Grafana
- **Template Variables**: Render dashboards with different variable values
- **Export/Import**: Backup and restore dashboard configurations

## 13. References

- [Grafana HTTP API Documentation](https://grafana.com/docs/grafana/latest/developers/http_api/)
- [Grafana Data Source Query API](https://grafana.com/docs/grafana/latest/developers/http_api/data_source/)
- [Grafana Alerting API](https://grafana.com/docs/grafana/latest/developers/http_api/alerting/)
- [Grafana Annotations API](https://grafana.com/docs/grafana/latest/developers/http_api/annotations/)
- [Service Account Tokens](https://grafana.com/docs/grafana/latest/administration/service-accounts/)
