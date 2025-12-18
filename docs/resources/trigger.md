# Trigger Resource

## Overview

The `Trigger` resource defines event sources that initiate agent workflows. Triggers watch for events from various platforms (Slack, Telegram, Discord, webhooks, schedules, PagerDuty, Jira) and activate bound flows when conditions are met.

## API Reference

**apiVersion:** `aof.dev/v1`
**kind:** `Trigger`

## Specification

### Trigger Spec Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Trigger type. Options: `Slack`, `Telegram`, `Discord`, `HTTP`, `Schedule`, `PagerDuty`, `Jira` |
| `config` | object | Yes | Type-specific configuration. See [Trigger Types](#trigger-types) below |

## Trigger Types

### Slack

Monitors Slack channels for messages matching patterns.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Slack bot token (xoxb-...) for API access |
| `app_token` | string | Yes | Slack app token (xapp-...) for Socket Mode |
| `channels` | []string | Yes | List of channel IDs to monitor (e.g., `C01234567`) |
| `users` | []string | No | Filter to specific user IDs. If empty, accepts from any user |
| `patterns` | []string | No | Regular expressions to match messages. If empty, matches all messages |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-oncall
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    app_token: ${SLACK_APP_TOKEN}
    channels:
      - C01234567  # #oncall-alerts
      - C76543210  # #incident-response
    users:
      - U11111111  # @alice
      - U22222222  # @bob
    patterns:
      - "^@agent.*"
      - "incident.*help"
```

### Telegram

Monitors Telegram chats for messages matching patterns.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Telegram bot token from BotFather |
| `allowed_chat_ids` | []string | Yes | List of chat IDs authorized to trigger flows |
| `allowed_users` | []string | No | Filter to specific username or user IDs. If empty, accepts from any user |
| `patterns` | []string | No | Regular expressions to match messages. If empty, matches all messages |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-ops
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
    allowed_chat_ids:
      - "-1001234567890"  # @ops-team group
      - "123456789"       # Direct message from admin
    allowed_users:
      - "alice_ops"
      - "bob_admin"
    patterns:
      - "^/agent.*"
      - "help.*k8s"
```

### Discord

Monitors Discord channels for messages matching patterns.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Discord bot token |
| `channels` | []string | Yes | List of channel IDs to monitor |
| `users` | []string | No | Filter to specific user IDs. If empty, accepts from any user |
| `patterns` | []string | No | Regular expressions to match messages. If empty, matches all messages |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-devops
spec:
  type: Discord
  config:
    bot_token: ${DISCORD_BOT_TOKEN}
    channels:
      - "1234567890123456"  # #devops
      - "9876543210987654"  # #alerts
    users:
      - "111222333444555"  # @alice
    patterns:
      - "^!agent.*"
      - "k8s.*issue"
```

### HTTP

Accepts webhook HTTP requests to trigger flows.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `port` | integer | Yes | HTTP server port to listen on |
| `path` | string | Yes | URL path for webhook endpoint (e.g., `/webhook/github`) |
| `method` | string | No | HTTP method to accept. Defaults to `POST` |
| `secret` | string | No | Shared secret for request verification (e.g., GitHub webhook secret) |
| `headers` | map[string]string | No | Required HTTP headers for validation |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-webhook
spec:
  type: HTTP
  config:
    port: 8080
    path: /webhook/github
    method: POST
    secret: ${GITHUB_WEBHOOK_SECRET}
    headers:
      X-GitHub-Event: push
```

### Schedule

Triggers flows on a cron schedule.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cron` | string | Yes | Cron expression (e.g., `0 */6 * * *` for every 6 hours) |
| `timezone` | string | No | IANA timezone name. Defaults to `UTC` |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: daily-report
spec:
  type: Schedule
  config:
    cron: "0 9 * * MON-FRI"
    timezone: America/New_York
```

### PagerDuty

Monitors PagerDuty incidents and triggers flows.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string | Yes | PagerDuty API key |
| `service_ids` | []string | No | Filter to specific service IDs. If empty, monitors all services |
| `urgency` | string | No | Filter by urgency: `high`, `low`, or empty for all |
| `status` | []string | No | Filter by status: `triggered`, `acknowledged`, `resolved`. Defaults to `[triggered]` |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-critical
spec:
  type: PagerDuty
  config:
    api_key: ${PAGERDUTY_API_KEY}
    service_ids:
      - PXXXXXX  # Production API
      - PYYYYYY  # Production Database
    urgency: high
    status:
      - triggered
```

### Jira

Monitors Jira issues and triggers flows on events.

#### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | string | Yes | Jira instance URL (e.g., `https://company.atlassian.net`) |
| `username` | string | Yes | Jira username or email |
| `api_token` | string | Yes | Jira API token |
| `project_keys` | []string | Yes | List of Jira project keys to monitor |
| `issue_types` | []string | No | Filter by issue types (e.g., `Bug`, `Incident`). If empty, monitors all types |
| `labels` | []string | No | Filter by labels. If empty, monitors all issues |
| `events` | []string | No | Jira events to monitor. Defaults to `[issue_created]`. Options: `issue_created`, `issue_updated`, `comment_added` |

#### Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-incidents
spec:
  type: Jira
  config:
    url: https://company.atlassian.net
    username: bot@example.com
    api_token: ${JIRA_API_TOKEN}
    project_keys:
      - PROD
      - OPS
    issue_types:
      - Incident
      - Bug
    labels:
      - critical
      - production
    events:
      - issue_created
      - comment_added
```

## CLI Usage

### List Triggers

```bash
# List all triggers
aofctl get triggers

# Output:
# NAME                TYPE         CONFIG
# slack-oncall        Slack        2 channels, 2 users
# telegram-ops        Telegram     1 chat, 2 users
# github-webhook      HTTP         :8080/webhook/github
# daily-report        Schedule     0 9 * * MON-FRI
```

### Describe Trigger

```bash
# View detailed trigger configuration
aofctl describe trigger slack-oncall

# Output:
# Name:         slack-oncall
# Type:         Slack
#
# Configuration:
#   Channels:
#     - C01234567 (#oncall-alerts)
#     - C76543210 (#incident-response)
#   Users:
#     - U11111111 (@alice)
#     - U22222222 (@bob)
#   Patterns:
#     - ^@agent.*
#     - incident.*help
#
# Status:
#   Connected: true
#   Last Event: 2025-12-18T10:23:45Z
```

### Create Trigger

```bash
# Create trigger from YAML file
aofctl apply -f trigger-slack.yaml

# Create Slack trigger inline
aofctl create trigger slack-ops \
  --type=Slack \
  --bot-token=${SLACK_BOT_TOKEN} \
  --app-token=${SLACK_APP_TOKEN} \
  --channel=C01234567

# Create schedule trigger inline
aofctl create trigger nightly-backup \
  --type=Schedule \
  --cron="0 2 * * *" \
  --timezone=UTC
```

### Update Trigger

```bash
# Update trigger from modified YAML
aofctl apply -f trigger-slack-updated.yaml

# Edit trigger interactively
aofctl edit trigger slack-oncall
```

### Delete Trigger

```bash
# Delete specific trigger
aofctl delete trigger slack-oncall

# Delete multiple triggers
aofctl delete triggers slack-oncall telegram-ops
```

### Test Trigger

```bash
# Test trigger connectivity and configuration
aofctl test trigger slack-oncall

# Output:
# Testing trigger 'slack-oncall'...
# ✓ Slack API connection successful
# ✓ Bot has access to 2/2 channels
# ✓ Pattern matching configured correctly
```

## Common Patterns

### Multi-Channel Slack Support

Monitor multiple Slack channels with different patterns:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-support
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    app_token: ${SLACK_APP_TOKEN}
    channels:
      - C01111111  # #support
      - C02222222  # #urgent-support
      - C03333333  # #vip-support
    patterns:
      - "@agent.*"
      - "help.*"
      - "urgent.*"
```

### Webhook with Security

GitHub-style webhook with signature verification:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-ci
spec:
  type: HTTP
  config:
    port: 8080
    path: /webhook/github
    method: POST
    secret: ${GITHUB_WEBHOOK_SECRET}
    headers:
      X-GitHub-Event: push
      X-Hub-Signature-256: "*"
```

### Business Hours Schedule

Trigger only during business hours:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: business-hours-check
spec:
  type: Schedule
  config:
    cron: "0 9-17 * * MON-FRI"  # 9 AM - 5 PM, weekdays
    timezone: America/New_York
```

### Multi-Platform Incident Response

Respond to incidents from multiple sources:

```yaml
# PagerDuty trigger
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  type: PagerDuty
  config:
    api_key: ${PAGERDUTY_API_KEY}
    urgency: high
---
# Jira trigger
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-incidents
spec:
  type: Jira
  config:
    url: https://company.atlassian.net
    username: bot@example.com
    api_token: ${JIRA_API_TOKEN}
    project_keys: [PROD]
    issue_types: [Incident]
    labels: [critical]
---
# Slack trigger
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-incidents
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    app_token: ${SLACK_APP_TOKEN}
    channels: [C01234567]
    patterns: ["incident.*"]
```

## Best Practices

1. **Secret Management**: Always use `${VAR}` syntax for tokens and secrets - never hardcode credentials
2. **Pattern Specificity**: Use specific regex patterns to reduce false positives and unnecessary flow executions
3. **User Filtering**: Restrict triggers to authorized users/channels to prevent abuse
4. **Webhook Security**: Always use secrets and header validation for HTTP triggers
5. **Schedule Timezones**: Explicitly specify timezones for schedule triggers to avoid confusion
6. **Testing**: Use `aofctl test trigger` to verify connectivity before deploying to production
7. **Monitoring**: Enable trigger status monitoring to detect connectivity issues early
8. **Rate Limiting**: Consider implementing rate limits in triggers to prevent overload

## Troubleshooting

### Trigger Not Firing

```bash
# Check trigger status
aofctl describe trigger slack-oncall

# View trigger logs
aofctl logs trigger slack-oncall

# Test connectivity
aofctl test trigger slack-oncall
```

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Slack events not received | Bot not invited to channel | Invite bot to channel with `/invite @bot` |
| Webhook 401 errors | Invalid secret | Verify webhook secret matches platform configuration |
| Schedule not executing | Timezone mismatch | Check timezone setting and verify cron expression |
| PagerDuty connection failed | Invalid API key | Regenerate API key with correct permissions |
| Pattern not matching | Incorrect regex | Test regex at https://regex101.com |

## Related Resources

- [Context Resource](context.md) - Define execution environments
- [FlowBinding Resource](binding.md) - Connect triggers to flows
- [Flow Resource](../concepts/flows.md) - Define agent workflows

## Kubernetes CRD Compatibility

AOF Trigger resources are designed for **Kubernetes Operator** deployment, following CRD conventions.

### CRD Metadata

| Field | Value |
|-------|-------|
| API Group | `aof.dev` |
| Version | `v1` |
| Kind | `Trigger` |
| Scope | `Namespaced` |
| Plural | `triggers` |
| Singular | `trigger` |
| Short Names | `trg` |

### Status Fields (Operator-Managed)

When deployed via a Kubernetes Operator, Trigger resources will include runtime status:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-oncall
  namespace: aof-system
spec:
  # User-defined spec fields (see above)
status:
  connected: true
  phase: Active
  conditions:
    - type: Connected
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: ConnectionEstablished
      message: "Successfully connected to Slack API"
    - type: ChannelsAccessible
      status: "True"
      lastTransitionTime: "2025-12-18T10:00:00Z"
      reason: BotInvited
      message: "Bot has access to 2/2 configured channels"
  lastEvent:
    timestamp: "2025-12-18T10:23:45Z"
    type: message
    source: "C01234567"
    user: "U11111111"
  eventCount: 142
  errorCount: 0
  webhookEndpoint: "https://aof-operator.example.com/triggers/slack-oncall"
  observedGeneration: 1
```

### Status Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `connected` | boolean | Whether trigger is connected to platform |
| `phase` | string | Trigger phase: `Pending`, `Active`, `Failed`, `Disconnected` |
| `conditions` | []Condition | Detailed status conditions |
| `lastEvent` | object | Information about the most recent event received |
| `eventCount` | integer | Total number of events processed since creation |
| `errorCount` | integer | Number of errors encountered |
| `webhookEndpoint` | string | Webhook URL for HTTP triggers (operator-assigned) |
| `observedGeneration` | integer | Generation of the resource last processed |

### Platform Connection Status

Each trigger type reports platform-specific status:

**Slack Trigger:**
```yaml
status:
  connected: true
  platformStatus:
    channels:
      - id: C01234567
        name: "#oncall-alerts"
        accessible: true
      - id: C76543210
        name: "#incident-response"
        accessible: true
    socketMode: connected
    lastHeartbeat: "2025-12-18T10:23:00Z"
```

**HTTP Trigger:**
```yaml
status:
  connected: true
  platformStatus:
    endpoint: "https://aof-operator.example.com/triggers/github-webhook"
    tlsEnabled: true
    certificateExpiry: "2026-12-18T00:00:00Z"
```

**Schedule Trigger:**
```yaml
status:
  connected: true
  platformStatus:
    nextExecution: "2025-12-19T09:00:00Z"
    lastExecution: "2025-12-18T09:00:00Z"
    executionCount: 245
```

### Namespace Support

Triggers are namespaced resources for tenant isolation:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
  namespace: team-production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN_PROD}
    # ...
```

### Cross-Namespace References

FlowBindings can reference triggers in different namespaces:

```yaml
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-binding
  namespace: workflows
spec:
  trigger:
    name: slack-prod
    namespace: team-production  # Cross-namespace reference
  context: prod
  flow: k8s-troubleshoot
```

### Webhook Endpoint Management

HTTP triggers receive operator-assigned webhook endpoints:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-webhook
  namespace: integrations
spec:
  type: HTTP
  config:
    path: /webhook/github
    secret: ${GITHUB_WEBHOOK_SECRET}
status:
  webhookEndpoint: "https://aof-operator.example.com/triggers/github-webhook"
  # Use this URL in GitHub webhook configuration
```

### Kubernetes Deployment

Once the AOF Operator is available, triggers can be managed via kubectl:

```bash
# Apply trigger
kubectl apply -f trigger-slack.yaml

# Get triggers
kubectl get triggers -n aof-system

# Describe trigger with status
kubectl describe trigger slack-oncall -n aof-system

# Check connection status
kubectl get trigger slack-oncall -n aof-system -o jsonpath='{.status.connected}'

# View event count
kubectl get trigger slack-oncall -n aof-system -o jsonpath='{.status.eventCount}'

# Monitor triggers
kubectl get triggers -n aof-system --watch
```

### Event Metrics

Triggers expose Prometheus-compatible metrics:

```yaml
status:
  metrics:
    eventsReceived: 1420
    eventsProcessed: 1418
    eventsDropped: 2
    averageProcessingTime: "45ms"
    errorRate: 0.001
```

> **Note**: Kubernetes Operator support is planned for a future release. See [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) for complete CRD definitions.

## See Also

- [Trigger Configuration Guide](../guides/triggers.md) - Detailed trigger setup instructions
- [Security Best Practices](../concepts/security-approval.md) - Secure trigger configuration
- [Platform Integration](../guides/integrations.md) - Platform-specific integration guides
- [Kubernetes CRDs Reference](../reference/kubernetes-crds.md) - Complete CRD definitions and operator architecture
