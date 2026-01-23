---
sidebar_position: 12
sidebar_label: Grafana
---

# Grafana MCP Server

Interact with Grafana dashboards, alerts, and data sources for observability automation.

## Installation

```bash
# Using npx
npx -y @anthropic/mcp-server-grafana

# Or via npm
npm install -g @anthropic/mcp-server-grafana
```

## Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: grafana-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: grafana
      command: npx
      args: ["-y", "@anthropic/mcp-server-grafana"]
      env:
        GRAFANA_URL: https://grafana.example.com
        GRAFANA_API_KEY: ${GRAFANA_API_KEY}
```

### With Service Account Token

```yaml
mcp_servers:
  - name: grafana
    command: npx
    args: ["-y", "@anthropic/mcp-server-grafana"]
    env:
      GRAFANA_URL: https://grafana.example.com
      GRAFANA_SERVICE_ACCOUNT_TOKEN: ${GRAFANA_SA_TOKEN}
```

## Available Tools

### search_dashboards

Search for dashboards by query or tags.

```json
{
  "name": "search_dashboards",
  "arguments": {
    "query": "kubernetes",
    "tags": ["production", "k8s"],
    "type": "dash-db"
  }
}
```

**Parameters**:
- `query` (optional): Search query string
- `tags` (optional): Filter by tags
- `type` (optional): Type filter (dash-db, dash-folder)
- `limit` (optional): Max results (default: 100)

### get_dashboard

Get dashboard by UID.

```json
{
  "name": "get_dashboard",
  "arguments": {
    "uid": "k8s-cluster-overview"
  }
}
```

**Parameters**:
- `uid` (required): Dashboard UID

### get_dashboard_panels

Get panel definitions from a dashboard.

```json
{
  "name": "get_dashboard_panels",
  "arguments": {
    "uid": "k8s-cluster-overview"
  }
}
```

### query_data_source

Query a Grafana data source directly.

```json
{
  "name": "query_data_source",
  "arguments": {
    "datasource_uid": "prometheus",
    "query": "up{job='kubernetes-pods'}",
    "from": "now-1h",
    "to": "now"
  }
}
```

**Parameters**:
- `datasource_uid` (required): Data source UID
- `query` (required): Query string (format depends on data source type)
- `from` (optional): Start time (default: now-6h)
- `to` (optional): End time (default: now)

### get_alerts

Get firing alerts from Grafana Alerting.

```json
{
  "name": "get_alerts",
  "arguments": {
    "state": "firing",
    "labels": {"severity": "critical"}
  }
}
```

**Parameters**:
- `state` (optional): Alert state filter (firing, pending, normal)
- `labels` (optional): Label filters

### get_alert_rules

List alert rules.

```json
{
  "name": "get_alert_rules",
  "arguments": {
    "folder_uid": "production-alerts"
  }
}
```

**Parameters**:
- `folder_uid` (optional): Filter by folder
- `dashboard_uid` (optional): Filter by dashboard

### create_annotation

Create an annotation on a dashboard.

```json
{
  "name": "create_annotation",
  "arguments": {
    "dashboard_uid": "k8s-overview",
    "text": "Deployment: v1.2.3 rolled out",
    "tags": ["deployment", "production"],
    "time": 1705320000000
  }
}
```

**Parameters**:
- `text` (required): Annotation text
- `dashboard_uid` (optional): Dashboard to annotate
- `panel_id` (optional): Specific panel
- `tags` (optional): Annotation tags
- `time` (optional): Timestamp (epoch ms, default: now)

### get_annotations

Get annotations for time range.

```json
{
  "name": "get_annotations",
  "arguments": {
    "dashboard_uid": "k8s-overview",
    "from": "now-24h",
    "to": "now",
    "tags": ["deployment"]
  }
}
```

### list_data_sources

List configured data sources.

```json
{
  "name": "list_data_sources",
  "arguments": {}
}
```

## Use Cases

### Dashboard Navigator Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dashboard-navigator
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Help users find and understand Grafana dashboards.

    When asked about metrics or dashboards:
    1. Search for relevant dashboards
    2. Explain what each dashboard shows
    3. Provide direct links to dashboards
    4. Query specific metrics if needed
  mcp_servers:
    - name: grafana
      command: npx
      args: ["-y", "@anthropic/mcp-server-grafana"]
      env:
        GRAFANA_URL: ${GRAFANA_URL}
        GRAFANA_API_KEY: ${GRAFANA_API_KEY}
```

### Alert Manager Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: alert-manager
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Monitor and manage Grafana alerts.

    Responsibilities:
    - Check firing alerts regularly
    - Correlate alerts with metrics
    - Create annotations for incidents
    - Provide runbook guidance
  mcp_servers:
    - name: grafana
      command: npx
      args: ["-y", "@anthropic/mcp-server-grafana"]
      env:
        GRAFANA_URL: ${GRAFANA_URL}
        GRAFANA_API_KEY: ${GRAFANA_API_KEY}
```

### Incident Annotator

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: incident-annotator
spec:
  trigger:
    type: PagerDuty
    config:
      events: [incident.triggered]
  nodes:
    - id: annotate
      type: Agent
      config:
        agent: annotator
        prompt: |
          Create a Grafana annotation for this incident:
          Title: {{trigger.incident.title}}
          Service: {{trigger.incident.service.name}}

          Tag with: incident, {{trigger.incident.urgency}}
```

## Security Considerations

1. **API Key Scope**: Use minimal permissions (Viewer for read-only agents)
2. **Service Accounts**: Prefer service accounts over user API keys
3. **Folder Permissions**: Restrict access to sensitive dashboards
4. **Audit Trail**: Grafana logs all API access

### Permission Levels

| Use Case | Required Permission |
|----------|-------------------|
| Read dashboards/alerts | Viewer |
| Create annotations | Editor |
| Modify alert rules | Admin |

## Troubleshooting

### Authentication Issues

```bash
# Test API key
curl -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  ${GRAFANA_URL}/api/org

# Check key permissions
curl -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  ${GRAFANA_URL}/api/user/permissions
```

### Data Source Queries

```bash
# List data sources
curl -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  ${GRAFANA_URL}/api/datasources

# Test data source
curl -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  ${GRAFANA_URL}/api/datasources/uid/prometheus/health
```

## Related

- [Prometheus MCP Server](./prometheus.md)
- [Alert Manager Agent](/docs/agent-library/observability/alert-manager)
- [SLO Guardian Agent](/docs/agent-library/observability/slo-guardian)
