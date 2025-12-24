# PagerDuty Trigger Platform

The PagerDuty trigger platform enables AOF agents to respond automatically to incident events from PagerDuty's V3 Webhooks API.

## Features

- ✅ **Incident Events**: Respond to triggered, acknowledged, resolved, escalated, and reassigned incidents
- ✅ **HMAC-SHA256 Signature Verification**: Secure webhook authentication
- ✅ **Advanced Filtering**: Filter by service, team, priority, and urgency
- ✅ **Incident Notes**: Add agent responses as notes to incidents via REST API
- ✅ **Rich Metadata**: Access detailed incident information for informed decisions

## Quick Start

### 1. Create PagerDuty Webhook

1. Go to **Integrations** → **Generic Webhooks (v3)** in PagerDuty
2. Click **New Webhook**
3. Configure:
   - **Webhook URL**: `https://your-aof-server.com/webhooks/pagerduty`
   - **Scope Type**: Account or Service
   - **Event Subscription**: Select incident events (triggered, acknowledged, resolved, etc.)
4. Copy the **Webhook Secret** (you'll need this for configuration)

### 2. Get API Token (Optional)

For adding notes to incidents, create a PagerDuty API token:

1. Go to **Integrations** → **API Access Keys**
2. Click **Create New API Key**
3. Copy the token (starts with `u+...`)

### 3. Configure Environment Variables

```bash
export PAGERDUTY_WEBHOOK_SECRET="your-webhook-secret-here"
export PAGERDUTY_API_TOKEN="your-api-token-here"  # Optional
```

### 4. Create Trigger Resource

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: pagerduty-incident-handler
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      api_token: ${PAGERDUTY_API_TOKEN}
      event_types:
        - incident.triggered
        - incident.acknowledged
        - incident.resolved
  agent: incident-response-agent
  enabled: true
```

## Configuration Reference

### PagerDuty Config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `webhook_secret` | string | ✅ Yes | Webhook secret for signature verification |
| `api_token` | string | ⬜ Optional | API token for adding notes to incidents |
| `bot_name` | string | ⬜ Optional | Bot name for display (default: "aofbot") |
| `event_types` | string[] | ⬜ Optional | Event types to process (default: all) |
| `allowed_services` | string[] | ⬜ Optional | Allowed service IDs |
| `allowed_teams` | string[] | ⬜ Optional | Allowed team IDs |
| `min_priority` | string | ⬜ Optional | Minimum priority ("P1" - "P5") |
| `min_urgency` | string | ⬜ Optional | Minimum urgency ("high" or "low") |

### Supported Event Types

| Event Type | Description |
|------------|-------------|
| `incident.triggered` | New incident created |
| `incident.acknowledged` | Incident acknowledged by responder |
| `incident.resolved` | Incident marked as resolved |
| `incident.escalated` | Incident escalated to next level |
| `incident.reassigned` | Incident reassigned to different user/team |

**Note**: PagerDuty V2 webhooks are deprecated. This implementation uses V3 webhooks only.

## Use Cases

### Auto-Diagnose Incidents

Automatically run diagnostic checks when incidents trigger:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: auto-diagnose-incidents
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.triggered
  agent: diagnostic-agent
  filter:
    metadata.service: "production-api"
```

### Escalation Response

Trigger senior SRE agent when incident escalates:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: escalation-response
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.escalated
      allowed_services:
        - production-database
        - payment-service
  agent: senior-sre-agent
```

### Post-Incident Cleanup

Trigger cleanup tasks when incident resolves:

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: post-incident-cleanup
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.resolved
  agent: cleanup-agent
  parameters:
    create_postmortem: true
    cleanup_temp_resources: true
```

## Metadata Available to Agents

When an incident event triggers an agent, the following metadata is available:

| Field | Description |
|-------|-------------|
| `event_id` | Unique event ID |
| `event_type` | Event type (e.g., "incident.triggered") |
| `occurred_at` | Timestamp when event occurred |
| `incident_id` | PagerDuty incident ID |
| `incident_number` | Human-readable incident number |
| `incident_key` | Deduplication key |
| `status` | Current incident status |
| `urgency` | Urgency level (high/low) |
| `priority` | Priority (P1-P5) |
| `html_url` | Web UI URL for incident |
| `service_id` | Service ID |
| `service_name` | Service name |
| `team_ids` | Team IDs (array) |
| `assignee_ids` | Assignee IDs (array) |

## Filtering Examples

### By Service

Only process incidents for specific services:

```yaml
platform:
  type: pagerduty
  config:
    webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
    allowed_services:
      - PXYZ123  # Production API
      - PXYZ456  # Payment Service
```

### By Team

Only process incidents assigned to specific teams:

```yaml
platform:
  type: pagerduty
  config:
    webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
    allowed_teams:
      - P456DEF  # Infrastructure Team
```

### By Priority

Only process P1 and P2 incidents:

```yaml
platform:
  type: pagerduty
  config:
    webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
    min_priority: "P2"  # Processes P1 and P2
```

### By Urgency

Only process high urgency incidents:

```yaml
platform:
  type: pagerduty
  config:
    webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
    min_urgency: "high"
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
    You are a diagnostic agent that runs automated checks when PagerDuty incidents trigger.

    Your workflow:
    1. Extract incident metadata (service, urgency, priority)
    2. Run relevant diagnostic commands
    3. Analyze logs and metrics
    4. Generate diagnostic report
    5. Add findings as note to PagerDuty incident

    Use the incident context provided in the message metadata.

  tools:
    - kubectl
    - prometheus
    - elasticsearch

  mcp:
    - server: kubernetes
      enabled: true
```

### Example Cleanup Agent

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: cleanup-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are a post-incident cleanup agent.

    When an incident is resolved, you should:
    1. Create incident postmortem template
    2. Collect relevant logs and artifacts
    3. Clean up temporary resources
    4. Update documentation
    5. Create follow-up tasks

  tools:
    - kubectl
    - git
```

## Security Considerations

### Webhook Secret Management

- ✅ Store webhook secret in environment variables or secrets management system
- ✅ Never commit secrets to version control
- ✅ Rotate secrets periodically
- ✅ Use different secrets for different environments

### Signature Verification

The platform automatically verifies the `x-pagerduty-signature` header using HMAC-SHA256:

```
Signature Format: v1=<hex_encoded_signature>
Algorithm: HMAC-SHA256(webhook_secret, raw_payload)
```

All requests without valid signatures are rejected.

### API Token Security

- ✅ API tokens have full account access - protect carefully
- ✅ Use service accounts with minimal required permissions
- ✅ Rotate tokens regularly
- ✅ Monitor API usage for anomalies

## Troubleshooting

### Webhooks Not Triggering

1. Verify webhook URL is correct: `https://your-server/webhooks/pagerduty`
2. Check webhook secret matches in both PagerDuty and AOF config
3. Verify event types are selected in PagerDuty webhook settings
4. Check AOF logs for signature verification errors

### Signature Verification Failures

```bash
# Check webhook secret matches
echo $PAGERDUTY_WEBHOOK_SECRET

# Verify it matches the secret in PagerDuty webhook settings
```

### Events Being Filtered

Check your filter configuration:

```yaml
# Too restrictive? Events might be filtered out
event_types: [incident.triggered]  # Only this event type
allowed_services: [PXYZ123]        # Only this service
min_priority: "P1"                 # Only P1 incidents
```

### Cannot Add Notes to Incidents

Ensure API token is configured:

```bash
# Verify API token is set
echo $PAGERDUTY_API_TOKEN

# Check it's included in config
api_token: ${PAGERDUTY_API_TOKEN}
```

## Platform Capabilities

| Feature | Supported |
|---------|-----------|
| Threading | ✅ Yes (incident notes) |
| Interactive Elements | ❌ No |
| File Attachments | ❌ No |
| Reactions | ❌ No |
| Rich Text | ✅ Yes (markdown in notes) |

## API Reference

### Add Note to Incident

```rust
platform.add_incident_note(
    incident_id: &str,
    note_content: &str,
    from_email: &str,
).await
```

### Update Incident Status

```rust
platform.update_incident_status(
    incident_id: &str,
    status: &str,  // "acknowledged" or "resolved"
    from_email: &str,
).await
```

## Related Documentation

- [Trigger Platform Architecture](/docs/architecture/triggers.md)
- [Agent Configuration](/docs/user/agents/configuration.md)
- [PagerDuty V3 Webhooks](https://developer.pagerduty.com/docs/88922dc5e1ad1-overview-v2-webhooks)
- [PagerDuty REST API](https://developer.pagerduty.com/api-reference/)

## Examples

See complete examples in `/docs/examples/triggers/`:
- `pagerduty-incident-response.yaml` - Full incident response workflow
- `pagerduty-auto-remediation.yaml` - Automated remediation
- `pagerduty-escalation.yaml` - Escalation handling
