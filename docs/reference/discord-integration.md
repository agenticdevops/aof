# Discord Integration Reference

Complete reference for Discord Interactions API integration in AOF.

## Configuration

### Trigger Specification

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-ops
  labels:
    platform: discord
spec:
  type: Discord
  config:
    # Bot credentials (required)
    bot_token: ${DISCORD_BOT_TOKEN}
    application_id: ${DISCORD_APPLICATION_ID}
    public_key: ${DISCORD_PUBLIC_KEY}

    # Guild restrictions (optional)
    guild_ids:
      - "123456789012345678"

    # Role restrictions (optional)
    allowed_roles:
      - "987654321098765432"  # Admin role
      - "876543210987654321"  # DevOps role

  # Command definitions
  commands:
    /status:
      agent: devops
      description: "Check system status"

    /deploy:
      agent: deployer
      description: "Deploy application"

    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis"

  # Default for unmatched commands
  default_agent: devops
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DISCORD_BOT_TOKEN` | Yes | Bot token from Developer Portal |
| `DISCORD_APPLICATION_ID` | Yes | Application ID |
| `DISCORD_PUBLIC_KEY` | Yes | Ed25519 public key for verification |

### Daemon Configuration

```yaml
# daemon.yaml
platforms:
  discord:
    enabled: true
    webhook_path: /webhook/discord

    # Bot credentials
    bot_token: ${DISCORD_BOT_TOKEN}
    application_id: ${DISCORD_APPLICATION_ID}
    public_key: ${DISCORD_PUBLIC_KEY}

    # Optional restrictions
    guild_ids:
      - ${DISCORD_GUILD_ID}
```

## Interaction Types

### Slash Commands (Type 2)

```json
{
  "id": "interaction-id",
  "type": 2,
  "application_id": "app-id",
  "token": "interaction-token",
  "data": {
    "id": "command-id",
    "name": "agent",
    "options": [
      { "name": "action", "value": "run" },
      { "name": "agent_id", "value": "k8s-ops" }
    ]
  },
  "guild_id": "guild-id",
  "channel_id": "channel-id",
  "member": {
    "user": {
      "id": "user-id",
      "username": "developer",
      "discriminator": "0001"
    },
    "roles": ["role-id-1", "role-id-2"]
  }
}
```

### Message Components (Type 3)

```json
{
  "id": "interaction-id",
  "type": 3,
  "token": "interaction-token",
  "data": {
    "custom_id": "approve_deployment",
    "component_type": 2
  },
  "message": {
    "id": "original-message-id",
    "content": "..."
  },
  "member": {
    "user": { "id": "user-id", "username": "admin" }
  }
}
```

### Modal Submit (Type 5)

```json
{
  "id": "interaction-id",
  "type": 5,
  "token": "interaction-token",
  "data": {
    "custom_id": "feedback_modal",
    "components": [
      {
        "type": 1,
        "components": [
          {
            "type": 4,
            "custom_id": "feedback_text",
            "value": "User's input text"
          }
        ]
      }
    ]
  }
}
```

## Slash Commands

### Command Structure

```json
{
  "name": "agent",
  "description": "Manage AOF agents",
  "options": [
    {
      "type": 3,
      "name": "action",
      "description": "Action to perform",
      "required": true,
      "choices": [
        { "name": "run", "value": "run" },
        { "name": "status", "value": "status" },
        { "name": "stop", "value": "stop" }
      ]
    },
    {
      "type": 3,
      "name": "agent_id",
      "description": "Agent ID",
      "required": true
    }
  ]
}
```

### Option Types

| Type | Name | Description |
|------|------|-------------|
| 1 | SUB_COMMAND | Subcommand |
| 2 | SUB_COMMAND_GROUP | Subcommand group |
| 3 | STRING | String input |
| 4 | INTEGER | Integer input |
| 5 | BOOLEAN | Boolean input |
| 6 | USER | User mention |
| 7 | CHANNEL | Channel selection |
| 8 | ROLE | Role selection |
| 10 | NUMBER | Decimal number |
| 11 | ATTACHMENT | File attachment |

### Built-in Commands

AOF registers these commands automatically:

| Command | Options | Description |
|---------|---------|-------------|
| `/agent` | action, agent_id | Manage agents |
| `/task` | action, description | Manage tasks |
| `/fleet` | action | Manage fleets |
| `/flow` | workflow | Execute workflows |

### Registering Commands

```bash
# Register commands globally
aofctl discord register-commands

# Register to specific guild (faster updates)
aofctl discord register-commands --guild-id 123456789012345678
```

## Response Formatting

### Basic Embed

```json
{
  "type": 4,
  "data": {
    "embeds": [
      {
        "title": "📊 System Status",
        "description": "All systems operational",
        "color": 5763719,
        "fields": [
          { "name": "Nodes", "value": "✅ 3/3 Ready", "inline": true },
          { "name": "Pods", "value": "✅ 45/45 Running", "inline": true },
          { "name": "Services", "value": "✅ 12/12 Active", "inline": true }
        ],
        "footer": { "text": "Last updated" },
        "timestamp": "2024-01-15T10:30:00.000Z"
      }
    ]
  }
}
```

### With Components

```json
{
  "type": 4,
  "data": {
    "embeds": [{ "title": "Status", "description": "..." }],
    "components": [
      {
        "type": 1,
        "components": [
          {
            "type": 2,
            "style": 1,
            "label": "Refresh",
            "custom_id": "refresh_status"
          },
          {
            "type": 2,
            "style": 3,
            "label": "View Logs",
            "custom_id": "view_logs"
          },
          {
            "type": 2,
            "style": 4,
            "label": "Restart",
            "custom_id": "restart_service"
          }
        ]
      }
    ]
  }
}
```

### Component Types

| Type | Name | Description |
|------|------|-------------|
| 1 | Action Row | Container for components |
| 2 | Button | Clickable button |
| 3 | String Select | Dropdown menu |
| 4 | Text Input | Text input (modals only) |
| 5 | User Select | User selection menu |
| 6 | Role Select | Role selection menu |
| 7 | Mentionable Select | User/role selection |
| 8 | Channel Select | Channel selection |

### Button Styles

| Style | Name | Color | Use Case |
|-------|------|-------|----------|
| 1 | Primary | Blue | Main actions |
| 2 | Secondary | Gray | Secondary actions |
| 3 | Success | Green | Confirmations |
| 4 | Danger | Red | Destructive actions |
| 5 | Link | Gray | External links |

### Modal Response

```json
{
  "type": 9,
  "data": {
    "custom_id": "feedback_modal",
    "title": "Provide Feedback",
    "components": [
      {
        "type": 1,
        "components": [
          {
            "type": 4,
            "custom_id": "feedback_text",
            "label": "Your Feedback",
            "style": 2,
            "placeholder": "Enter feedback...",
            "required": true,
            "max_length": 1000
          }
        ]
      }
    ]
  }
}
```

## Signature Verification

### Ed25519 Verification

Discord uses Ed25519 for signature verification:

```rust
// Headers required
X-Signature-Ed25519: <hex_signature>
X-Signature-Timestamp: <timestamp>

// Message to verify
message = timestamp + request_body

// Verification
ed25519::verify(public_key, message, signature)
```

### Verification Implementation

```rust
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

fn verify_signature(
    public_key: &VerifyingKey,
    timestamp: &str,
    body: &[u8],
    signature: &[u8; 64]
) -> bool {
    let message = format!("{}{}", timestamp, String::from_utf8_lossy(body));
    let sig = Signature::from_bytes(signature);
    public_key.verify(message.as_bytes(), &sig).is_ok()
}
```

## API Endpoints

### Respond to Interaction

```
POST https://discord.com/api/v10/interactions/{interaction_id}/{interaction_token}/callback

{
  "type": 4,
  "data": {
    "content": "Response text",
    "embeds": [...],
    "components": [...]
  }
}
```

### Response Types

| Type | Name | Description |
|------|------|-------------|
| 1 | PONG | ACK ping |
| 4 | CHANNEL_MESSAGE | Reply with message |
| 5 | DEFERRED_CHANNEL_MESSAGE | ACK, send later |
| 6 | DEFERRED_UPDATE_MESSAGE | ACK component, update later |
| 7 | UPDATE_MESSAGE | Update original message |
| 9 | MODAL | Show modal dialog |

### Edit Original Response

```
PATCH https://discord.com/api/v10/webhooks/{application_id}/{interaction_token}/messages/@original

{
  "content": "Updated content",
  "embeds": [...]
}
```

### Send Follow-up Message

```
POST https://discord.com/api/v10/webhooks/{application_id}/{interaction_token}

{
  "content": "Follow-up message"
}
```

### Register Global Commands

```
PUT https://discord.com/api/v10/applications/{application_id}/commands

[
  { "name": "agent", "description": "...", "options": [...] },
  { "name": "task", "description": "...", "options": [...] }
]
```

### Register Guild Commands

```
PUT https://discord.com/api/v10/applications/{application_id}/guilds/{guild_id}/commands

[...]
```

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| Interaction response | 3 seconds | Must respond |
| Deferred response | 15 minutes | Follow-up window |
| Global commands | 200/day | Per application |
| Guild commands | 200/day | Per guild |
| Message send | 5/5s | Per channel |

## Error Handling

### Common Errors

| Code | Description | Solution |
|------|-------------|----------|
| 10003 | Unknown channel | Verify channel ID |
| 10008 | Unknown message | Message deleted |
| 10062 | Unknown interaction | Timeout expired |
| 40001 | Unauthorized | Check bot token |
| 40060 | Interaction acknowledged | Already responded |
| 50001 | Missing access | Check permissions |
| 50035 | Invalid form body | Check request format |

### Error Response Format

```json
{
  "code": 50035,
  "message": "Invalid Form Body",
  "errors": {
    "data": {
      "embeds": {
        "0": {
          "title": {
            "_errors": [
              { "code": "BASE_TYPE_MAX_LENGTH", "message": "..." }
            ]
          }
        }
      }
    }
  }
}
```

## Limits

| Resource | Limit |
|----------|-------|
| Embed title | 256 characters |
| Embed description | 4096 characters |
| Embed fields | 25 fields |
| Field name | 256 characters |
| Field value | 1024 characters |
| Total embed | 6000 characters |
| Components per row | 5 |
| Action rows | 5 |
| Select options | 25 |
| Button label | 80 characters |

## Security Best Practices

1. **Always verify signatures** - Never skip Ed25519 verification
2. **Validate user roles** - Check allowed_roles before execution
3. **Use ephemeral messages** - For sensitive information
4. **Rate limit commands** - Prevent abuse
5. **Audit logging** - Log all command invocations
6. **Secure tokens** - Never expose in logs or responses

## Testing

### Discord Developer Portal

Test in the Developer Portal:
1. Go to your application
2. Navigate to "Bot" section
3. Use "Interactions Endpoint URL" testing

### Local Development

```bash
# Use ngrok for local testing
ngrok http 8080

# Update Interactions Endpoint URL
# https://xxx.ngrok.io/webhook/discord
```

### Interaction Verification Test

```bash
# Test PING verification
curl -X POST http://localhost:8080/webhook/discord \
  -H "Content-Type: application/json" \
  -H "X-Signature-Ed25519: <signature>" \
  -H "X-Signature-Timestamp: <timestamp>" \
  -d '{"type": 1}'
```

## Troubleshooting

### Bot Not Responding

1. Check webhook URL is correct
2. Verify signature verification passes
3. Confirm bot has required permissions
4. Check AOF daemon logs

### Commands Not Appearing

1. Wait 1 hour for global commands
2. Use guild commands for instant updates
3. Check command registration succeeded
4. Verify bot is in the guild

### Signature Verification Failing

1. Verify public key is correct
2. Check timestamp header is present
3. Ensure body hasn't been modified
4. Verify hex decoding is correct

## See Also

- [Discord Concepts](../concepts/discord-integration.md)
- [Discord Quickstart](../guides/quickstart-discord.md)
- [Discord Tutorial](../tutorials/discord-ops-bot.md)
- [Trigger Specification](./trigger-spec.md)
