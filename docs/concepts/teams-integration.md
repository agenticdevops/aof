# Microsoft Teams Integration

AOF integrates with Microsoft Teams through the Bot Framework for enterprise collaboration. Perfect for organizations using Microsoft 365 for communication and workflow automation.

## Why Teams?

- **320M+ daily active users** - Deep enterprise penetration
- **Microsoft 365 integration** - Seamless with Office, Azure, SharePoint
- **Enterprise security** - Azure AD, compliance, data residency
- **Adaptive Cards** - Rich interactive message components
- **Channels & chats** - Flexible conversation contexts

## How It Works

```
User sends message → Teams Bot Framework → AOF Webhook
                                                ↓
                                         Parse & Route
                                                ↓
                                         Execute Agent
                                                ↓
User receives reply ← Teams Bot Framework ← Format Response
```

AOF receives messages via Bot Framework webhooks, routes them to agents, and sends responses back through the Bot Framework REST API.

## Key Features

### Adaptive Cards

Teams uses Adaptive Cards for rich interactive messages:

| Element | Description | Use Case |
|---------|-------------|----------|
| **TextBlock** | Formatted text display | Status messages, logs |
| **Action.Submit** | Clickable buttons | Quick actions, approvals |
| **Action.ShowCard** | Expandable sections | Detailed info on demand |
| **Input.Text** | Text input fields | Parameters, search |
| **ColumnSet** | Multi-column layouts | Dashboards, metrics |
| **FactSet** | Key-value displays | Pod status, configs |

### Message Types Supported

- **Text messages** - Natural language queries
- **Adaptive Card submissions** - Button clicks, form inputs
- **@mentions** - Direct bot mentions in channels
- **1:1 chats** - Private bot conversations
- **Channel messages** - Team-wide interactions

### Security

- **Azure AD authentication** - Enterprise identity management
- **HMAC-SHA256 verification** - Bot Framework signature validation
- **Tenant restrictions** - Limit to specific Azure AD tenants
- **Channel allowlists** - Control which channels can interact

## Architecture

### Platform Adapter

The Teams platform adapter implements `TriggerPlatform`:

```
TeamsPlatform
├── parse_message()      # Parse Bot Framework activities
├── send_response()      # Send text/Adaptive Cards
├── verify_signature()   # HMAC-SHA256 verification
└── verify_webhook()     # Not needed (uses Bearer token)
```

### Webhook Flow

1. **Bot Registration** - Register bot in Azure Bot Service
2. **Activity Handler** - Receive activities via POST webhook
3. **Authentication** - Verify JWT token from Bot Framework
4. **Response** - Send replies via Bot Framework API

### Response Formatting

AOF automatically:
- Adds status indicators with appropriate styling
- Converts action buttons to Adaptive Card actions
- Formats code blocks with proper styling
- Handles message length limits (28KB for Adaptive Cards)
- Supports @mentions in responses

## Use Cases

### Enterprise DevOps

```
👤 User: @OpsBot deployment status for prod
🤖 Bot: 📊 Production Deployment Status

         ✅ api-gateway: Running (v2.1.3)
         ✅ user-service: Running (v1.8.0)
         ⚠️ payment-service: Degraded (v3.2.1)

         [View Details] [Check Logs] [Rollback]
```

### Approval Workflows

Teams excels at approval workflows:
- Adaptive Card with approve/reject buttons
- Action.Submit captures user decision
- Integration with Microsoft 365 approval policies
- Audit trail via Teams messages

### Incident Response

Perfect for enterprise incident management:
- Channel-based incident coordination
- @mention for on-call notifications
- Adaptive Cards for status dashboards
- Integration with Azure DevOps, ServiceNow

### Team Notifications

Teams channels work great for:
- Deployment notifications
- Alert broadcasts
- Status updates
- Sprint summaries

## Comparison with Slack

| Feature | Teams | Slack |
|---------|-------|-------|
| User Base | 320M+ enterprise | 54M+ mixed |
| Message Format | Adaptive Cards | Block Kit |
| Authentication | Azure AD | Slack OAuth |
| Interactive | Adaptive Card Actions | Block Kit Buttons |
| Threading | Supported | Supported |
| Files | Supported | Supported |
| Enterprise SSO | Native Azure AD | Enterprise Grid |
| Setup Complexity | Medium (Azure config) | Lower |

## Getting Started

1. **Azure Account** - Create at [portal.azure.com](https://portal.azure.com)
2. **Bot Registration** - Register bot in Azure Bot Service
3. **App Registration** - Create Azure AD app registration
4. **Configure AOF** - Add Teams platform to daemon config
5. **Install Bot** - Add bot to your Teams tenant

See the [Teams Quickstart Guide](../guides/quickstart-teams.md) for step-by-step setup.

## Next Steps

- [Teams Quickstart](../guides/quickstart-teams.md) - 15-minute setup guide
- [Teams Tutorial](../tutorials/teams-ops-bot.md) - Build a complete ops bot
- [Teams Reference](../reference/teams-integration.md) - Full API reference
- [Slack Tutorial](../tutorials/slack-bot.md) - Alternative enterprise platform
