# Webhook Triggers

Webhook triggers fire an agent when an HTTP POST request is sent to
`/webhooks/:trigger_id`. Three specializations exist:

| Type | YAML `type` | Signature Header | Source Field |
|------|-------------|-----------------|--------------|
| Generic webhook | `webhook` | `x-webhook-signature` (HMAC-SHA256) | `"webhook"` |
| GitHub | `github` | `x-hub-signature-256` (`sha256=…`) | `"github"` |
| Jira | `jira` | `x-hub-signature` (HMAC-SHA256) | `"jira"` |

## Trigger ID Format

Each trigger is assigned a `trigger_id` at agent load time:

```
{agent_name}-{type}-{index}
```

For example, an agent named `deploy-bot` with a single `webhook` trigger gets
the id `deploy-bot-webhook-0`. The HTTP endpoint is:

```
POST /webhooks/deploy-bot-webhook-0
```

## Generic Webhook

### Agent YAML

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: deploy-bot
  description: Triggers a deployment on any webhook POST
spec:
  model: anthropic/claude-3-haiku-20240307
  system_prompt: |
    You are a deployment bot. When triggered via webhook, read the
    TriggerEvent in your input. The payload.message field contains
    instructions. Execute the requested deployment.
  triggers:
    - type: webhook
      webhook_secret: ${DEPLOY_WEBHOOK_SECRET}
```

### Sending a Payload

```bash
curl -X POST http://localhost:7777/webhooks/deploy-bot-webhook-0 \
  -H "Content-Type: application/json" \
  -H "x-webhook-signature: <hmac-sha256-hex>" \
  -d '{"message": "deploy to production", "ref": "v1.2.3"}'
```

### HMAC Signature

When `webhook_secret` is configured, the sender must include an
`x-webhook-signature` header containing the HMAC-SHA256 hex digest
of the request body:

```python
import hmac, hashlib, json

secret = "your-secret"
body   = json.dumps({"message": "deploy"}, separators=(",", ":"))
sig    = hmac.new(secret.encode(), body.encode(), hashlib.sha256).hexdigest()
# sig = "abc123..."
```

If no secret is configured, any payload is accepted.

## GitHub Trigger

GitHub triggers filter by event type from the `x-github-event` header and
verify signatures via `x-hub-signature-256`.

### Agent YAML

```yaml
spec:
  triggers:
    - type: github
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events:
        - pull_request
        - push
```

### GitHub Webhook Setup

1. Go to **Settings → Webhooks** in your GitHub repository
2. Set **Payload URL** to `https://your-gateway/webhooks/{agent}-github-0`
3. Set **Content type** to `application/json`
4. Set **Secret** to match `GITHUB_WEBHOOK_SECRET`
5. Select the events to subscribe to (must match your `events:` list)

### TriggerEvent from GitHub

```json
{
  "source": "github",
  "payload": {
    "action": "opened",
    "number": 42,
    "pull_request": {
      "title": "Fix authentication bug",
      "html_url": "https://github.com/org/repo/pull/42",
      "body": "This PR fixes the OAuth token refresh logic."
    }
  },
  "context": {
    "github_event": "pull_request"
  },
  "fired_at": "2026-03-13T09:15:00Z",
  "trigger_id": "pr-reviewer-github-0"
}
```

### Event Filtering

Only events listed in `events:` are accepted; others are silently dropped
(HTTP 202 is returned but no run is started):

```yaml
events:
  - push           # branch pushes
  - pull_request   # PR opened/closed/merged
  - issues         # issue created/updated/closed
  - release        # release published
  - workflow_run   # GitHub Actions workflow completed
```

## Jira Trigger

Jira triggers filter by event type from the `webhookEvent` field in the body.

### Agent YAML

```yaml
spec:
  triggers:
    - type: jira
      webhook_secret: ${JIRA_WEBHOOK_SECRET}
      events:
        - jira:issue_created
        - jira:issue_updated
```

### Jira Webhook Setup

1. Go to **Jira Settings → System → WebHooks**
2. Set **URL** to `https://your-gateway/webhooks/{agent}-jira-0`
3. Select the events to subscribe to
4. Set the shared secret (optional but recommended)

### TriggerEvent from Jira

```json
{
  "source": "jira",
  "payload": {
    "webhookEvent": "jira:issue_created",
    "issue": {
      "id": "10001",
      "key": "PROJ-42",
      "fields": {
        "summary": "Build fails on CI after last merge",
        "priority": { "name": "High" },
        "status":   { "name": "Open" }
      }
    }
  },
  "context": {
    "jira_event": "jira:issue_created"
  },
  "fired_at": "2026-03-13T10:30:00Z",
  "trigger_id": "incident-bot-jira-0"
}
```

### Common Jira Event Types

| Event | When it fires |
|-------|---------------|
| `jira:issue_created` | A new issue is created |
| `jira:issue_updated` | An issue is updated (field change, comment, etc.) |
| `jira:issue_deleted` | An issue is deleted |
| `comment_created` | A comment is added to an issue |
| `comment_updated` | A comment is edited |

## Response Format

All webhook endpoints return **202 Accepted** when the event is queued:

```json
{
  "status": "queued",
  "trigger_id": "deploy-bot-webhook-0",
  "agent": "deploy-bot"
}
```

The agent run starts asynchronously. The HTTP response does not wait for the
agent to complete.

Returns **404 Not Found** if no agent is registered for the given `trigger_id`:

```json
{
  "error": "No agent registered for trigger_id 'unknown-webhook-0'"
}
```

## Security Recommendations

- Always configure a `webhook_secret` in production environments
- Rotate secrets regularly via environment variable updates
- For GitHub, use the `sha256=…` prefix format which is automatically handled
- Use HTTPS for all gateway endpoints in production
- Restrict the gateway's network exposure to known source IPs where possible

## See Also

- [Triggers Overview](triggers.md) — all trigger types and the TriggerEvent model
- [Cron Triggers](cron-triggers.md) — scheduled agent execution
- [Triggers Quickstart](../guides/triggers-quickstart.md) — step-by-step setup guide
