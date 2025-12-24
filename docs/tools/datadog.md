# Datadog Tools

AOF provides native Datadog integration tools for querying metrics, searching logs, managing monitors, posting events, and creating downtimes.

> **Feature Flag Required**: These tools require the `observability` feature flag to be enabled during compilation.

## Prerequisites

- Datadog account (any tier)
- Datadog API key (organization-level)
- Datadog Application key (user-level permissions)
- Correct endpoint for your Datadog region

## Authentication

Datadog tools require **dual authentication**:

1. **API Key** (`DD-API-KEY` header): Organization-level API key
2. **Application Key** (`DD-APPLICATION-KEY` header): User-level key with specific permissions

### Creating API Keys

1. **API Key**: Go to **Organization Settings → API Keys** in Datadog
2. **Application Key**: Go to **Organization Settings → Application Keys**
3. Ensure the application key has permissions for the operations you need

### Environment Variables

You can use environment variables to avoid hardcoding credentials:

```yaml
env:
  DATADOG_API_KEY: "${DD_API_KEY}"
  DATADOG_APP_KEY: "${DD_APP_KEY}"
```

## Supported Regions

Datadog operates in multiple regions. Use the correct endpoint for your account:

| Region | Endpoint | Description |
|--------|----------|-------------|
| **US1** | `https://api.datadoghq.com` | Default US region |
| **US3** | `https://api.us3.datadoghq.com` | US3 region |
| **US5** | `https://api.us5.datadoghq.com` | US5 region |
| **EU1** | `https://api.datadoghq.eu` | European Union |
| **AP1** | `https://api.ap1.datadoghq.com` | Asia Pacific |
| **US1-FED** | `https://api.ddog-gov.com` | US Government (FedRAMP) |

## Available Tools

| Tool | Description | Use Cases |
|------|-------------|-----------|
| `datadog_metric_query` | Query metrics using Datadog query language | Performance analysis, trend detection |
| `datadog_log_query` | Search logs using Datadog log search syntax | Debugging, error investigation |
| `datadog_monitor_list` | List monitors and their states | Monitor status checks, inventory |
| `datadog_monitor_mute` | Mute a monitor or monitor group | Maintenance windows, noise reduction |
| `datadog_event_post` | Post custom events to event stream | Deployment tracking, incident logging |
| `datadog_downtime_create` | Create scheduled downtime | Planned maintenance, alert suppression |

---

## datadog_metric_query

Query Datadog metrics using Datadog query language. Returns time-series data for analysis and monitoring.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key |
| `app_key` | string | Yes | Datadog Application key |
| `query` | string | Yes | Metric query in Datadog query language |
| `from` | string | Yes | Start time (Unix timestamp, ISO 8601, or relative like `-1h`) |
| `to` | string | Yes | End time (Unix timestamp, ISO 8601, or `now`) |
| `endpoint` | string | No | Datadog API endpoint (default: `https://api.datadoghq.com`) |

**Time Format Options:**

- **Unix timestamp**: `1640000000`
- **ISO 8601**: `2024-12-23T10:00:00Z`
- **Relative time**: `-1h`, `-30m`, `-7d`
- **Current time**: `now`

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: datadog-metrics-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Datadog metrics analyst. Query metrics and provide performance insights.
  tools:
    - datadog_metric_query
  env:
    DATADOG_API_KEY: "${DD_API_KEY}"
    DATADOG_APP_KEY: "${DD_APP_KEY}"
```

**Example Tool Call:**

```json
{
  "name": "datadog_metric_query",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "app_key": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
    "query": "avg:system.cpu.user{host:web-*}",
    "from": "-1h",
    "to": "now"
  }
}
```

**Query Language Examples:**

```python
# Average CPU across all hosts
"avg:system.cpu.user{*}"

# Max memory by host
"max:system.mem.used{*} by {host}"

# Request rate with aggregation
"sum:trace.http.request.hits{service:api}.as_rate()"

# Custom metrics with tags
"avg:my.custom.metric{env:prod,region:us-east-1}"
```

**Response:**

```json
{
  "series": [
    {
      "metric": "system.cpu.user",
      "pointlist": [
        [1640000000000, 45.2],
        [1640000060000, 47.8]
      ],
      "scope": "host:web-01",
      "aggr": "avg"
    }
  ]
}
```

**Common Use Cases:**

- Monitor system resource usage
- Analyze application performance metrics
- Compare metrics across environments
- Detect performance anomalies

---

## datadog_log_query

Search Datadog logs using Datadog log search syntax. Returns log entries for debugging and analysis.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key |
| `app_key` | string | Yes | Datadog Application key |
| `query` | string | Yes | Log search query (e.g., `service:api status:error`) |
| `from` | string | Yes | Start time (ISO 8601 format) |
| `to` | string | Yes | End time (ISO 8601 format) |
| `limit` | integer | No | Maximum logs to return (max: 1000, default: 50) |
| `sort` | string | No | Sort order: `timestamp` or `-timestamp` (default: `-timestamp`) |
| `endpoint` | string | No | Datadog API endpoint |

**Example:**

```json
{
  "name": "datadog_log_query",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "app_key": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
    "query": "service:api status:error",
    "from": "2024-12-23T09:00:00Z",
    "to": "2024-12-23T10:00:00Z",
    "limit": 100,
    "sort": "-timestamp"
  }
}
```

**Query Syntax Examples:**

```python
# Errors in specific service
"service:api status:error"

# Logs from specific host
"host:web-01"

# Multiple conditions
"service:api env:prod status:(error OR warn)"

# Keyword search
"service:api \"database connection failed\""

# Attribute filtering
"@http.status_code:500 @user.id:12345"
```

**Response:**

```json
{
  "data": [
    {
      "id": "abc123",
      "attributes": {
        "timestamp": "2024-12-23T09:30:00Z",
        "message": "Database connection failed",
        "service": "api",
        "status": "error",
        "host": "web-01",
        "tags": ["env:prod", "version:1.2.3"]
      }
    }
  ],
  "meta": {
    "page": {
      "after": "next_page_cursor"
    }
  }
}
```

**Common Use Cases:**

- Debug application errors
- Search for specific error messages
- Analyze log patterns
- Investigate incidents

---

## datadog_monitor_list

List Datadog monitors and their current states. Use filters to narrow results by tags, name, or monitor tags.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key |
| `app_key` | string | Yes | Datadog Application key |
| `tags` | string | No | Filter by tags (comma-separated: `env:prod,service:api`) |
| `name` | string | No | Filter by monitor name (substring match) |
| `monitor_tags` | string | No | Filter by monitor-specific tags |
| `endpoint` | string | No | Datadog API endpoint |

**Example:**

```json
{
  "name": "datadog_monitor_list",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "app_key": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
    "tags": "env:prod,service:api"
  }
}
```

**Response:**

```json
[
  {
    "id": 12345,
    "name": "High API Error Rate",
    "type": "metric alert",
    "query": "avg(last_5m):sum:trace.http.request.errors{service:api} > 100",
    "message": "API error rate is high. @pagerduty",
    "tags": ["service:api", "env:prod"],
    "options": {
      "notify_audit": false,
      "locked": false,
      "timeout_h": 0,
      "new_host_delay": 300,
      "require_full_window": false,
      "notify_no_data": false,
      "renotify_interval": 0
    },
    "overall_state": "Alert",
    "created": "2024-01-15T10:30:00Z",
    "modified": "2024-12-20T15:45:00Z"
  }
]
```

**Common Use Cases:**

- Check monitor status before deployments
- Generate monitor inventory
- Find monitors by tag or name
- Monitor health checks

---

## datadog_monitor_mute

Mute a Datadog monitor or monitor group during maintenance or to silence false positives.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key |
| `app_key` | string | Yes | Datadog Application key |
| `monitor_id` | integer | Yes | Monitor ID to mute |
| `scope` | string | No | Scope to mute (e.g., `host:web-01`, `env:staging`) |
| `end` | integer | No | End time for mute (Unix timestamp). Omit for indefinite |
| `endpoint` | string | No | Datadog API endpoint |

**Example:**

```json
{
  "name": "datadog_monitor_mute",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "app_key": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
    "monitor_id": 12345,
    "scope": "env:staging",
    "end": 1640004000
  }
}
```

**Scope Examples:**

```python
# Mute for specific host
"host:web-01"

# Mute for environment
"env:staging"

# Mute for region
"region:us-east-1"

# Multiple scopes
"env:staging,service:api"

# Wildcard patterns
"host:web-*"
```

**Response:**

```json
{
  "id": 12345,
  "active": true,
  "muted": true,
  "scope": ["env:staging"],
  "end": 1640004000
}
```

**Common Use Cases:**

- Silence monitors during deployments
- Mute false positives temporarily
- Suppress alerts for specific hosts
- Maintenance window management

---

## datadog_event_post

Post custom events to the Datadog event stream. Track deployments, incidents, and other milestones.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key (Application key not required) |
| `title` | string | Yes | Event title (max 500 characters) |
| `text` | string | Yes | Event description (supports Markdown, max 4000 characters) |
| `alert_type` | string | No | Event severity: `error`, `warning`, `info`, `success` (default: `info`) |
| `tags` | array[string] | No | Event tags (e.g., `["env:prod", "version:1.2.3"]`) |
| `endpoint` | string | No | Datadog API endpoint |

**Example:**

```json
{
  "name": "datadog_event_post",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "title": "Deployment: v1.2.3 to Production",
    "text": "Successfully deployed version 1.2.3 to production environment.\n\n**Changes:**\n- Fixed database connection pool leak\n- Added new API endpoint\n- Updated dependencies",
    "alert_type": "success",
    "tags": ["deployment", "env:prod", "version:1.2.3", "service:api"]
  }
}
```

**Alert Types:**

- `error`: Red, for failures and critical issues
- `warning`: Yellow, for warnings and degraded states
- `info`: Blue, for informational events (default)
- `success`: Green, for successful operations

**Response:**

```json
{
  "event": {
    "id": 9876543210,
    "title": "Deployment: v1.2.3 to Production",
    "text": "Successfully deployed...",
    "date_happened": 1640000000,
    "alert_type": "success",
    "tags": ["deployment", "env:prod", "version:1.2.3"]
  },
  "status": "ok"
}
```

**Common Use Cases:**

- Track deployment events
- Log incident milestones
- Document configuration changes
- Mark release timelines
- Correlate events with metrics

---

## datadog_downtime_create

Create scheduled downtime for maintenance windows. Suppress alerts during planned work.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | Datadog API key |
| `app_key` | string | Yes | Datadog Application key |
| `scope` | array[string] | Yes | Scopes to apply downtime (e.g., `["host:web-*", "env:prod"]`) |
| `start` | integer | No | Start time (Unix timestamp, default: now) |
| `end` | integer | No | End time (Unix timestamp). Required if no recurrence |
| `message` | string | No | Message to include with notifications |
| `endpoint` | string | No | Datadog API endpoint |

**Example:**

```json
{
  "name": "datadog_downtime_create",
  "input": {
    "api_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "app_key": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
    "scope": ["env:prod", "service:database"],
    "start": 1640001600,
    "end": 1640005200,
    "message": "Scheduled database maintenance: upgrading to PostgreSQL 15"
  }
}
```

**Scope Examples:**

```python
# All production hosts
["env:prod"]

# Specific service
["service:api"]

# Multiple scopes
["env:prod", "service:api", "region:us-east-1"]

# Host wildcards
["host:web-*"]

# Complex scoping
["env:staging", "service:api", "!host:api-critical"]
```

**Response:**

```json
{
  "id": 123456,
  "scope": ["env:prod", "service:database"],
  "start": 1640001600,
  "end": 1640005200,
  "message": "Scheduled database maintenance...",
  "timezone": "UTC",
  "active": true,
  "disabled": false
}
```

**Common Use Cases:**

- Schedule maintenance windows
- Suppress alerts during upgrades
- Planned downtime for specific services
- Temporary alert muting for known issues

---

## Multi-Region Configuration

For non-US1 regions, specify the appropriate endpoint:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: datadog-eu-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Query Datadog EU region for metrics.
  tools:
    - datadog_metric_query
  env:
    DATADOG_ENDPOINT: "https://api.datadoghq.eu"
    DATADOG_API_KEY: "${DD_API_KEY}"
    DATADOG_APP_KEY: "${DD_APP_KEY}"
```

---

## Environment Variables

For secure credential management:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: datadog-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Monitor production systems using Datadog.
  tools:
    - datadog_metric_query
    - datadog_log_query
    - datadog_event_post
  env:
    # Injected from secrets
    DATADOG_API_KEY: "${DD_API_KEY}"
    DATADOG_APP_KEY: "${DD_APP_KEY}"
    DATADOG_ENDPOINT: "${DD_ENDPOINT}"
```

---

## Error Handling

Tools return structured error responses for common issues:

| Status | Error Type | Description |
|--------|------------|-------------|
| 400 | Bad Request | Invalid parameters or malformed query |
| 401 | Unauthorized | Invalid API key or App key |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist (monitor, event, etc.) |
| 429 | Rate Limited | Too many requests, retry with backoff |
| 500+ | Server Error | Datadog internal error |

**Example Error Response:**

```json
{
  "error": "Authentication failed: Invalid API key"
}
```

---

## Best Practices

1. **Secure Credentials**: Never hardcode API keys. Use environment variables or secrets
2. **Scope Permissions**: Use application keys with minimal required permissions
3. **Rate Limiting**: Respect Datadog API rate limits. Implement exponential backoff for retries
4. **Time Ranges**: Use relative times for dynamic queries (`-1h`, `-30m`)
5. **Tag Consistently**: Use consistent tagging for better filtering and organization
6. **Monitor Costs**: Be aware of metric and log query costs, especially for high-frequency queries
7. **Error Handling**: Always check for authentication and permission errors before retrying
8. **Regional Endpoints**: Use the correct endpoint for your region to reduce latency

---

## Complete Example: Production Monitoring Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: production-datadog-monitor
  namespace: monitoring
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a production monitoring assistant using Datadog. Your responsibilities:

    1. Monitor key metrics (CPU, memory, request rates, error rates)
    2. Search logs for errors and anomalies
    3. Check monitor status before and after deployments
    4. Create events for significant changes
    5. Mute monitors during planned maintenance
    6. Provide analysis and recommendations

    When you detect issues:
    - Query relevant metrics to understand scope
    - Search logs for error details
    - Check if monitors are alerting
    - Create events to document findings

  tools:
    - datadog_metric_query
    - datadog_log_query
    - datadog_monitor_list
    - datadog_monitor_mute
    - datadog_event_post
    - datadog_downtime_create

  env:
    DATADOG_API_KEY: "${DD_API_KEY}"
    DATADOG_APP_KEY: "${DD_APP_KEY}"
    DATADOG_ENDPOINT: "https://api.datadoghq.com"

  triggers:
    - type: cron
      schedule: "*/10 * * * *"  # Every 10 minutes
      message: "Check production health: metrics, logs, and monitors"

    - type: webhook
      path: /deployment
      message: "Deployment started. Monitor metrics and create event."
```

---

## Related Documentation

- [Builtin Tools Reference](./builtin-tools.md)
- [Grafana Tools](./grafana.md)
- [Custom Tools](./custom-tools.md)
- [MCP Integration](./mcp-integration.md)
