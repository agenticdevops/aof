# Grafana Tools

AOF provides native Grafana integration tools for querying metrics, managing dashboards, working with alerts, and creating annotations.

> **Feature Flag Required**: These tools require the `observability` feature flag to be enabled during compilation.

## Prerequisites

- Grafana instance (cloud or self-hosted)
- Grafana API key or Service Account token
- Bearer token authentication enabled
- Appropriate permissions for desired operations

## Authentication

All Grafana tools use Bearer token authentication. You can create an API key or Service Account token from your Grafana instance:

1. Go to **Configuration → API Keys** (or **Service Accounts**)
2. Create a new key with appropriate permissions
3. Copy the token for use in agent configurations

## Available Tools

| Tool | Description | Use Cases |
|------|-------------|-----------|
| `grafana_query` | Query data sources through Grafana | Metrics analysis, trend detection |
| `grafana_dashboard_get` | Retrieve dashboard by UID | Dashboard inspection, backup |
| `grafana_dashboard_list` | Search and list dashboards | Discovery, inventory |
| `grafana_alert_list` | List alert rules | Alert monitoring, status checks |
| `grafana_alert_silence` | Create alert silences | Maintenance windows, incident management |
| `grafana_annotation_create` | Create annotations | Deployment tracking, event marking |

---

## grafana_query

Query data sources through Grafana's unified query API. Supports Prometheus, Loki, and other data sources configured in your Grafana instance.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL (e.g., `https://grafana.example.com`) |
| `datasource_uid` | string | Yes | Data source UID from Grafana |
| `query` | string | Yes | Query in data source's native language (PromQL, LogQL, etc.) |
| `api_key` | string | Yes | Grafana API key or service account token |
| `from` | string | No | Start time (RFC3339 or Unix timestamp in milliseconds) |
| `to` | string | No | End time (RFC3339 or Unix timestamp in milliseconds) |
| `max_data_points` | integer | No | Maximum number of data points to return (default: 1000) |
| `interval_ms` | integer | No | Interval in milliseconds between data points |
| `org_id` | integer | No | Organization ID for multi-org setups |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: grafana-metrics-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Grafana metrics analyst. Query Prometheus data and provide insights.
  tools:
    - grafana_query
  env:
    GRAFANA_ENDPOINT: "https://grafana.example.com"
    GRAFANA_API_KEY: "${GRAFANA_TOKEN}"
    GRAFANA_DATASOURCE_UID: "prometheus-uid"
```

**Example Tool Call:**

```json
{
  "name": "grafana_query",
  "input": {
    "endpoint": "https://grafana.example.com",
    "datasource_uid": "prometheus-main",
    "query": "avg(rate(http_requests_total[5m]))",
    "from": "now-1h",
    "to": "now",
    "api_key": "glsa_xxxxxxxxxxxx"
  }
}
```

**Response:**

```json
{
  "results": {
    "A": {
      "frames": [
        {
          "schema": {...},
          "data": {...}
        }
      ]
    }
  },
  "datasource_uid": "prometheus-main",
  "query": "avg(rate(http_requests_total[5m]))"
}
```

**Common Use Cases:**

- Query Prometheus metrics for troubleshooting
- Analyze time-series data patterns
- Fetch Loki logs for specific time ranges
- Monitor application performance metrics

---

## grafana_dashboard_get

Retrieve a complete dashboard definition by UID. Returns full dashboard JSON including panels, variables, and settings.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL |
| `dashboard_uid` | string | Yes | Dashboard UID (found in dashboard URL) |
| `api_key` | string | Yes | Grafana API key |
| `org_id` | integer | No | Organization ID |

**Example:**

```json
{
  "name": "grafana_dashboard_get",
  "input": {
    "endpoint": "https://grafana.example.com",
    "dashboard_uid": "abc123def",
    "api_key": "glsa_xxxxxxxxxxxx"
  }
}
```

**Response:**

```json
{
  "dashboard": {
    "id": 123,
    "uid": "abc123def",
    "title": "Production Metrics",
    "panels": [...],
    "templating": {...}
  },
  "meta": {
    "isStarred": false,
    "url": "/d/abc123def/production-metrics"
  }
}
```

**Common Use Cases:**

- Backup dashboard configurations
- Inspect panel queries and settings
- Clone dashboards to other environments
- Audit dashboard definitions

---

## grafana_dashboard_list

Search and list dashboards with optional filters. Useful for discovery and inventory management.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL |
| `api_key` | string | Yes | Grafana API key |
| `query` | string | No | Search query string (matches title) |
| `tags` | array[string] | No | Filter by tags |
| `folder_ids` | array[integer] | No | Filter by folder IDs |
| `limit` | integer | No | Maximum results (default: 100) |
| `org_id` | integer | No | Organization ID |

**Example:**

```json
{
  "name": "grafana_dashboard_list",
  "input": {
    "endpoint": "https://grafana.example.com",
    "api_key": "glsa_xxxxxxxxxxxx",
    "tags": ["production", "kubernetes"],
    "limit": 50
  }
}
```

**Response:**

```json
{
  "dashboards": [
    {
      "id": 123,
      "uid": "abc123",
      "title": "Kubernetes Cluster",
      "uri": "db/kubernetes-cluster",
      "url": "/d/abc123/kubernetes-cluster",
      "tags": ["production", "kubernetes"]
    }
  ],
  "count": 1
}
```

**Common Use Cases:**

- Find dashboards by tag or name
- Generate dashboard inventory
- Locate specific monitoring views
- Organize dashboard discovery

---

## grafana_alert_list

List alert rules and their current states. Supports filtering by dashboard, state, and folder.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL |
| `api_key` | string | Yes | Grafana API key |
| `dashboard_uid` | string | No | Filter by dashboard UID |
| `panel_id` | integer | No | Filter by panel ID |
| `state` | string | No | Filter by state: `alerting`, `ok`, `no_data`, `paused` |
| `folder_id` | integer | No | Filter by folder ID |
| `org_id` | integer | No | Organization ID |

**Example:**

```json
{
  "name": "grafana_alert_list",
  "input": {
    "endpoint": "https://grafana.example.com",
    "api_key": "glsa_xxxxxxxxxxxx",
    "state": "alerting"
  }
}
```

**Response:**

```json
{
  "alerts": [
    {
      "id": 1,
      "name": "High CPU Usage",
      "state": "alerting",
      "dashboardUid": "abc123",
      "panelId": 2,
      "newStateDate": "2024-12-23T10:30:00Z"
    }
  ],
  "count": 1
}
```

**Common Use Cases:**

- Monitor active alerts
- Check alert status before deployments
- Filter alerts by environment or service
- Alert health checks

---

## grafana_alert_silence

Create an alert silence to suppress notifications during maintenance windows or known issues.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL |
| `api_key` | string | Yes | Grafana API key |
| `matchers` | array[object] | Yes | Label matchers for silence (see below) |
| `ends_at` | string | Yes | End time (RFC3339 format) |
| `comment` | string | Yes | Reason for silence |
| `starts_at` | string | No | Start time (default: now) |
| `created_by` | string | No | Username |
| `org_id` | integer | No | Organization ID |

**Matcher Object:**

```json
{
  "name": "alertname",        // Label name
  "value": "HighCPU",         // Label value
  "isRegex": false            // Whether value is regex
}
```

**Example:**

```json
{
  "name": "grafana_alert_silence",
  "input": {
    "endpoint": "https://grafana.example.com",
    "api_key": "glsa_xxxxxxxxxxxx",
    "matchers": [
      {
        "name": "alertname",
        "value": "HighCPU",
        "isRegex": false
      },
      {
        "name": "environment",
        "value": "production",
        "isRegex": false
      }
    ],
    "ends_at": "2024-12-23T12:00:00Z",
    "comment": "Planned maintenance window for database upgrade",
    "created_by": "ops-team"
  }
}
```

**Response:**

```json
{
  "silence_id": "abc123-def456",
  "starts_at": "2024-12-23T10:00:00Z",
  "ends_at": "2024-12-23T12:00:00Z"
}
```

**Common Use Cases:**

- Silence alerts during deployments
- Suppress known issues temporarily
- Maintenance window alert management
- Reduce alert noise during incidents

---

## grafana_annotation_create

Create annotations on dashboards to mark significant events like deployments, incidents, or configuration changes.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | Grafana server URL |
| `api_key` | string | Yes | Grafana API key |
| `text` | string | Yes | Annotation text/description |
| `time` | integer | No | Event time in Unix milliseconds (default: now) |
| `time_end` | integer | No | End time for region annotations |
| `tags` | array[string] | No | Tags for annotation |
| `dashboard_uid` | string | No | Associate with specific dashboard |
| `panel_id` | integer | No | Associate with specific panel |
| `org_id` | integer | No | Organization ID |

**Example:**

```json
{
  "name": "grafana_annotation_create",
  "input": {
    "endpoint": "https://grafana.example.com",
    "api_key": "glsa_xxxxxxxxxxxx",
    "text": "Deployed v1.2.3 to production",
    "tags": ["deployment", "production", "v1.2.3"],
    "dashboard_uid": "prod-overview"
  }
}
```

**Response:**

```json
{
  "annotation_id": 456,
  "message": "Annotation created"
}
```

**Common Use Cases:**

- Mark deployment events on metrics
- Annotate incident start/end times
- Document configuration changes
- Track release timelines

---

## Multi-Organization Support

For Grafana instances with multiple organizations, include the `org_id` parameter:

```yaml
spec:
  tools:
    - grafana_query
  env:
    GRAFANA_ORG_ID: "2"
```

The organization ID will be sent as the `X-Grafana-Org-Id` header.

---

## Environment Variables

For secure credential management, use environment variables:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: grafana-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Query Grafana for metrics and create annotations.
  tools:
    - grafana_query
    - grafana_annotation_create
  env:
    # Injected from secrets or local environment
    GRAFANA_ENDPOINT: "${GRAFANA_URL}"
    GRAFANA_API_KEY: "${GRAFANA_TOKEN}"
    DATASOURCE_UID: "prometheus-prod"
```

---

## Error Handling

Tools return structured error responses for common issues:

| Status | Error | Meaning |
|--------|-------|---------|
| 401 | Authentication failed | Invalid API key or expired token |
| 404 | Resource not found | Dashboard UID or datasource doesn't exist |
| 429 | Rate limited | Too many requests, retry later |
| 500+ | Server error | Grafana internal error |

**Example Error Response:**

```json
{
  "error": "Authentication failed. Check API key and permissions."
}
```

---

## Best Practices

1. **Use Service Account Tokens**: Prefer service accounts over API keys for better security and audit trails
2. **Scope Permissions**: Grant minimal permissions needed (viewer for queries, editor for annotations)
3. **Cache Datasource UIDs**: Look up UIDs once and store in environment variables
4. **Time Ranges**: Use relative times (`now-1h`) for dynamic queries
5. **Rate Limiting**: Respect Grafana API rate limits, especially for automated queries
6. **Error Handling**: Check for authentication and resource errors before retrying

---

## Complete Example: Production Monitoring Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: production-monitor
  namespace: monitoring
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You monitor production metrics in Grafana. When you detect anomalies:
    1. Query relevant metrics using grafana_query
    2. Check alert status with grafana_alert_list
    3. Create annotations for significant events
    4. Provide analysis and recommendations

  tools:
    - grafana_query
    - grafana_alert_list
    - grafana_annotation_create

  env:
    GRAFANA_ENDPOINT: "${GRAFANA_URL}"
    GRAFANA_API_KEY: "${GRAFANA_TOKEN}"
    PROMETHEUS_UID: "prometheus-prod"

  triggers:
    - type: cron
      schedule: "*/5 * * * *"  # Every 5 minutes
      message: "Check production metrics and alert status"
```

---

## Related Documentation

- [Builtin Tools Reference](./builtin-tools.md)
- [Datadog Tools](./datadog.md)
- [Custom Tools](./custom-tools.md)
- [MCP Integration](./mcp-integration.md)
