# Datadog Tool Specification

## 1. Overview

The Datadog Tool provides integration with Datadog's observability platform APIs, enabling agents to query metrics, logs, monitors, and post events programmatically. This tool follows the existing observability tool pattern in `aof-tools` and provides comprehensive access to Datadog's monitoring capabilities.

### Datadog Platform Capabilities

Datadog is a cloud-scale monitoring and analytics platform that provides:

- **Metrics**: Time-series data for infrastructure and application monitoring
- **Logs**: Centralized log aggregation and analysis
- **APM**: Application Performance Monitoring with distributed tracing
- **Monitors**: Alert configuration and management
- **Events**: Event stream for tracking deployments, incidents, and custom events
- **Downtime**: Scheduled maintenance window management
- **Dashboards**: Visualization and reporting (read-only via API)
- **Service Catalog**: Service dependencies and ownership

### API Regions

Datadog supports multiple regions with different endpoints:
- **US1** (default): `https://api.datadoghq.com`
- **US3**: `https://api.us3.datadoghq.com`
- **US5**: `https://api.us5.datadoghq.com`
- **EU1**: `https://api.datadoghq.eu`
- **AP1**: `https://api.ap1.datadoghq.com`
- **US1-FED**: `https://api.ddog-gov.com`

## 2. Tool Operations

### 2.1 Metrics Query (`datadog_metric_query`)

Query time-series metrics using Datadog's metric query language.

**Purpose**: Retrieve metric data for analysis, dashboards, and alerting decisions.

**API Endpoint**: `GET /api/v1/query`

**Parameters**:
- `query` (required): Metric query in Datadog query language (e.g., `avg:system.cpu.user{*}`)
- `from` (required): Start time (Unix timestamp in seconds or ISO 8601)
- `to` (required): End time (Unix timestamp in seconds or ISO 8601)

**Example Query Language**:
```
avg:system.cpu.user{host:web-01}
sum:kubernetes.memory.usage{kube_namespace:production}
avg:trace.servlet.request.duration{service:api,env:prod}.rollup(avg, 60)
```

**Response Format**:
```json
{
  "status": "ok",
  "res_type": "time_series",
  "series": [
    {
      "aggr": "avg",
      "metric": "system.cpu.user",
      "tag_set": ["host:web-01"],
      "pointlist": [[1639065600000, 23.5], [1639065660000, 24.1]]
    }
  ],
  "from_date": 1639065600000,
  "to_date": 1639069200000
}
```

### 2.2 Log Query (`datadog_log_query`)

Search and analyze logs using Datadog's log query syntax.

**Purpose**: Retrieve log entries for debugging, troubleshooting, and audit trails.

**API Endpoint**: `POST /api/v2/logs/events/search`

**Parameters**:
- `query` (required): Log search query (e.g., `service:web status:error`)
- `from` (required): Start time (ISO 8601 format)
- `to` (required): End time (ISO 8601 format)
- `limit` (optional): Maximum number of logs to return (default: 50, max: 1000)
- `sort` (optional): Sort order - `timestamp` or `-timestamp` (default: `-timestamp`)
- `indexes` (optional): Array of log indexes to search (default: all)

**Example Query Syntax**:
```
service:api status:error
source:kubernetes @http.status_code:[500 TO 599]
host:prod-* -status:info
@user.email:admin@example.com
```

**Response Format**:
```json
{
  "data": [
    {
      "id": "AQAAAYRh...",
      "attributes": {
        "timestamp": "2024-01-15T10:30:45.123Z",
        "status": "error",
        "message": "Database connection timeout",
        "service": "api",
        "tags": ["env:production", "version:1.2.3"],
        "attributes": {
          "@http.status_code": 500,
          "@error.message": "Connection timeout after 30s"
        }
      }
    }
  ],
  "meta": {
    "page": {
      "after": "eyJhZnRlciI6..."
    }
  }
}
```

### 2.3 Monitor List (`datadog_monitor_list`)

List all monitors or filter by tags/status.

**Purpose**: Retrieve monitor configurations and current states for automation workflows.

**API Endpoint**: `GET /api/v1/monitor`

**Parameters**:
- `tags` (optional): Filter by tags (e.g., `"env:prod,service:api"`)
- `monitor_tags` (optional): Filter by monitor tags
- `group_states` (optional): Filter by monitor state (`all`, `alert`, `warn`, `no data`)
- `name` (optional): Filter by monitor name (substring match)
- `with_downtimes` (optional): Include downtime information (default: false)

**Response Format**:
```json
[
  {
    "id": 12345678,
    "name": "High CPU Usage on Production",
    "type": "metric alert",
    "query": "avg(last_5m):avg:system.cpu.user{env:prod} > 90",
    "message": "CPU usage is high @pagerduty-production",
    "tags": ["env:prod", "team:platform"],
    "options": {
      "thresholds": {
        "critical": 90,
        "warning": 80
      },
      "notify_no_data": true,
      "no_data_timeframe": 20
    },
    "overall_state": "OK",
    "created": "2024-01-01T00:00:00.000000+00:00",
    "modified": "2024-01-15T12:30:00.000000+00:00"
  }
]
```

### 2.4 Monitor Mute (`datadog_monitor_mute`)

Mute a specific monitor or monitor group.

**Purpose**: Temporarily silence alerts during maintenance or known issues.

**API Endpoint**: `POST /api/v1/monitor/{monitor_id}/mute`

**Parameters**:
- `monitor_id` (required): Monitor ID to mute
- `scope` (optional): Scope to mute (e.g., `"host:web-01"` or `"env:staging"`)
- `end` (optional): End time for mute (Unix timestamp). If not provided, mute is indefinite
- `override` (optional): Override existing mute settings (default: false)

**Response Format**:
```json
{
  "id": 12345678,
  "name": "High CPU Usage on Production",
  "overall_state": "No Data"
}
```

### 2.5 Monitor Unmute (`datadog_monitor_unmute`)

Unmute a previously muted monitor.

**Purpose**: Restore alerting after maintenance or resolution.

**API Endpoint**: `POST /api/v1/monitor/{monitor_id}/unmute`

**Parameters**:
- `monitor_id` (required): Monitor ID to unmute
- `scope` (optional): Scope to unmute (must match mute scope)
- `all_scopes` (optional): Unmute all scopes for this monitor (default: false)

### 2.6 Event Post (`datadog_event_post`)

Post custom events to the Datadog event stream.

**Purpose**: Track deployments, incidents, configuration changes, and custom milestones.

**API Endpoint**: `POST /api/v1/events`

**Parameters**:
- `title` (required): Event title (max 500 characters)
- `text` (required): Event description (supports Markdown, max 4000 characters)
- `priority` (optional): Event priority (`normal` or `low`, default: `normal`)
- `tags` (optional): Array of tags (e.g., `["env:prod", "version:1.2.3"]`)
- `alert_type` (optional): Event severity (`error`, `warning`, `info`, `success`, default: `info`)
- `source_type_name` (optional): Source type (e.g., `"jenkins"`, `"kubernetes"`)
- `date_happened` (optional): Event timestamp (Unix timestamp, default: now)
- `aggregation_key` (optional): Key for event aggregation

**Response Format**:
```json
{
  "status": "ok",
  "event": {
    "id": 987654321,
    "title": "Deployment: API v1.2.3 to Production",
    "text": "Successfully deployed API version 1.2.3 to production environment",
    "tags": ["env:prod", "service:api", "version:1.2.3"],
    "alert_type": "success",
    "priority": "normal",
    "date_happened": 1639065600,
    "url": "/event/event?id=987654321"
  }
}
```

### 2.7 Downtime Create (`datadog_downtime_create`)

Schedule a downtime window to suppress alerts.

**Purpose**: Plan maintenance windows or suppress alerts during known outages.

**API Endpoint**: `POST /api/v1/downtime`

**Parameters**:
- `scope` (required): Array of scopes (e.g., `["host:web-*", "env:prod"]`)
- `start` (optional): Start time (Unix timestamp, default: now)
- `end` (optional): End time (Unix timestamp). Required if not using recurrence
- `recurrence` (optional): Recurrence rule (see Downtime Recurrence below)
- `message` (optional): Message to include with notifications
- `timezone` (optional): Timezone for recurrence (default: `"UTC"`)
- `monitor_id` (optional): Specific monitor ID (if omitted, applies to all monitors matching scope)
- `monitor_tags` (optional): Array of monitor tags to match

**Downtime Recurrence Format**:
```json
{
  "type": "weeks",
  "period": 1,
  "week_days": ["Mon", "Tue", "Wed", "Thu", "Fri"],
  "until_date": 1640995200
}
```

**Response Format**:
```json
{
  "id": 555666777,
  "scope": ["env:staging"],
  "start": 1639065600,
  "end": 1639069200,
  "message": "Weekly maintenance window",
  "recurrence": {
    "type": "weeks",
    "period": 1,
    "week_days": ["Sat"],
    "until_date": null
  },
  "timezone": "America/New_York",
  "active": true,
  "disabled": false,
  "creator_id": 123456,
  "updater_id": null
}
```

### 2.8 Downtime Cancel (`datadog_downtime_cancel`)

Cancel an active or scheduled downtime.

**Purpose**: Remove scheduled maintenance or end downtime early.

**API Endpoint**: `DELETE /api/v1/downtime/{downtime_id}`

**Parameters**:
- `downtime_id` (required): Downtime ID to cancel

**Response**: HTTP 204 No Content on success

## 3. Configuration

### 3.1 Authentication

Datadog uses dual API key authentication:

**Headers Required**:
```
DD-API-KEY: <api_key>
DD-APPLICATION-KEY: <application_key>
```

**API Key**: Identifies the organization
**Application Key**: Identifies the user/service account with specific permissions

### 3.2 Configuration Schema

```yaml
# Environment variables (recommended)
DATADOG_API_KEY: "abc123..."
DATADOG_APP_KEY: "xyz789..."
DATADOG_SITE: "datadoghq.com"  # or datadoghq.eu, us3.datadoghq.com, etc.

# Or in agent configuration
tools:
  - name: datadog_metric_query
    config:
      endpoint: "https://api.datadoghq.com"
      api_key: "${DATADOG_API_KEY}"
      app_key: "${DATADOG_APP_KEY}"
```

### 3.3 Endpoint Configuration

The tool should construct the full endpoint URL based on the site parameter:

```rust
fn get_endpoint(site: &str) -> String {
    match site {
        "datadoghq.com" | "us1" => "https://api.datadoghq.com",
        "datadoghq.eu" | "eu1" => "https://api.datadoghq.eu",
        "us3.datadoghq.com" | "us3" => "https://api.us3.datadoghq.com",
        "us5.datadoghq.com" | "us5" => "https://api.us5.datadoghq.com",
        "ap1.datadoghq.com" | "ap1" => "https://api.ap1.datadoghq.com",
        "ddog-gov.com" | "us1-fed" => "https://api.ddog-gov.com",
        custom => custom, // Allow custom endpoints
    }.to_string()
}
```

## 4. Implementation Details

### 4.1 Tool Structure

Following the existing observability tools pattern:

```rust
// File: crates/aof-tools/src/tools/observability.rs (extend existing)

pub struct DatadogTools;

impl DatadogTools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(DatadogMetricQueryTool::new()),
            Box::new(DatadogLogQueryTool::new()),
            Box::new(DatadogMonitorListTool::new()),
            Box::new(DatadogMonitorMuteTool::new()),
            Box::new(DatadogMonitorUnmuteTool::new()),
            Box::new(DatadogEventPostTool::new()),
            Box::new(DatadogDowntimeCreateTool::new()),
            Box::new(DatadogDowntimeCancelTool::new()),
        ]
    }
}

// Individual tool implementations
pub struct DatadogMetricQueryTool {
    config: ToolConfig,
}

impl DatadogMetricQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY)"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY)"
                },
                "query": {
                    "type": "string",
                    "description": "Metric query (e.g., 'avg:system.cpu.user{*}')"
                },
                "from": {
                    "type": "string",
                    "description": "Start time (Unix timestamp or ISO 8601)"
                },
                "to": {
                    "type": "string",
                    "description": "End time (Unix timestamp or ISO 8601)"
                }
            }),
            vec!["api_key", "app_key", "query", "from", "to"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_metric_query",
                "Query Datadog metrics using Datadog query language. Returns time-series data.",
                parameters,
                60,
            ),
        }
    }
}
```

### 4.2 HTTP Client Setup

```rust
use reqwest::Client;

async fn create_datadog_client(
    api_key: &str,
    app_key: &str,
) -> Result<Client, AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "DD-API-KEY",
        reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|e| AofError::tool(format!("Invalid API key: {}", e)))?,
    );

    headers.insert(
        "DD-APPLICATION-KEY",
        reqwest::header::HeaderValue::from_str(app_key)
            .map_err(|e| AofError::tool(format!("Invalid app key: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.3 Response Parsing

```rust
async fn handle_datadog_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();

    // Parse response body
    let body: serde_json::Value = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            return Ok(ToolResult::error(format!(
                "{} failed to parse response: {}",
                operation, e
            )));
        }
    };

    // Check for errors
    if status >= 400 {
        let error_msg = body
            .get("errors")
            .or_else(|| body.get("error"))
            .map(|e| format!("{:?}", e))
            .unwrap_or_else(|| "Unknown error".to_string());

        return Ok(ToolResult::error(format!(
            "{} returned status {}: {}",
            operation, status, error_msg
        )));
    }

    // Success
    Ok(ToolResult::success(body))
}
```

### 4.4 Time Parsing Utilities

```rust
fn parse_time_param(time: &str) -> Result<i64, String> {
    // Try parsing as Unix timestamp
    if let Ok(ts) = time.parse::<i64>() {
        return Ok(ts);
    }

    // Try parsing as ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time) {
        return Ok(dt.timestamp());
    }

    // Try parsing relative time (e.g., "-1h", "-30m")
    if time.starts_with('-') {
        if let Some(duration) = parse_relative_time(&time[1..]) {
            let now = chrono::Utc::now().timestamp();
            return Ok(now - duration);
        }
    }

    Err(format!("Invalid time format: {}. Expected Unix timestamp, ISO 8601, or relative time like '-1h'", time))
}

fn parse_relative_time(s: &str) -> Option<i64> {
    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    match unit {
        "s" => Some(num),
        "m" => Some(num * 60),
        "h" => Some(num * 3600),
        "d" => Some(num * 86400),
        _ => None,
    }
}
```

## 5. Tool Parameters Schema

### 5.1 Common Parameters

All Datadog tools share these common parameters:

```json
{
  "endpoint": {
    "type": "string",
    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
    "default": "https://api.datadoghq.com"
  },
  "api_key": {
    "type": "string",
    "description": "Datadog API key (DD-API-KEY header). Can use env var DATADOG_API_KEY"
  },
  "app_key": {
    "type": "string",
    "description": "Datadog Application key (DD-APPLICATION-KEY header). Can use env var DATADOG_APP_KEY"
  }
}
```

### 5.2 Metric Query Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "query": {
      "type": "string",
      "description": "Datadog metric query (e.g., 'avg:system.cpu.user{*}')"
    },
    "from": {
      "type": "string",
      "description": "Start time (Unix timestamp, ISO 8601, or relative like '-1h')"
    },
    "to": {
      "type": "string",
      "description": "End time (Unix timestamp, ISO 8601, or relative like 'now')"
    }
  },
  "required": ["api_key", "app_key", "query", "from", "to"]
}
```

### 5.3 Log Query Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "query": {
      "type": "string",
      "description": "Log search query (e.g., 'service:api status:error')"
    },
    "from": {
      "type": "string",
      "description": "Start time (ISO 8601 format)"
    },
    "to": {
      "type": "string",
      "description": "End time (ISO 8601 format)"
    },
    "limit": {
      "type": "integer",
      "description": "Maximum logs to return (max: 1000)",
      "default": 50,
      "maximum": 1000
    },
    "sort": {
      "type": "string",
      "description": "Sort order: 'timestamp' or '-timestamp'",
      "enum": ["timestamp", "-timestamp"],
      "default": "-timestamp"
    },
    "indexes": {
      "type": "array",
      "description": "Log indexes to search",
      "items": { "type": "string" }
    }
  },
  "required": ["api_key", "app_key", "query", "from", "to"]
}
```

### 5.4 Monitor List Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "tags": {
      "type": "string",
      "description": "Filter by tags (comma-separated: 'env:prod,service:api')"
    },
    "monitor_tags": {
      "type": "string",
      "description": "Filter by monitor-specific tags"
    },
    "group_states": {
      "type": "string",
      "description": "Filter by state",
      "enum": ["all", "alert", "warn", "no data"]
    },
    "name": {
      "type": "string",
      "description": "Filter by monitor name (substring match)"
    },
    "with_downtimes": {
      "type": "boolean",
      "description": "Include downtime information",
      "default": false
    }
  },
  "required": ["api_key", "app_key"]
}
```

### 5.5 Monitor Mute Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "monitor_id": {
      "type": "integer",
      "description": "Monitor ID to mute"
    },
    "scope": {
      "type": "string",
      "description": "Scope to mute (e.g., 'host:web-01', 'env:staging')"
    },
    "end": {
      "type": "integer",
      "description": "End time for mute (Unix timestamp). Omit for indefinite"
    },
    "override": {
      "type": "boolean",
      "description": "Override existing mute settings",
      "default": false
    }
  },
  "required": ["api_key", "app_key", "monitor_id"]
}
```

### 5.6 Event Post Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "title": {
      "type": "string",
      "description": "Event title (max 500 chars)",
      "maxLength": 500
    },
    "text": {
      "type": "string",
      "description": "Event description (supports Markdown, max 4000 chars)",
      "maxLength": 4000
    },
    "priority": {
      "type": "string",
      "description": "Event priority",
      "enum": ["normal", "low"],
      "default": "normal"
    },
    "tags": {
      "type": "array",
      "description": "Event tags",
      "items": { "type": "string" }
    },
    "alert_type": {
      "type": "string",
      "description": "Event severity",
      "enum": ["error", "warning", "info", "success"],
      "default": "info"
    },
    "source_type_name": {
      "type": "string",
      "description": "Source system (e.g., 'jenkins', 'kubernetes')"
    },
    "date_happened": {
      "type": "integer",
      "description": "Event timestamp (Unix timestamp, default: now)"
    },
    "aggregation_key": {
      "type": "string",
      "description": "Key for event aggregation"
    }
  },
  "required": ["api_key", "app_key", "title", "text"]
}
```

### 5.7 Downtime Create Schema

```json
{
  "type": "object",
  "properties": {
    "endpoint": { "type": "string", "default": "https://api.datadoghq.com" },
    "api_key": { "type": "string" },
    "app_key": { "type": "string" },
    "scope": {
      "type": "array",
      "description": "Scopes to apply downtime (e.g., ['host:web-*', 'env:prod'])",
      "items": { "type": "string" }
    },
    "start": {
      "type": "integer",
      "description": "Start time (Unix timestamp, default: now)"
    },
    "end": {
      "type": "integer",
      "description": "End time (Unix timestamp). Required if no recurrence"
    },
    "recurrence": {
      "type": "object",
      "description": "Recurrence configuration",
      "properties": {
        "type": {
          "type": "string",
          "enum": ["days", "weeks", "months", "years"]
        },
        "period": {
          "type": "integer",
          "description": "Recurrence period"
        },
        "week_days": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
          }
        },
        "until_date": {
          "type": "integer",
          "description": "Recurrence end date (Unix timestamp)"
        }
      }
    },
    "message": {
      "type": "string",
      "description": "Message to include with notifications"
    },
    "timezone": {
      "type": "string",
      "description": "Timezone for recurrence (default: UTC)",
      "default": "UTC"
    },
    "monitor_id": {
      "type": "integer",
      "description": "Specific monitor ID (optional)"
    },
    "monitor_tags": {
      "type": "array",
      "description": "Monitor tags to match",
      "items": { "type": "string" }
    }
  },
  "required": ["api_key", "app_key", "scope"]
}
```

## 6. Error Handling

### 6.1 Common Error Scenarios

**Authentication Errors (403)**:
```json
{
  "errors": [
    "Forbidden: Invalid API key or Application key"
  ]
}
```

**Rate Limiting (429)**:
```json
{
  "errors": [
    "Rate limit exceeded. Try again in 60 seconds"
  ]
}
```

**Invalid Query Syntax (400)**:
```json
{
  "errors": [
    "Invalid query syntax: unknown aggregation function 'averge'"
  ]
}
```

**Resource Not Found (404)**:
```json
{
  "errors": [
    "Monitor not found: 12345678"
  ]
}
```

### 6.2 Error Handling Strategy

```rust
impl Tool for DatadogMetricQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let query: String = input.get_arg("query")?;
        let from: String = input.get_arg("from")?;
        let to: String = input.get_arg("to")?;

        debug!(
            endpoint = %endpoint,
            query = %query,
            "Querying Datadog metrics"
        );

        // Parse time parameters
        let from_ts = match parse_time_param(&from) {
            Ok(ts) => ts,
            Err(e) => return Ok(ToolResult::error(format!("Invalid 'from' time: {}", e))),
        };

        let to_ts = match parse_time_param(&to) {
            Ok(ts) => ts,
            Err(e) => return Ok(ToolResult::error(format!("Invalid 'to' time: {}", e))),
        };

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request
        let url = format!("{}/api/v1/query", endpoint.trim_end_matches('/'));
        let params = [
            ("query", query.as_str()),
            ("from", &from_ts.to_string()),
            ("to", &to_ts.to_string()),
        ];

        // Execute request
        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Datadog metric query failed: {}", e)));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog metric query").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 6.3 Retry Logic

For transient errors (rate limiting, network issues):

```rust
async fn execute_with_retry<F, Fut>(
    operation: F,
    max_retries: u32,
) -> AofResult<ToolResult>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = AofResult<ToolResult>>,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                // Check if result indicates rate limiting
                if let Some(error) = result.error() {
                    if error.contains("429") || error.contains("rate limit") {
                        attempt += 1;
                        if attempt >= max_retries {
                            return Ok(result);
                        }

                        let backoff = std::time::Duration::from_secs(2u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                        continue;
                    }
                }

                return Ok(result);
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 7. Example Usage in Agent YAML

### 7.1 Basic Metric Query Agent

```yaml
name: datadog-metric-monitor
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_metric_query

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a Datadog metrics monitoring agent.

  Query Datadog metrics to analyze system performance and alert on anomalies.

  Available metrics include:
  - system.cpu.user - CPU usage
  - system.mem.used - Memory usage
  - kubernetes.* - Kubernetes metrics
  - trace.* - APM traces

  Use the datadog_metric_query tool to retrieve time-series data.

user_prompt: |
  Check CPU usage across all production hosts for the last hour.
  Alert if any host exceeds 90% CPU.
```

### 7.2 Log Analysis Agent

```yaml
name: datadog-log-analyzer
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_log_query

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a log analysis agent using Datadog.

  Search logs to troubleshoot issues, identify errors, and analyze patterns.

  Log query syntax:
  - service:api - Filter by service
  - status:error - Filter by status level
  - @http.status_code:[500 TO 599] - Filter by attribute range
  - -status:info - Exclude info logs

  Use the datadog_log_query tool to search logs.

user_prompt: |
  Find all 5xx errors from the API service in production
  in the last 30 minutes. Identify the most common error messages.
```

### 7.3 Monitor Management Agent

```yaml
name: datadog-monitor-manager
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_monitor_list
  - datadog_monitor_mute
  - datadog_monitor_unmute

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a Datadog monitor management agent.

  Manage monitors, handle alerts, and coordinate incident response.

  Capabilities:
  - List monitors and their current states
  - Mute monitors during maintenance
  - Unmute monitors after issues are resolved

  Use monitor tags and scopes to target specific monitors.

user_prompt: |
  List all monitors in the production environment that are currently alerting.
  For any false positives, mute them with an appropriate scope.
```

### 7.4 Event Tracking Agent

```yaml
name: datadog-event-tracker
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_event_post
  - shell  # For deployment scripts

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a deployment tracking agent using Datadog events.

  Post events to Datadog for:
  - Deployments
  - Configuration changes
  - Incidents
  - Maintenance windows

  Use the datadog_event_post tool to create events with:
  - Descriptive titles
  - Detailed text (supports Markdown)
  - Appropriate tags (env, service, version)
  - Correct alert_type (success, error, warning, info)

user_prompt: |
  Deploy API version 1.2.3 to production and post a success event
  to Datadog with appropriate tags.
```

### 7.5 Maintenance Window Agent

```yaml
name: datadog-maintenance-scheduler
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_downtime_create
  - datadog_downtime_cancel
  - datadog_monitor_list

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a maintenance window management agent.

  Schedule and manage downtimes to suppress alerts during:
  - Planned maintenance
  - Deployment windows
  - Testing periods

  Use scopes to target specific hosts, services, or environments.
  Set appropriate start/end times or create recurring downtimes.

user_prompt: |
  Schedule a maintenance window for all staging hosts
  every Saturday from 2 AM to 4 AM UTC for the next 3 months.
```

### 7.6 Comprehensive Observability Agent

```yaml
name: datadog-observability-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - datadog_metric_query
  - datadog_log_query
  - datadog_monitor_list
  - datadog_monitor_mute
  - datadog_event_post
  - datadog_downtime_create

config:
  datadog:
    endpoint: "https://api.datadoghq.com"
    api_key: "${DATADOG_API_KEY}"
    app_key: "${DATADOG_APP_KEY}"

system_prompt: |
  You are a comprehensive observability agent with full Datadog access.

  Capabilities:
  1. Query metrics for performance analysis
  2. Search logs for troubleshooting
  3. Manage monitors and alerts
  4. Post deployment/incident events
  5. Schedule maintenance windows

  Use all available tools to provide complete observability workflows.

user_prompt: |
  Investigate the production API service:
  1. Check error rate metrics for the last hour
  2. Search logs for 5xx errors
  3. List any alerting monitors
  4. Post an investigation event
  5. If needed, mute false-positive monitors
```

## 8. Implementation Checklist

- [ ] Create `DatadogTools` struct with all 8 operations
- [ ] Implement authentication with DD-API-KEY and DD-APPLICATION-KEY headers
- [ ] Add endpoint configuration with multi-region support
- [ ] Implement time parsing utilities (Unix, ISO 8601, relative)
- [ ] Add retry logic for rate limiting
- [ ] Create comprehensive error handling
- [ ] Write unit tests for each tool
- [ ] Add integration tests with mock Datadog API
- [ ] Document in `docs/tools/datadog.md`
- [ ] Add examples to `examples/agents/datadog-*.yaml`
- [ ] Update `ObservabilityTools::all()` to include Datadog tools
- [ ] Export tools in `crates/aof-tools/src/lib.rs`
- [ ] Add to feature flags in `Cargo.toml`

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_unix() {
        let result = parse_time_param("1639065600");
        assert_eq!(result.unwrap(), 1639065600);
    }

    #[test]
    fn test_parse_time_iso8601() {
        let result = parse_time_param("2024-01-15T10:30:00Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_time_relative() {
        let result = parse_relative_time("1h");
        assert_eq!(result, Some(3600));
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_metric_query_integration() {
    // Use wiremock to mock Datadog API
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "ok",
            "series": []
        })))
        .mount(&mock_server)
        .await;

    let tool = DatadogMetricQueryTool::new();
    let input = ToolInput::from_json(json!({
        "endpoint": mock_server.uri(),
        "api_key": "test-key",
        "app_key": "test-app-key",
        "query": "avg:system.cpu.user{*}",
        "from": "1639065600",
        "to": "1639069200"
    }));

    let result = tool.execute(input).await.unwrap();
    assert!(result.success());
}
```

## 10. Performance Considerations

### 10.1 Rate Limiting

Datadog API rate limits:
- **Metrics Query**: 300 requests per hour per organization
- **Log Search**: 300 requests per hour per organization
- **Monitor APIs**: 1000 requests per hour per organization
- **Events**: 500,000 events per hour per organization

Implement exponential backoff for 429 responses.

### 10.2 Caching

For frequently accessed data (monitor lists, metric metadata), implement TTL-based caching:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CachedValue<T> {
    value: T,
    expires_at: Instant,
}

struct DatadogCache {
    monitors: Arc<Mutex<Option<CachedValue<Vec<serde_json::Value>>>>>,
    ttl: Duration,
}

impl DatadogCache {
    fn new(ttl_secs: u64) -> Self {
        Self {
            monitors: Arc::new(Mutex::new(None)),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    async fn get_monitors(&self) -> Option<Vec<serde_json::Value>> {
        let cache = self.monitors.lock().await;
        if let Some(cached) = &*cache {
            if cached.expires_at > Instant::now() {
                return Some(cached.value.clone());
            }
        }
        None
    }

    async fn set_monitors(&self, monitors: Vec<serde_json::Value>) {
        let mut cache = self.monitors.lock().await;
        *cache = Some(CachedValue {
            value: monitors,
            expires_at: Instant::now() + self.ttl,
        });
    }
}
```

### 10.3 Pagination

For operations that may return large datasets (logs, monitor lists), implement cursor-based pagination:

```rust
async fn fetch_all_logs(
    client: &Client,
    endpoint: &str,
    query: &str,
    from: &str,
    to: &str,
    max_pages: u32,
) -> AofResult<Vec<serde_json::Value>> {
    let mut all_logs = Vec::new();
    let mut cursor: Option<String> = None;
    let mut page = 0;

    loop {
        if page >= max_pages {
            break;
        }

        let mut body = json!({
            "filter": {
                "query": query,
                "from": from,
                "to": to
            }
        });

        if let Some(c) = cursor {
            body["page"] = json!({ "cursor": c });
        }

        let response = client
            .post(format!("{}/api/v2/logs/events/search", endpoint))
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;

        if let Some(logs) = data["data"].as_array() {
            all_logs.extend_from_slice(logs);
        }

        // Check for next page
        cursor = data["meta"]["page"]["after"]
            .as_str()
            .map(|s| s.to_string());

        if cursor.is_none() {
            break;
        }

        page += 1;
    }

    Ok(all_logs)
}
```

## 11. Security Considerations

### 11.1 API Key Management

- **Never log API keys or application keys**
- Support environment variables: `DATADOG_API_KEY`, `DATADOG_APP_KEY`
- Use secure secret management (Vault, AWS Secrets Manager, etc.)
- Implement key rotation support

### 11.2 Scope Validation

For mute/downtime operations, validate scopes to prevent accidental global mutes:

```rust
fn validate_scope(scope: &str) -> Result<(), String> {
    // Prevent overly broad scopes
    if scope == "*" || scope.is_empty() {
        return Err("Scope cannot be global ('*' or empty). Specify a specific scope like 'env:staging'".to_string());
    }

    // Validate scope format
    if !scope.contains(':') {
        return Err(format!("Invalid scope format: '{}'. Expected 'key:value' format", scope));
    }

    Ok(())
}
```

### 11.3 Audit Logging

Log all write operations (mute, post event, create downtime) for audit trails:

```rust
use tracing::info;

info!(
    operation = "monitor_mute",
    monitor_id = monitor_id,
    scope = ?scope,
    user = "aof-agent",
    "Muted Datadog monitor"
);
```

## 12. Future Enhancements

### Phase 2 Features (Not in Initial Implementation)

1. **APM Trace Query** - Query distributed traces
2. **Service Dependency Map** - Retrieve service topology
3. **Dashboard Management** - CRUD operations for dashboards
4. **SLO Query** - Service Level Objective metrics
5. **Incident Management** - Create and manage incidents
6. **Synthetic Test Management** - Manage synthetic monitoring
7. **RUM Query** - Real User Monitoring data
8. **Security Signals** - Query security events

These can be added incrementally based on user demand.

---

**Document Version**: 1.0
**Last Updated**: 2024-01-15
**Author**: SPARC Specification Agent
**Status**: Ready for Implementation
