# CLI Reference: Channel Commands

Manage multi-channel gateway routes and send outbound notifications.

## `agentix channels`

List all configured channel routes from the gateway.

### Synopsis

```
agentix channels [OPTIONS]
```

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output json` | text | Output raw JSON instead of formatted table |
| `--gateway-url URL` | `http://127.0.0.1:7777` | Gateway base URL |

### Examples

```bash
# List all configured channel routes
agentix channels

# JSON output
agentix channels --output json
agentix channels --output json | jq '.[] | select(.platform == "slack")'
```

### Output Format

```
PLATFORM    CHANNEL ID    AGENT              DIRECTION       DESCRIPTION
---------------------------------------------------------------------------
slack       C_DEVOPS      dba-optimizer      bidirectional   DevOps team channel
slack       C_ALERTS      cost-monitor       outbound        Alerts channel
telegram    -100123456    dba-optimizer      bidirectional   On-call group
discord     123456789     devops-assistant   bidirectional   Discord support

4 route(s) configured.
```

---

## `agentix notify`

Send an outbound notification to a configured channel.

### Synopsis

```
agentix notify [OPTIONS]
```

### Flags

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--platform PLATFORM` | Yes | - | Target platform (`slack`, `telegram`, `discord`) |
| `--channel-id ID` | Yes | - | Target channel identifier |
| `--title TEXT` | Yes | - | Notification title |
| `--body TEXT` | Yes | - | Notification body text |
| `--severity LEVEL` | No | `info` | Severity level: `info`, `warning`, `error`, `critical` |
| `--output json` | No | text | Output raw JSON instead of formatted text |
| `--gateway-url URL` | No | `http://127.0.0.1:7777` | Gateway base URL |

### Examples

```bash
# Send an info notification to Slack
agentix notify \
  --platform slack \
  --channel-id C_ALERTS \
  --title "Deploy Complete" \
  --body "v2.1.0 deployed to production"

# Send a warning notification
agentix notify \
  --platform slack \
  --channel-id C_ALERTS \
  --title "Budget Alert" \
  --body "Monthly spend exceeded $500 threshold" \
  --severity warning

# Send to Telegram
agentix notify \
  --platform telegram \
  --channel-id "-100123456" \
  --title "Incident Detected" \
  --body "High CPU usage on prod-web-03" \
  --severity error

# Send to Discord
agentix notify \
  --platform discord \
  --channel-id "123456789" \
  --title "Security Alert" \
  --body "Unauthorized access attempt blocked" \
  --severity critical
```

### Severity Levels

| Level | Description | Slack | Telegram | Discord |
|-------|-------------|-------|----------|---------|
| `info` | Run completed, status update | Green sidebar | Info emoji | Green embed |
| `warning` | Budget threshold, degraded state | Amber sidebar | Warning emoji | Amber embed |
| `error` | Run failed, tool error | Red sidebar | Error emoji | Red embed |
| `critical` | System down, security breach | Bold red sidebar | Critical emoji | Bright red embed |

## REST API Equivalents

| CLI command | REST endpoint |
|-------------|---------------|
| `agentix channels` | `GET /api/v1/channels` |
| `agentix notify` | `POST /api/v1/notify` |

### POST /api/v1/notify Request Body

```json
{
  "platform": "slack",
  "channel_id": "C_ALERTS",
  "agent_name": "cli",
  "title": "Budget Alert",
  "body": "Monthly spend exceeded threshold",
  "severity": "warning"
}
```

## Related

- [Multi-channel gateway concepts](../concepts/multi-channel-gateway.md) -- architecture, routing, and security
- [Channel config reference](./channel-config.md) -- full YAML configuration reference
- [Quickstart: multi-channel](../../quickstart/agentix-channels.yaml) -- example workspace config
