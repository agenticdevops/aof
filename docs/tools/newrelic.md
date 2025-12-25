# New Relic Tools

AOF provides native New Relic integration tools for querying metrics via NRQL, listing alerts and incidents, searching entities, and managing incident workflows.

> **Feature Flag Required**: These tools require the `observability` feature flag to be enabled during compilation.

## Prerequisites

- New Relic account (any tier)
- New Relic User API Key (NRAK-...)
- Account ID for account-scoped operations

**Important**: REST API keys are deprecated as of March 1, 2025. Only User API Keys are supported.

## Authentication

New Relic uses User API Keys for authentication via the `API-Key` header.

### Creating API Keys

1. Go to **Account Settings → API Keys** in New Relic
2. Click **Create a key**
3. Select **User** as the key type
4. Name your key and assign appropriate permissions

### Environment Variables

You can use environment variables to avoid hardcoding credentials:

```yaml
env:
  NEWRELIC_API_KEY: "${NR_API_KEY}"
  NEWRELIC_ACCOUNT_ID: "${NR_ACCOUNT_ID}"
```

## Supported Regions

New Relic operates in two main regions:

| Region | Endpoint | Description |
|--------|----------|-------------|
| **US** | `https://api.newrelic.com/graphql` | Default US region |
| **EU** | `https://api.eu.newrelic.com/graphql` | European Union |

## Available Tools

| Tool | Description | Use Cases |
|------|-------------|-----------|
| `newrelic_nrql_query` | Execute NRQL queries | Metrics analysis, log queries, custom analytics |
| `newrelic_alerts_list` | List alert policies | Alert inventory, policy management |
| `newrelic_incidents_list` | List active incidents | Incident response, status monitoring |
| `newrelic_entity_search` | Search monitored entities | Service discovery, impact analysis |
| `newrelic_metrics_query` | Query metric timeslices | Performance deep-dives |
| `newrelic_incident_ack` | Acknowledge incidents | Incident response workflows |

---

## newrelic_nrql_query

Execute NRQL (New Relic Query Language) queries against New Relic data. Query metrics, logs, traces, and events.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key (NRAK-...) |
| `account_id` | string | Yes | New Relic Account ID |
| `query` | string | Yes | NRQL query string |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |

**Example NRQL Queries:**

```sql
-- CPU usage across hosts
SELECT average(cpuPercent) FROM SystemSample FACET hostname SINCE 1 hour ago

-- Error rate by service
SELECT percentage(count(*), WHERE error IS true) FROM Transaction
WHERE appName = 'my-app' SINCE 30 minutes ago TIMESERIES

-- Log count by severity
SELECT count(*) FROM Log FACET level SINCE 1 day ago
```

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: newrelic-query-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a New Relic observability agent. Use NRQL to query:
    - Application metrics (Transaction, Span)
    - Infrastructure metrics (SystemSample, ProcessSample)
    - Logs (Log)
    - Custom events
  tools:
    - newrelic_nrql_query
  env:
    NEWRELIC_API_KEY: "${NR_API_KEY}"
    NEWRELIC_ACCOUNT_ID: "${NR_ACCOUNT_ID}"
```

---

## newrelic_alerts_list

List New Relic alert policies and their configurations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key |
| `account_id` | string | Yes | New Relic Account ID |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |
| `limit` | integer | No | Max policies to return (default: 50) |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: newrelic-alerts-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a New Relic alerts analyst. List and analyze alert policies.
  tools:
    - newrelic_alerts_list
    - newrelic_nrql_query
```

---

## newrelic_incidents_list

List active and recent incidents from New Relic.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key |
| `account_id` | string | Yes | New Relic Account ID |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |
| `states` | array | No | Filter by state: `ACTIVATED`, `CREATED`, `CLOSED` |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: newrelic-incident-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a New Relic incident response agent.

    Capabilities:
    1. List active incidents
    2. Query related metrics and logs
    3. Acknowledge incidents

    Provide recommendations for resolution.
  tools:
    - newrelic_incidents_list
    - newrelic_incident_ack
    - newrelic_nrql_query
    - newrelic_entity_search
```

---

## newrelic_entity_search

Search for monitored entities (applications, hosts, services) across the New Relic platform.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key |
| `query` | string | Yes | Entity search query |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |
| `limit` | integer | No | Max entities to return (default: 50) |

**Example Entity Queries:**

```
# All APM applications
type = 'APPLICATION'

# Hosts in production
type = 'HOST' AND tags.environment = 'production'

# Services by name
name LIKE 'payment%'

# Alerting entities
alertSeverity = 'CRITICAL'
```

---

## newrelic_metrics_query

Query detailed metric timeslice data for specific entities.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key |
| `account_id` | string | Yes | New Relic Account ID |
| `entity_guid` | string | Yes | Entity GUID to query |
| `metric_names` | array | Yes | Array of metric names |
| `from` | string | Yes | Start time (epoch ms or ISO 8601) |
| `to` | string | Yes | End time (epoch ms or ISO 8601) |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |

---

## newrelic_incident_ack

Acknowledge an active incident in New Relic.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `api_key` | string | Yes | New Relic User API Key |
| `account_id` | string | Yes | New Relic Account ID |
| `issue_id` | string | Yes | Issue/Incident ID to acknowledge |
| `region` | string | No | Region: `us` or `eu` (default: `us`) |

---

## Pre-built Agents

For production-ready agents using these tools, see the Agent Library:

- **[newrelic-ops](../agent-library/observability.md#newrelic-ops)** - Comprehensive New Relic observability agent

```bash
# Run the pre-built agent
aofctl run agent library://observability/newrelic-ops \
  --prompt "Check API error rate for the last hour"
```

## Rate Limits

- **NRQL Queries**: 3,000 per account per minute
- **Concurrent Requests**: 25 per user
- **Query Result Limit**: 5,000 rows per query

## Best Practices

1. **Always use time bounds in NRQL**: Include `SINCE` and `UNTIL` clauses
2. **Limit result sets**: Use `LIMIT` to prevent excessive data transfer
3. **Cache entity searches**: Entity metadata changes infrequently
4. **Batch operations**: Use concurrent queries where possible
5. **Handle pagination**: Use cursors for large result sets

## See Also

- [Observability Agents](../agent-library/observability.md) - Pre-built agents using New Relic
- [Datadog Tools](./datadog.md) - Alternative observability platform
- [Grafana Tools](./grafana.md) - Dashboard and metrics tools
- [Built-in Tools Reference](./builtin-tools.md) - Complete list of built-in tools
