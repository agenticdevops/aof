# Splunk Tools

AOF provides native Splunk integration tools for executing SPL queries, managing alerts, running saved searches, and sending events via HTTP Event Collector (HEC).

> **Feature Flag Required**: These tools require the `siem` feature flag to be enabled during compilation.

## Prerequisites

- Splunk deployment (Cloud or Enterprise)
- Splunk authentication token or credentials
- Network access to Splunk REST API (port 8089) and HEC (port 8088)

## Authentication

Splunk supports multiple authentication methods:

### Bearer Token (Recommended)

```yaml
env:
  SPLUNK_TOKEN: "${SPLUNK_AUTH_TOKEN}"
```

### Splunk Token

```
Authorization: Splunk <token>
```

### Basic Auth

```
Authorization: Basic <base64(username:password)>
```

### HEC Token (for event ingestion)

```yaml
env:
  SPLUNK_HEC_TOKEN: "${SPLUNK_HEC_TOKEN}"
```

## Network Ports

| Port | Service | Description |
|------|---------|-------------|
| **8089** | REST API | Management and search operations |
| **8088** | HEC | HTTP Event Collector for event ingestion |

## Available Tools

| Tool | Description | Use Cases |
|------|-------------|-----------|
| `splunk_search` | Execute SPL queries | Log analysis, security investigation |
| `splunk_alerts_list` | List fired alerts | Incident response, alert monitoring |
| `splunk_saved_searches` | List saved searches | Search inventory, management |
| `splunk_saved_search_run` | Run a saved search | On-demand analysis, scheduled execution |
| `splunk_hec_send` | Send events via HEC | Event ingestion, audit logging |
| `splunk_indexes_list` | List available indexes | Data source discovery |

---

## splunk_search

Execute SPL (Search Processing Language) queries against Splunk data. Searches are asynchronous - the tool handles job creation, polling, and result retrieval automatically.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base_url` | string | Yes | Splunk REST API URL (e.g., `https://splunk:8089`) |
| `token` | string | Yes | Splunk authentication token |
| `query` | string | Yes | SPL search query |
| `earliest_time` | string | No | Start time (default: `-1h`) |
| `latest_time` | string | No | End time (default: `now`) |
| `max_count` | integer | No | Maximum results (default: 1000) |

**Time Format Options:**

- **Relative time**: `-1h`, `-1d@d`, `-30m`
- **Absolute time**: `2025-12-25T00:00:00`
- **Snap-to**: `@d` (midnight), `@h` (hour)
- **Current time**: `now`

**Example SPL Queries:**

```spl
# Error logs from web servers
index=web sourcetype=access_combined status>=500 | stats count by host

# Security events with failed actions
index=security action=failure | timechart count by user

# Application metrics
index=metrics source="app_metrics" | stats avg(response_time) by endpoint

# Transactions spanning multiple events
index=web | transaction startswith="start" endswith="end"
```

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: splunk-analyst-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Splunk log analysis agent.

    Use SPL queries to:
    - Search for errors and anomalies
    - Analyze access patterns
    - Investigate security events
    - Generate statistics and trends

    Common SPL patterns:
    - `stats count by field` - Aggregate counts
    - `timechart span=1h count` - Time-based charts
    - `rex field=_raw "pattern"` - Extract fields
    - `transaction startswith="..." endswith="..."` - Group related events

  tools:
    - splunk_search
    - splunk_indexes_list

  env:
    SPLUNK_BASE_URL: "${SPLUNK_URL}"
    SPLUNK_TOKEN: "${SPLUNK_TOKEN}"
```

---

## splunk_alerts_list

List fired/triggered alerts from Splunk.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base_url` | string | Yes | Splunk REST API URL |
| `token` | string | Yes | Splunk authentication token |
| `count` | integer | No | Number of alerts to retrieve (default: 50) |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: splunk-security-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Splunk security analysis agent.

    Capabilities:
    1. List fired security alerts
    2. Run security-related searches
    3. Analyze attack patterns

    Focus on:
    - Failed authentication attempts
    - Unusual access patterns
    - Privilege escalation
    - Data exfiltration indicators

  tools:
    - splunk_alerts_list
    - splunk_search
    - splunk_saved_search_run
```

---

## splunk_saved_searches

List configured saved searches in Splunk.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base_url` | string | Yes | Splunk REST API URL |
| `token` | string | Yes | Splunk authentication token |
| `search` | string | No | Filter by name pattern |
| `count` | integer | No | Number of results (default: 50) |

---

## splunk_saved_search_run

Execute a pre-configured saved search.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base_url` | string | Yes | Splunk REST API URL |
| `token` | string | Yes | Splunk authentication token |
| `name` | string | Yes | Saved search name |
| `trigger_actions` | boolean | No | Trigger alert actions (default: false) |

---

## splunk_hec_send

Send events to Splunk via HTTP Event Collector for ingestion.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hec_url` | string | Yes | HEC endpoint URL (e.g., `https://splunk:8088`) |
| `hec_token` | string | Yes | HEC token (GUID format) |
| `event` | object | Yes | Event data to send |
| `source` | string | No | Event source (default: `aof`) |
| `sourcetype` | string | No | Event sourcetype (default: `aof:event`) |
| `index` | string | No | Target index |
| `host` | string | No | Host value for the event |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: splunk-logger-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Splunk event logging agent.

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

  tools:
    - splunk_hec_send

  env:
    SPLUNK_HEC_URL: "${SPLUNK_HEC_URL}"
    SPLUNK_HEC_TOKEN: "${SPLUNK_HEC_TOKEN}"
```

---

## splunk_indexes_list

List available Splunk indexes to discover data sources.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base_url` | string | Yes | Splunk REST API URL |
| `token` | string | Yes | Splunk authentication token |

---

## Pre-built Agents

For production-ready agents using these tools, see the Agent Library:

- **[splunk-analyst](../agent-library/observability.md#splunk-analyst)** - Comprehensive Splunk log analysis and SIEM agent

```bash
# Run the pre-built agent
aofctl run agent library://observability/splunk-analyst \
  --prompt "Search for error logs in the last hour"
```

## Rate Limits

- **REST API**: Deployment-specific (no hard default limits)
- **Search Concurrency**: Typically 5-10 concurrent searches
- **Result Limit**: 50,000 rows per search
- **HEC**: High throughput, batching recommended

## Best Practices

1. **Always use time bounds**: Include `earliest_time` and `latest_time`
2. **Use export for large results**: For streaming large datasets
3. **Limit fields**: Use `| fields` to reduce data transfer
4. **Paginate results**: Use `offset` and `count` parameters
5. **Batch HEC events**: Send multiple events in single requests

## Security Considerations

1. **Token Management**: Store tokens in environment variables
2. **Service Accounts**: Use accounts with minimal permissions
3. **Token Rotation**: Rotate tokens periodically
4. **Network Security**: Always use HTTPS, validate SSL certificates

## See Also

- [Observability Agents](../agent-library/observability.md) - Pre-built agents using Splunk
- [Datadog Tools](./datadog.md) - Alternative observability platform
- [New Relic Tools](./newrelic.md) - APM and observability
- [Built-in Tools Reference](./builtin-tools.md) - Complete list of built-in tools
