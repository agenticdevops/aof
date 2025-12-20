# Teams Integration Reference

Complete reference for Microsoft Teams Bot Framework integration in AOF.

## Configuration

### Trigger Specification

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: teams-ops
  labels:
    platform: teams
spec:
  type: Teams
  config:
    # Bot Framework credentials (required)
    app_id: ${TEAMS_APP_ID}
    app_password: ${TEAMS_APP_PASSWORD}

    # Tenant restrictions (optional)
    allowed_tenants:
      - "your-tenant-id"

    # Channel restrictions (optional)
    allowed_channels:
      - "19:channel-id@thread.tacv2"

    # User restrictions for approvals (optional)
    approval_allowed_users:
      - "user@company.com"

  # Slash commands
  commands:
    /help:
      agent: devops
      description: "Show available commands"

    /status:
      agent: k8s-ops
      description: "Check cluster status"

    /deploy:
      agent: deployer
      description: "Deploy application"
      requires_approval: true

  # Default for non-command messages
  default_agent: devops
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TEAMS_APP_ID` | Yes | Bot Framework App ID |
| `TEAMS_APP_PASSWORD` | Yes | Bot Framework App Secret |
| `TEAMS_TENANT_ID` | No | Restrict to specific tenant |

### Daemon Configuration

```yaml
# daemon.yaml
platforms:
  teams:
    enabled: true
    webhook_path: /webhook/teams

    # Bot Framework settings
    app_id: ${TEAMS_APP_ID}
    app_password: ${TEAMS_APP_PASSWORD}

    # Security settings
    verify_tokens: true
    allowed_tenants:
      - ${TEAMS_TENANT_ID}
```

## Bot Framework Activities

### Activity Types

| Type | Description | Handler |
|------|-------------|---------|
| `message` | Text message from user | Route to agent |
| `invoke` | Adaptive Card action | Handle submission |
| `conversationUpdate` | Member added/removed | Welcome message |
| `messageReaction` | Reaction added/removed | Approval workflow |

### Message Activity Structure

```json
{
  "type": "message",
  "id": "activity-id",
  "timestamp": "2024-01-15T10:30:00Z",
  "serviceUrl": "https://smba.trafficmanager.net/...",
  "channelId": "msteams",
  "from": {
    "id": "29:user-id",
    "name": "John Doe",
    "aadObjectId": "azure-ad-object-id"
  },
  "conversation": {
    "id": "19:channel-id@thread.tacv2",
    "conversationType": "channel",
    "tenantId": "tenant-id"
  },
  "recipient": {
    "id": "28:bot-id",
    "name": "OpsBot"
  },
  "text": "check pod status",
  "textFormat": "plain",
  "channelData": {
    "teamsChannelId": "19:channel-id@thread.tacv2",
    "teamsTeamId": "team-id",
    "tenant": {
      "id": "tenant-id"
    }
  }
}
```

### Invoke Activity (Adaptive Card Action)

```json
{
  "type": "invoke",
  "name": "adaptiveCard/action",
  "value": {
    "action": {
      "type": "Action.Submit",
      "id": "approve_deployment",
      "data": {
        "action": "approve",
        "deployment_id": "deploy-123"
      }
    }
  }
}
```

## Adaptive Cards

### Basic Response Card

```json
{
  "type": "AdaptiveCard",
  "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
  "version": "1.4",
  "body": [
    {
      "type": "TextBlock",
      "text": "Pod Status",
      "weight": "Bolder",
      "size": "Medium"
    },
    {
      "type": "FactSet",
      "facts": [
        { "title": "api-server", "value": "✅ Running" },
        { "title": "database", "value": "✅ Running" },
        { "title": "worker", "value": "⚠️ Degraded" }
      ]
    }
  ],
  "actions": [
    {
      "type": "Action.Submit",
      "title": "View Logs",
      "data": { "action": "view_logs", "pod": "worker" }
    },
    {
      "type": "Action.Submit",
      "title": "Restart",
      "data": { "action": "restart", "pod": "worker" }
    }
  ]
}
```

### Approval Card

```json
{
  "type": "AdaptiveCard",
  "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
  "version": "1.4",
  "body": [
    {
      "type": "TextBlock",
      "text": "🚀 Deployment Approval Required",
      "weight": "Bolder",
      "size": "Large"
    },
    {
      "type": "FactSet",
      "facts": [
        { "title": "Application", "value": "api-server" },
        { "title": "Version", "value": "v2.1.0 → v2.2.0" },
        { "title": "Environment", "value": "production" },
        { "title": "Requested By", "value": "john.doe@company.com" }
      ]
    },
    {
      "type": "TextBlock",
      "text": "Changes: Bug fixes and performance improvements",
      "wrap": true
    }
  ],
  "actions": [
    {
      "type": "Action.Submit",
      "title": "✅ Approve",
      "style": "positive",
      "data": { "action": "approve", "deployment_id": "deploy-123" }
    },
    {
      "type": "Action.Submit",
      "title": "❌ Reject",
      "style": "destructive",
      "data": { "action": "reject", "deployment_id": "deploy-123" }
    },
    {
      "type": "Action.ShowCard",
      "title": "💬 Request Changes",
      "card": {
        "type": "AdaptiveCard",
        "body": [
          {
            "type": "Input.Text",
            "id": "feedback",
            "placeholder": "Enter feedback...",
            "isMultiline": true
          }
        ],
        "actions": [
          {
            "type": "Action.Submit",
            "title": "Submit",
            "data": { "action": "request_changes", "deployment_id": "deploy-123" }
          }
        ]
      }
    }
  ]
}
```

### Status Dashboard Card

```json
{
  "type": "AdaptiveCard",
  "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
  "version": "1.4",
  "body": [
    {
      "type": "TextBlock",
      "text": "📊 Cluster Dashboard",
      "weight": "Bolder",
      "size": "Large"
    },
    {
      "type": "ColumnSet",
      "columns": [
        {
          "type": "Column",
          "width": "stretch",
          "items": [
            { "type": "TextBlock", "text": "Nodes", "weight": "Bolder" },
            { "type": "TextBlock", "text": "5/5 Ready", "color": "Good" }
          ]
        },
        {
          "type": "Column",
          "width": "stretch",
          "items": [
            { "type": "TextBlock", "text": "Pods", "weight": "Bolder" },
            { "type": "TextBlock", "text": "42/45 Running", "color": "Warning" }
          ]
        },
        {
          "type": "Column",
          "width": "stretch",
          "items": [
            { "type": "TextBlock", "text": "CPU", "weight": "Bolder" },
            { "type": "TextBlock", "text": "67%", "color": "Attention" }
          ]
        }
      ]
    }
  ],
  "actions": [
    { "type": "Action.Submit", "title": "Refresh", "data": { "action": "refresh" } },
    { "type": "Action.Submit", "title": "Details", "data": { "action": "details" } }
  ]
}
```

## Authentication

### JWT Token Verification

Teams uses JWT tokens for authentication:

```rust
// Token structure
{
  "iss": "https://api.botframework.com",
  "serviceUrl": "https://smba.trafficmanager.net/...",
  "aud": "your-app-id",
  "exp": 1705312200,
  "nbf": 1705308600
}
```

### Verification Steps

1. Extract `Authorization: Bearer <token>` header
2. Fetch OpenID metadata from Bot Framework
3. Validate JWT signature using public keys
4. Verify `aud` matches your App ID
5. Check `exp` and `nbf` for token validity

### Service URL Validation

Always verify the `serviceUrl` in activities:
- Must start with `https://`
- Must match known Bot Framework endpoints
- Store per-conversation for reply routing

## API Endpoints

### Send Message

```
POST {serviceUrl}/v3/conversations/{conversationId}/activities
Authorization: Bearer {bot_token}
Content-Type: application/json

{
  "type": "message",
  "text": "Response text",
  "attachments": [
    {
      "contentType": "application/vnd.microsoft.card.adaptive",
      "content": { /* Adaptive Card JSON */ }
    }
  ]
}
```

### Reply to Activity

```
POST {serviceUrl}/v3/conversations/{conversationId}/activities/{activityId}
Authorization: Bearer {bot_token}
Content-Type: application/json

{
  "type": "message",
  "text": "Reply text"
}
```

### Update Message

```
PUT {serviceUrl}/v3/conversations/{conversationId}/activities/{activityId}
Authorization: Bearer {bot_token}
Content-Type: application/json

{
  "type": "message",
  "text": "Updated text",
  "attachments": [...]
}
```

### Get Bot Token

```
POST https://login.microsoftonline.com/botframework.com/oauth2/v2.0/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id={app_id}
&client_secret={app_password}
&scope=https://api.botframework.com/.default
```

## Slash Commands

### Command Format

Users can invoke commands with:
- `/command` - In 1:1 chat
- `@BotName /command` - In channels
- `@BotName command` - Natural mention

### Command Configuration

```yaml
commands:
  /status:
    agent: k8s-ops
    description: "Check cluster status"

  /deploy:
    agent: deployer
    description: "Deploy application"
    requires_approval: true
    approval_channel: "19:approvals-channel@thread.tacv2"

  /logs:
    agent: k8s-ops
    description: "View pod logs"
    parameters:
      - name: pod
        required: true
        description: "Pod name"
      - name: lines
        required: false
        default: "100"
```

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| 401 Unauthorized | Invalid/expired token | Refresh bot token |
| 403 Forbidden | Tenant not allowed | Check allowed_tenants |
| 404 Not Found | Invalid conversation | Verify conversation ID |
| 429 Too Many Requests | Rate limited | Implement backoff |
| 502 Bad Gateway | Service URL issue | Retry with exponential backoff |

### Error Response Format

```json
{
  "error": {
    "code": "BadArgument",
    "message": "Conversation not found"
  }
}
```

## Rate Limits

| Scope | Limit | Window |
|-------|-------|--------|
| Per conversation | 1 message/second | Rolling |
| Per bot | 50 messages/second | Rolling |
| Adaptive Card size | 28 KB | Per message |

## Proactive Messaging

### Store Conversation Reference

```rust
// On first message, store reference
let reference = ConversationReference {
    service_url: activity.service_url.clone(),
    conversation_id: activity.conversation.id.clone(),
    user_id: activity.from.id.clone(),
    bot_id: activity.recipient.id.clone(),
};
// Store in memory/database for later use
```

### Send Proactive Message

```rust
// Later, send proactive message
POST {stored_service_url}/v3/conversations/{conversation_id}/activities
Authorization: Bearer {fresh_bot_token}

{
  "type": "message",
  "text": "Alert: Pod api-server restarted"
}
```

## Security Best Practices

1. **Validate all tokens** - Never skip JWT verification
2. **Restrict tenants** - Use allowed_tenants for enterprise
3. **Allowlist channels** - Control which channels can interact
4. **Audit logging** - Log all bot interactions
5. **Secure secrets** - Use Azure Key Vault for credentials
6. **Rate limit responses** - Prevent bot abuse
7. **Validate service URLs** - Only reply to known endpoints

## Testing

### Bot Framework Emulator

Test locally with [Bot Framework Emulator](https://github.com/Microsoft/BotFramework-Emulator):

```bash
# Start AOF daemon
aofctl daemon start

# Configure emulator
# Endpoint: http://localhost:8080/webhook/teams
# App ID: your-app-id
# App Password: your-app-password
```

### Ngrok for Development

```bash
# Expose local server
ngrok http 8080

# Update Azure Bot Service messaging endpoint
# https://xxx.ngrok.io/webhook/teams
```

## Troubleshooting

### Bot Not Responding

1. Check Azure Bot Service health
2. Verify messaging endpoint URL
3. Confirm App ID/Password match
4. Check tenant restrictions
5. Review AOF daemon logs

### Adaptive Cards Not Rendering

1. Validate card JSON with [Adaptive Cards Designer](https://adaptivecards.io/designer/)
2. Check Teams version supports card features
3. Verify card size < 28KB
4. Test in Bot Framework Emulator first

### Token Errors

1. Verify App ID matches Azure registration
2. Check App Password hasn't expired
3. Confirm token endpoint is accessible
4. Review clock synchronization (JWT exp/nbf)

## See Also

- [Teams Concepts](../concepts/teams-integration.md)
- [Teams Quickstart](../guides/quickstart-teams.md)
- [Teams Tutorial](../tutorials/teams-ops-bot.md)
- [Trigger Specification](./trigger-spec.md)
