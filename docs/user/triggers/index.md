# Trigger Platforms

Triggers enable AOF agents to respond automatically to external events from various platforms. Each trigger platform provides integration with a specific service's webhook or event system.

## Supported Platforms

### Incident Management

#### [PagerDuty](pagerduty.md)
Respond to PagerDuty incident events with automated diagnostics, escalation handling, and post-incident workflows.

**Key Features:**
- Incident lifecycle events (triggered, acknowledged, resolved, escalated, reassigned)
- HMAC-SHA256 signature verification
- Filter by service, team, priority, and urgency
- Add notes to incidents via REST API

**Common Use Cases:**
- Auto-diagnose incidents when they trigger
- Escalation response automation
- Post-incident cleanup and postmortem creation

[View PagerDuty Documentation →](pagerduty.md)

---

#### [Opsgenie](opsgenie.md)
Respond to Opsgenie alert events with automated response, remediation, and alert management.

**Key Features:**
- Alert lifecycle events (Create, Acknowledge, Close, Escalate, AddNote, CustomAction)
- Integration ID verification
- Filter by priority, tags, teams, and sources
- Full alert management API (notes, acknowledge, close, create)

**Common Use Cases:**
- Auto-diagnose high-priority alerts
- Process commands from alert notes
- Source-based routing to specialized agents
- Team-specific automation

[View Opsgenie Documentation →](opsgenie.md)

---

### Chat Platforms

#### Slack
*(Coming Soon)* Respond to Slack messages, slash commands, and interactive actions.

#### Microsoft Teams
*(Coming Soon)* Respond to Teams messages and adaptive cards.

#### Discord
*(Coming Soon)* Respond to Discord messages and bot commands.

---

### Monitoring & Observability

#### Prometheus Alertmanager
*(Coming Soon)* Respond to Prometheus alerts with automated remediation.

#### Datadog
*(Coming Soon)* Respond to Datadog monitors and events.

#### Grafana
*(Coming Soon)* Respond to Grafana alerts and annotations.

---

### CI/CD & DevOps

#### GitHub
*(Coming Soon)* Respond to GitHub webhooks (issues, pull requests, releases).

#### GitLab
*(Coming Soon)* Respond to GitLab webhooks and pipeline events.

#### Jenkins
*(Coming Soon)* Respond to Jenkins build and deployment events.

---

### Cloud Platforms

#### AWS EventBridge
*(Coming Soon)* Respond to AWS CloudWatch Events and custom events.

#### Azure Event Grid
*(Coming Soon)* Respond to Azure events and resource changes.

#### Google Cloud Pub/Sub
*(Coming Soon)* Respond to GCP events and messages.

---

## Quick Start

### 1. Choose Your Platform

Select a trigger platform based on your monitoring or communication system:

```yaml
spec:
  platform:
    type: pagerduty  # or opsgenie, slack, etc.
```

### 2. Configure Credentials

Set up environment variables for authentication:

```bash
# PagerDuty
export PAGERDUTY_WEBHOOK_SECRET="your-secret"
export PAGERDUTY_API_TOKEN="your-token"

# Opsgenie
export OPSGENIE_API_KEY="your-api-key"
export OPSGENIE_INTEGRATION_ID="your-integration-id"
```

### 3. Create Trigger Resource

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: my-trigger
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      api_token: ${PAGERDUTY_API_TOKEN}
  agent: my-response-agent
  enabled: true
```

### 4. Deploy

```bash
# Apply the trigger configuration
aofctl apply trigger my-trigger.yaml

# Verify it's running
aofctl get triggers
```

## Platform Comparison

| Platform | Events | Signature Verification | Response Actions | Threading |
|----------|--------|----------------------|------------------|-----------|
| **PagerDuty** | Incident lifecycle | HMAC-SHA256 | Add notes, update status | ✅ Yes |
| **Opsgenie** | Alert lifecycle | Integration ID | Notes, acknowledge, close, create | ✅ Yes |
| Slack | Messages, commands | Signing secret | Messages, threads, reactions | ✅ Yes |
| Teams | Messages, actions | JWT verification | Messages, adaptive cards | ✅ Yes |

## Architecture

### Webhook Flow

```
External Platform → Webhook → AOF Trigger → Agent → Response → Platform
```

1. **Platform Event**: External system generates event (incident, alert, message)
2. **Webhook Delivery**: Platform sends webhook to AOF trigger endpoint
3. **Signature Verification**: AOF verifies webhook authenticity
4. **Event Parsing**: Webhook payload parsed into TriggerMessage
5. **Filtering**: Event checked against configured filters
6. **Agent Invocation**: Matching agent is triggered with event context
7. **Agent Processing**: Agent executes logic, generates response
8. **Response Delivery**: Response sent back to platform (note, message, etc.)

### Trigger Resource

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: trigger-name
  namespace: default
spec:
  # Platform configuration
  platform:
    type: pagerduty | opsgenie | slack | teams
    config:
      # Platform-specific configuration

  # Agent to invoke
  agent: agent-name

  # Optional filters
  filter:
    metadata.priority: P1
    text_contains: "/diagnose"

  # Optional parameters passed to agent
  parameters:
    key: value

  # Enable/disable trigger
  enabled: true
```

## Best Practices

### Security

1. **Always verify signatures**: Every platform provides a mechanism to verify webhook authenticity
2. **Use environment variables**: Never hardcode secrets in configuration files
3. **Rotate credentials regularly**: Set up periodic rotation of API keys and tokens
4. **Use minimal permissions**: Grant only required permissions to API tokens
5. **Monitor for anomalies**: Track webhook volume and API usage

### Filtering

1. **Filter early**: Use platform filters to reduce unnecessary agent invocations
2. **Combine filters**: Stack multiple filters for precise event matching
3. **Test filters thoroughly**: Verify filters match expected events
4. **Document filter logic**: Explain why specific filters are configured

### Performance

1. **Avoid blocking operations**: Keep webhook handlers fast to prevent timeouts
2. **Use async processing**: Offload heavy processing to background tasks
3. **Implement retries**: Handle transient failures gracefully
4. **Monitor latency**: Track webhook processing time

### Reliability

1. **Handle duplicates**: Many platforms retry webhooks, implement idempotency
2. **Validate payloads**: Check for required fields before processing
3. **Log everything**: Comprehensive logging aids debugging
4. **Set up alerts**: Monitor trigger health and error rates

## Configuration Patterns

### Multi-Environment Setup

Different configurations for dev/staging/prod:

```yaml
# production.yaml
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET_PROD}
      min_priority: "P2"  # Only P1 and P2 in production

# staging.yaml
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET_STAGING}
      # All priorities in staging
```

### Agent Routing

Route different events to different agents:

```yaml
# High-priority incidents → senior SRE agent
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: p1-incident-handler
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      min_priority: "P1"
  agent: senior-sre-agent

---
# Lower-priority incidents → junior agent
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: p3-incident-handler
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      min_priority: "P3"
  agent: junior-sre-agent
```

### Command Processing

Process commands from notes/messages:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: command-processor
spec:
  platform:
    type: opsgenie
    config:
      api_key: ${OPSGENIE_API_KEY}
      allowed_actions:
        - AddNote
  agent: command-processor-agent
  filter:
    text_contains: "/"  # Only process notes with commands
```

## Troubleshooting

### Common Issues

#### Webhooks Not Arriving

1. Check webhook URL is correct and accessible
2. Verify firewall/network rules allow incoming webhooks
3. Check platform webhook configuration is active
4. Review platform's webhook delivery logs

#### Signature Verification Failures

1. Verify webhook secret matches in both systems
2. Check for extra whitespace in secret values
3. Ensure secret hasn't been rotated without updating configuration
4. Review signature header name (varies by platform)

#### Events Being Filtered

1. Review filter configuration for overly restrictive rules
2. Check event payload matches expected structure
3. Verify metadata fields exist before filtering on them
4. Test with minimal filters first, then add incrementally

#### Agent Not Triggering

1. Verify agent exists and is enabled
2. Check agent name matches trigger configuration
3. Review agent logs for errors
4. Ensure agent has required tools/MCP servers configured

## Related Documentation

- [Agent Configuration](/docs/user/agents/configuration.md)
- [Trigger Architecture](/docs/architecture/triggers.md)
- [Security Best Practices](/docs/user/security.md)
- [Webhook Server Setup](/docs/guides/webhook-server.md)

## Examples

Browse complete examples:

```bash
# List all trigger examples
ls docs/examples/triggers/

# View specific example
cat docs/examples/triggers/pagerduty-incident-response.yaml
```

Example files:
- `pagerduty-incident-response.yaml` - Full PagerDuty incident workflow
- `opsgenie-alert-response.yaml` - Opsgenie alert automation
- `opsgenie-command-processor.yaml` - Command handling pattern
- `multi-platform-routing.yaml` - Route different platforms to different agents
