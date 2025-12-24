# Opsgenie Trigger Platform

The Opsgenie trigger platform enables AOF agents to respond automatically to alert events from Opsgenie's Alert API and webhooks.

## Features

- ✅ **Alert Events**: Respond to Create, Acknowledge, Close, Escalate, AddNote, and Custom Actions
- ✅ **Integration Verification**: Secure webhook authentication via integration ID
- ✅ **Advanced Filtering**: Filter by priority, tags, teams, sources, and actions
- ✅ **Alert Management**: Acknowledge, close, add notes, and create alerts via REST API
- ✅ **Rich Metadata**: Access comprehensive alert information for informed decisions
- ✅ **Custom Actions**: Support for custom alert actions and automation

## Quick Start

### 1. Create Opsgenie Integration

1. Go to **Settings** → **Integrations** in Opsgenie
2. Click **Add Integration**
3. Search for **Webhook** or **API Integration**
4. Configure:
   - **Name**: AOF Alert Handler
   - **Webhook URL**: `https://your-aof-server.com/webhooks/opsgenie`
   - **Alert Actions**: Select actions to forward (Create, AddNote, etc.)
5. Copy the **Integration ID** from the integration details

### 2. Get API Key

1. Go to **Settings** → **Integrations** → **API**
2. Click **Create API Key**
3. Configure:
   - **Name**: AOF Agent
   - **Access Rights**: Configure & Access or Full Access
4. Copy the API key

### 3. Configure Environment Variables

```bash
export OPSGENIE_API_KEY="your-api-key-here"
export OPSGENIE_INTEGRATION_ID="your-integration-id-here"  # Optional
```

### 4. Create Trigger Resource

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: opsgenie-alert-handler
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      integration_id: ${OPSGENIE_INTEGRATION_ID}
      allowed_actions:
        - Create
        - AddNote
      priority_filter:
        - P1
        - P2
  agent: alert-response-agent
  enabled: true
```

## Configuration Reference

### Opsgenie Config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_url` | string | ✅ Yes | Opsgenie API base URL |
| `api_key` | string | ✅ Yes | API key for Opsgenie integration |
| `webhook_token` | string | ⬜ Optional | Webhook verification token (legacy) |
| `bot_name` | string | ⬜ Optional | Bot name for display (default: "aofbot") |
| `allowed_actions` | string[] | ⬜ Optional | Alert actions to process (default: all) |
| `priority_filter` | string[] | ⬜ Optional | Allowed priorities (P1-P5) |
| `tag_filter` | string[] | ⬜ Optional | Required tags (ALL must match) |
| `team_filter` | string[] | ⬜ Optional | Allowed team IDs/names |
| `source_filter` | string[] | ⬜ Optional | Allowed alert sources |
| `integration_id` | string | ⬜ Optional | Integration ID for verification |
| `verify_with_api_callback` | bool | ⬜ Optional | Verify alerts via API (default: false) |
| `enable_notes` | bool | ⬜ Optional | Enable posting notes (default: true) |
| `enable_acknowledge` | bool | ⬜ Optional | Enable acknowledging (default: true) |
| `enable_close` | bool | ⬜ Optional | Enable closing alerts (default: true) |
| `enable_create` | bool | ⬜ Optional | Enable creating alerts (default: true) |
| `rate_limit` | number | ⬜ Optional | API calls per minute (default: 600) |
| `timeout_secs` | number | ⬜ Optional | HTTP timeout (default: 30) |

### API URLs by Region

| Region | API URL |
|--------|---------|
| US | `https://api.opsgenie.com` |
| EU | `https://api.eu.opsgenie.com` |
| On-Premise | `https://opsgenie.yourcompany.com` |

### Supported Alert Actions

| Action | Description |
|--------|-------------|
| `Create` | New alert created |
| `Acknowledge` | Alert acknowledged by responder |
| `Close` | Alert closed/resolved |
| `Escalate` | Alert escalated to next level |
| `AddNote` | Note/comment added to alert |
| `CustomAction` | Custom action executed |

## Use Cases

### Auto-Diagnose Alerts

Automatically run diagnostic checks when high-priority alerts are created:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: auto-diagnose-p1-alerts
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      allowed_actions:
        - Create
      priority_filter:
        - P1
        - P2
      tag_filter:
        - production
  agent: diagnostic-agent
```

### Respond to Alert Notes

Trigger agents when oncall engineers add notes with specific commands:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: alert-command-handler
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      allowed_actions:
        - AddNote
  agent: command-processor-agent
  filter:
    text_contains: "/diagnose"
```

### Source-Based Routing

Route alerts from different monitoring systems to specialized agents:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: prometheus-alert-handler
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      source_filter:
        - prometheus
        - grafana
  agent: prometheus-specialist-agent
```

### Team-Specific Automation

Route alerts to agents based on assigned team:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: database-team-alerts
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      team_filter:
        - database-team
        - sre-team
  agent: database-automation-agent
```

## Metadata Available to Agents

When an alert event triggers an agent, the following metadata is available:

| Field | Description |
|-------|-------------|
| `action` | Alert action (Create, Acknowledge, Close, etc.) |
| `alert_id` | Opsgenie alert UUID |
| `tiny_id` | Short numeric alert ID (for display) |
| `message` | Alert title/message |
| `description` | Detailed alert description |
| `priority` | Priority level (P1-P5) |
| `tags` | Alert tags (array) |
| `source` | Source system (e.g., "datadog", "prometheus") |
| `entity` | Resource identifier (e.g., "db-prod-01") |
| `alias` | Deduplication key |
| `owner` | Current owner (who acknowledged) |
| `responders` | Assigned teams/users (array) |
| `details` | Custom fields (key-value map) |

## Filtering Examples

### By Priority

Only process P1 and P2 alerts:

```yaml
platform:
  type: opsgenie
  config:
    api_url: https://api.opsgenie.com
    api_key: ${OPSGENIE_API_KEY}
    priority_filter:
      - P1
      - P2
```

### By Tags

Only process production alerts with critical tag:

```yaml
platform:
  type: opsgenie
  config:
    api_url: https://api.opsgenie.com
    api_key: ${OPSGENIE_API_KEY}
    tag_filter:
      - production
      - critical
```

**Note**: ALL tags in `tag_filter` must be present on the alert for it to match.

### By Source

Only process alerts from specific monitoring systems:

```yaml
platform:
  type: opsgenie
  config:
    api_url: https://api.opsgenie.com
    api_key: ${OPSGENIE_API_KEY}
    source_filter:
      - datadog
      - prometheus
      - cloudwatch
```

### By Team

Only process alerts assigned to specific teams:

```yaml
platform:
  type: opsgenie
  config:
    api_url: https://api.opsgenie.com
    api_key: ${OPSGENIE_API_KEY}
    team_filter:
      - infrastructure-team
      - platform-team
```

### Combined Filters

Stack multiple filters for precise control:

```yaml
platform:
  type: opsgenie
  config:
    api_url: https://api.opsgenie.com
    api_key: ${OPSGENIE_API_KEY}
    allowed_actions:
      - Create
      - AddNote
    priority_filter:
      - P1
      - P2
    tag_filter:
      - production
    source_filter:
      - prometheus
```

## Agent Integration

### Example Diagnostic Agent

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: diagnostic-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are a diagnostic agent that responds to Opsgenie alerts.

    Your workflow:
    1. Extract alert metadata (priority, tags, source, entity)
    2. Identify the affected service/resource
    3. Run relevant diagnostic commands
    4. Analyze metrics and logs
    5. Generate diagnostic report
    6. Add findings as note to Opsgenie alert

    Use the alert context provided in the message metadata.

  tools:
    - kubectl
    - prometheus
    - elasticsearch

  mcp:
    - server: kubernetes
      enabled: true
```

### Example Command Processor Agent

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: command-processor-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are a command processor that responds to commands in Opsgenie alert notes.

    Supported commands:
    - /diagnose - Run diagnostic checks
    - /remediate - Attempt automatic remediation
    - /logs [service] - Fetch recent logs
    - /metrics [service] - Show key metrics

    When you receive an AddNote event:
    1. Parse the note text for commands
    2. Execute the requested command
    3. Add results as a note to the alert

  tools:
    - kubectl
    - prometheus
```

### Example Auto-Remediation Agent

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: auto-remediation-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are an auto-remediation agent for common production issues.

    When a P1/P2 alert triggers:
    1. Identify the issue type from alert details
    2. Check if it matches known remediation patterns
    3. Execute safe remediation steps
    4. Verify the fix worked
    5. Add detailed notes about actions taken
    6. Acknowledge or close the alert if resolved

    Safety rules:
    - Only auto-remediate known, safe patterns
    - Always add notes before taking action
    - Never make destructive changes without human approval

  tools:
    - kubectl
    - aws-cli

  mcp:
    - server: kubernetes
      enabled: true
```

## Security Considerations

### API Key Management

- ✅ Store API keys in environment variables or secrets management system
- ✅ Never commit secrets to version control
- ✅ Rotate keys periodically
- ✅ Use integration-specific API keys with minimal permissions
- ✅ Use different keys for different environments

### Integration Verification

The platform supports two verification methods:

**1. Integration ID Verification** (Recommended)
```yaml
config:
  integration_id: ${OPSGENIE_INTEGRATION_ID}
```
Verifies webhook payload contains matching integration ID.

**2. API Callback Verification** (More Secure, Higher Latency)
```yaml
config:
  verify_with_api_callback: true
```
Validates alert exists via API call to Opsgenie.

### Rate Limiting

Opsgenie API limits:
- **600 requests per minute** (default)
- Configure `rate_limit` to stay within limits
- AOF automatically respects rate limits

## Troubleshooting

### Webhooks Not Triggering

1. Verify webhook URL is correct: `https://your-server/webhooks/opsgenie`
2. Check integration is active in Opsgenie
3. Verify alert actions are selected in integration settings
4. Check AOF logs for parsing errors

**Debug Steps:**
```bash
# Check if webhooks are reaching your server
tail -f /var/log/aof/triggers.log | grep opsgenie

# Verify integration ID
echo $OPSGENIE_INTEGRATION_ID
```

### Integration ID Mismatch

```yaml
# Ensure integration ID matches Opsgenie
config:
  integration_id: ${OPSGENIE_INTEGRATION_ID}
```

Check the integration details in Opsgenie to get the correct ID.

### Events Being Filtered

Check your filter configuration is not too restrictive:

```yaml
# Too many filters? Events might be filtered out
allowed_actions: [Create]           # Only Create events
priority_filter: [P1]               # Only P1 alerts
tag_filter: [production, critical]  # BOTH tags required
source_filter: [datadog]            # Only from datadog
```

### Cannot Add Notes to Alerts

Ensure API key has correct permissions:

```bash
# Verify API key is set
echo $OPSGENIE_API_KEY

# Check it's included in config
api_key: ${OPSGENIE_API_KEY}

# Verify enable_notes is true
enable_notes: true
```

### API Rate Limiting

If hitting rate limits:

```yaml
config:
  rate_limit: 400  # Reduce from default 600
  # Or batch operations and reduce frequency
```

### EU Region Configuration

For EU Opsgenie accounts:

```yaml
config:
  api_url: https://api.eu.opsgenie.com
  api_key: ${OPSGENIE_API_KEY}
```

## Platform Capabilities

| Feature | Supported |
|---------|-----------|
| Threading | ✅ Yes (all alert actions grouped by alert ID) |
| Interactive Elements | ✅ Yes (custom actions) |
| File Attachments | ❌ No |
| Reactions | ❌ No |
| Rich Text | ✅ Yes (markdown in notes) |

## API Reference

### Add Note to Alert

```rust
platform.add_note(
    alert_id: &str,
    note: &str,
).await
```

### Acknowledge Alert

```rust
platform.acknowledge_alert(
    alert_id: &str,
    note: Option<&str>,
).await
```

### Close Alert

```rust
platform.close_alert(
    alert_id: &str,
    note: Option<&str>,
).await
```

### Create Alert

```rust
platform.create_alert(
    message: &str,
    description: Option<&str>,
    priority: Option<&str>,
    tags: Vec<String>,
    details: HashMap<String, String>,
).await
```

## Advanced Configuration

### Custom Action Handlers

Handle custom actions defined in Opsgenie:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: custom-action-handler
spec:
  platform:
    type: opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key: ${OPSGENIE_API_KEY}
      allowed_actions:
        - CustomAction
  agent: custom-action-agent
```

### Disable Specific Features

Control what the agent can do:

```yaml
config:
  enable_notes: true        # Allow adding notes
  enable_acknowledge: false # Prevent acknowledging
  enable_close: false       # Prevent closing
  enable_create: true       # Allow creating new alerts
```

### Alert Details as Context

Access custom fields in agent prompts:

```yaml
spec:
  agent: diagnostic-agent
  parameters:
    # Custom fields from alert.details are available in metadata
    use_runbook_url: "{{ metadata.details.runbook_url }}"
    affected_region: "{{ metadata.details.region }}"
```

## Related Documentation

- [Trigger Platform Architecture](/docs/architecture/triggers.md)
- [Agent Configuration](/docs/user/agents/configuration.md)
- [Opsgenie Alert API](https://docs.opsgenie.com/docs/alert-api)
- [Opsgenie Webhooks](https://docs.opsgenie.com/docs/webhook-integration)

## Examples

See complete examples in `/docs/examples/triggers/`:
- `opsgenie-alert-response.yaml` - Full alert response workflow
- `opsgenie-auto-remediation.yaml` - Automated remediation
- `opsgenie-command-processor.yaml` - Command handling from notes
