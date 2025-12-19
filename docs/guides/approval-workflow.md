# Human-in-the-Loop Approval Workflow

AOF supports human-in-the-loop approval for destructive operations, ensuring that sensitive commands require explicit user approval before execution.

## Overview

When an AI agent determines that a requested operation is potentially destructive (create, delete, scale down, etc.), it can request human approval before proceeding. This is implemented as a native feature of the trigger handler, with first-class support for Slack reactions.

## Platform Support

### Current Implementation

| Platform | Approval Method | Status |
|----------|----------------|--------|
| **Slack** | Reaction-based (✅/❌) | ✅ Fully implemented |
| **Discord** | Reaction-based | 🔄 Planned |
| **Telegram** | Inline keyboard buttons | 🔄 Planned |
| **Microsoft Teams** | Adaptive Cards | 🔄 Planned |
| **WhatsApp** | Button replies | 🔄 Planned |

### Architecture

The approval workflow is designed to be **platform-agnostic at the handler level**:

1. **Agent Output Parsing**: The `TriggerHandler` parses `requires_approval: true` from agent responses - this is platform-independent
2. **PendingApproval Storage**: Approvals are stored in a shared `DashMap` accessible by all platforms
3. **Platform-Specific UI**: Each platform implements its own approval UI (reactions, buttons, etc.)
4. **Approval Processing**: The handler processes approval events from any platform

To add approval support for a new platform:
1. Implement the approval UI in the platform's `TriggerPlatform` implementation
2. Handle approval events (reactions, button clicks, etc.)
3. Call the shared `handle_approval` method on `TriggerHandler`

### Platform-Specific Approval Methods (Planned)

Each platform uses its native interaction mechanism for approvals:

| Platform | Mechanism | API | Notes |
|----------|-----------|-----|-------|
| **Slack** | Reactions | Events API | ✅ Implemented - `reaction_added` events |
| **Discord** | Reactions | Gateway API | Similar to Slack, use emoji reactions |
| **Telegram** | Inline Keyboards | Bot API | Interactive buttons below message |
| **Microsoft Teams** | Adaptive Cards | Bot Framework | Rich card with action buttons |
| **WhatsApp** | Interactive Buttons | Cloud API | Up to 3 buttons per message |

#### WhatsApp Business API (Planned)

WhatsApp Business API supports [Interactive Messages](https://developers.facebook.com/docs/whatsapp/cloud-api/messages/interactive-messages) with button replies - perfect for approval workflows:

```json
{
  "type": "interactive",
  "interactive": {
    "type": "button",
    "header": { "type": "text", "text": "⚠️ Approval Required" },
    "body": { "text": "kubectl create deployment nginx --image=nginx" },
    "action": {
      "buttons": [
        { "type": "reply", "reply": { "id": "approve", "title": "✅ Approve" } },
        { "type": "reply", "reply": { "id": "deny", "title": "❌ Deny" } }
      ]
    }
  }
}
```

**Requirements:**
- WhatsApp Business Account
- Cloud API access (Meta Developer Portal)
- Verified business phone number
- Webhook endpoint for button click callbacks

**Planned Config:**
```yaml
platforms:
  whatsapp:
    enabled: true
    access_token_env: WHATSAPP_ACCESS_TOKEN
    verify_token_env: WHATSAPP_VERIFY_TOKEN
    phone_number_id_env: WHATSAPP_PHONE_NUMBER_ID

    # Approval whitelist (phone numbers)
    approval_allowed_users:
      - "+1234567890"
      - "+0987654321"
```

## How It Works

### 1. Agent Returns Approval Request

When the agent's response includes approval-related fields, the system intercepts and triggers the approval flow:

```
The agent's response should include:
- requires_approval: true
- command: "kubectl create deployment nginx --image=nginx"
```

### 2. Approval Message is Sent

The system posts an approval request message to Slack with reaction buttons:

```
⚠️ This action requires approval
`kubectl create deployment nginx --image=nginx`

React with ✅ to approve or ❌ to deny.
```

The message automatically gets ✅ and ❌ reactions added for easy approval/denial.

### 3. User Reacts to Approve/Deny

- **✅ (white_check_mark)** - Approves and executes the command
- **❌ (x)** - Denies and cancels the operation

### 4. Feedback After Execution

After approval and execution, the result is posted back:

```
✅ Command completed successfully
```kubectl create deployment nginx --image=nginx```
deployment.apps/nginx created
```

## Agent Configuration

Configure your agent to request approval for destructive operations:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-assistant

spec:
  model: google:gemini-2.5-flash
  temperature: 0.3

  instructions: |
    You are a Kubernetes assistant.

    IMPORTANT: For destructive operations (create, delete, scale, apply, patch),
    you MUST return the following format:

    [Your explanation]

    requires_approval: true
    command: "kubectl ..."

    This will trigger a human approval flow before execution.

    For read-only operations (get, describe, logs), execute directly
    using your kubectl tool.

  tools:
    - kubectl
    - helm
    - shell
```

## Supported Reactions

The approval system recognizes these reactions:

### Approve Reactions
- `white_check_mark` (✅) - Primary approval
- `heavy_check_mark` (✔️)
- `+1` (👍)

### Deny Reactions
- `x` (❌) - Primary denial
- `no_entry` (⛔)
- `-1` (👎)

## Technical Implementation

The approval workflow is implemented in the trigger handler (`aof-triggers` crate):

### Key Components

1. **PendingApproval Storage** (`handler/mod.rs`)
   - Stores pending approvals in a concurrent `DashMap<String, PendingApproval>`
   - Key is the Slack message timestamp (for lookup on reaction)
   - Value contains command, user, channel, and metadata

2. **Output Parsing** (`parse_approval_output`)
   - Parses agent responses for `requires_approval: true` pattern
   - Extracts the command to execute

3. **Reaction Events** (`platforms/slack.rs`)
   - Handles `reaction_added` events from Slack
   - Maps reactions to approve/deny actions

4. **Command Execution** (`handle_reaction_event`)
   - Executes approved commands via shell
   - Sends result feedback to Slack
   - Removes pending approval after processing

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User Message                                                 │
│    @bot "create nginx deployment"                               │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. Agent Processes & Returns                                    │
│    requires_approval: true                                      │
│    command: "kubectl create deployment nginx ..."               │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. Handler Parses Output                                        │
│    - Detects requires_approval: true                            │
│    - Extracts command                                           │
│    - Posts approval message                                     │
│    - Adds ✅ ❌ reactions                                        │
│    - Stores PendingApproval                                     │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. User Reacts                                                  │
│    ✅ → Approve | ❌ → Deny                                      │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│ 5. Handler Processes Reaction                                   │
│    - Looks up PendingApproval by message_ts                     │
│    - If approved: Execute command, send result                  │
│    - If denied: Send denial message                             │
│    - Remove from pending approvals                              │
└─────────────────────────────────────────────────────────────────┘
```

## Slack App Configuration

To enable reaction-based approvals, your Slack app needs these additional event subscriptions:

1. Go to **Event Subscriptions** in your Slack app settings
2. Add to **Subscribe to bot events**:
   - `reaction_added` - Required for approval
   - `reaction_removed` - Optional

3. Add OAuth scope:
   - `reactions:write` - To add ✅ ❌ reactions

See [Slack Bot Tutorial](../tutorials/slack-bot.md) for complete configuration.

## Example Session

### User Request
```
@K8s Bot create an nginx deployment with 3 replicas
```

### Bot Response (Approval Request)
```
I'll create an nginx deployment with 3 replicas.

⚠️ This action requires approval
`kubectl create deployment nginx --image=nginx --replicas=3`

React with ✅ to approve or ❌ to deny.
```

### After User Approves (✅)
```
⚡ Executing approved command...
`kubectl create deployment nginx --image=nginx --replicas=3`
```

```
✅ Command completed successfully
```deployment.apps/nginx created```
Approved by: @user
```

### Or If User Denies (❌)
```
❌ Action denied by @user
`kubectl create deployment nginx --image=nginx --replicas=3`
```

## Role-Based Approval (RBAC)

AOF supports configuring which users are allowed to approve commands. The approval authorization system is designed to be **platform-agnostic**, allowing consistent RBAC policies across all platforms.

### Configuration Levels

AOF supports two levels of approval configuration:

1. **Global Configuration** (Platform-Agnostic) - Applies to all platforms
2. **Platform-Specific Configuration** - Overrides global for a specific platform

#### Global Configuration (Planned)

> **Status:** 🔄 Coming Soon - This feature is planned for a future release.

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-platform-bot

spec:
  # Global approval settings - applies to ALL platforms
  approval:
    # Users who can approve commands (platform-agnostic IDs)
    allowed_users:
      - email:admin@company.com      # Email-based (works across platforms)
      - email:teamlead@company.com
      - slack:U11111111              # Platform-specific ID with prefix
      - discord:123456789
      - teams:user@company.onmicrosoft.com

    # Approval timeout in minutes (default: 30)
    timeout_minutes: 30

    # Require approval for these command patterns
    require_for:
      - "kubectl delete *"
      - "kubectl scale * --replicas=0"
      - "helm uninstall *"

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

    discord:
      enabled: true
      bot_token_env: DISCORD_BOT_TOKEN

    teams:
      enabled: true
      app_id_env: TEAMS_APP_ID
      app_secret_env: TEAMS_APP_SECRET
```

#### Platform-Specific Configuration (Current Implementation)

For Slack-only deployments or when you need platform-specific overrides:

```yaml
platforms:
  slack:
    enabled: true
    bot_token_env: SLACK_BOT_TOKEN
    signing_secret_env: SLACK_SIGNING_SECRET

    # Platform-specific: Overrides global approval.allowed_users for Slack
    approval_allowed_users:
      - U11111111  # Admin 1 (Slack user ID)
      - U22222222  # Admin 2
      - U33333333  # Team Lead
```

> **Important:** After changing `approval_allowed_users`, you must restart the server for changes to take effect. Hot-reload is planned for a future release (see [GitHub Issue #22](https://github.com/agenticdevops/aof/issues/22)).

### Finding Your Slack User ID

1. In Slack, click on a user's profile
2. Click the "..." (More) button
3. Select "Copy member ID"
4. The ID looks like `U015VBH1GTZ`

### User ID Resolution

The approval system resolves user identities across platforms:

| ID Format | Example | Status |
|-----------|---------|--------|
| Raw ID | `U12345678` | ✅ Implemented (Slack) |
| `slack:U12345678` | Slack user ID | 🔄 Planned |
| `discord:123456789` | Discord user ID | 🔄 Planned |
| `teams:user@tenant.com` | Teams UPN | 🔄 Planned |
| `telegram:123456789` | Telegram user ID | 🔄 Planned |
| `whatsapp:+1234567890` | WhatsApp phone number | 🔄 Planned |
| `email:user@company.com` | Universal | 🔄 Planned (requires identity mapping) |

### Behavior

- **No whitelist configured**: Anyone can approve (default)
- **Global whitelist only**: Applies to all platforms (planned)
- **Platform-specific whitelist**: Overrides global for that platform
- **Unauthorized approval attempt**: User sees "⚠️ @user is not authorized to approve commands"

### Bot Self-Approval Prevention

The system automatically ignores reactions from the bot itself. When the bot adds ✅ and ❌ reactions to an approval message, these are filtered out and don't trigger approval.

**Automatic Detection (v0.1.12+):**
The bot's user ID is now **automatically detected** at startup using Slack's `auth.test` API. You no longer need to manually configure `bot_user_id` - it will be fetched from your bot token automatically.

You'll see this log message when the server starts:
```
INFO Auto-detected bot_user_id: U1234567890
```

This is implemented by:
1. Calling Slack's `auth.test` API at startup to get the bot's user ID
2. Storing the `bot_user_id` in SlackConfig
3. Filtering `reaction_added` events where `user == bot_user_id`

**Manual Override (Optional):**
If you need to override the auto-detected ID, you can still specify `bot_user_id` in your config:

```yaml
platforms:
  slack:
    enabled: true
    bot_user_id: U1234567890  # Optional: overrides auto-detection
```

## Security Considerations

1. **User Authorization**: Configure `approval_allowed_users` in production to restrict who can approve destructive commands.

2. **Bot Self-Approval Prevention**: The bot automatically ignores its own reactions to prevent self-approval.

3. **Command Validation**: Commands are executed via shell. Consider:
   - Allowlist of approved command patterns
   - Namespace restrictions for kubectl
   - Command sanitization

4. **Audit Trail**: All approvals are logged with:
   - User who requested
   - User who approved/denied
   - Command executed
   - Execution result

5. **Timeout**: Pending approvals should timeout after a period (not yet implemented). Consider adding expiration handling.

## Troubleshooting

### Reactions not triggering approval

1. Check Slack Event Subscriptions include `reaction_added`
2. Verify bot has `reactions:write` scope
3. Check server logs for reaction events

```bash
RUST_LOG=debug aofctl serve --config config.yaml
```

### Command not parsed correctly

The command extraction regex expects format:
```
command: "your command here"
```
or
```
command: your command here
```

### Pending approval not found

Approvals are keyed by message timestamp. If the bot restarts, pending approvals are lost. For production, consider persisting to Redis/database.

## Implementation Status

### What's Implemented Now

| Feature | Status | Notes |
|---------|--------|-------|
| Slack approval (reactions) | ✅ Complete | ✅/❌ reactions for approve/deny |
| Bot self-approval prevention | ✅ Complete | Bot ignores its own reactions |
| Platform-specific RBAC (`approval_allowed_users`) | ✅ Complete | Slack only for now |
| Conversation memory | ✅ Complete | Context maintained across messages |

### Coming Soon

| Feature | Status | Platform |
|---------|--------|----------|
| Global `spec.approval.allowed_users` | 🔄 Planned | All platforms |
| Config hot-reload (`aofctl serve --reload`) | 🔄 [Issue #22](https://github.com/agenticdevops/aof/issues/22) | All |
| Platform-prefixed IDs (`slack:U123`, `discord:123`) | 🔄 Planned | All platforms |
| Discord approval (reactions) | 🔄 Planned | Discord |
| Teams approval (Adaptive Cards) | 🔄 Planned | Microsoft Teams |
| Telegram approval (inline buttons) | 🔄 Planned | Telegram |
| WhatsApp approval (button replies) | 🔄 Planned | WhatsApp |
| Email-based identity mapping | 🔄 Planned | All platforms |

## Future Enhancements

- [ ] Config hot-reload without server restart ([Issue #22](https://github.com/agenticdevops/aof/issues/22))
- [ ] Approval timeout/expiration
- [ ] Multi-party approval (require 2+ approvals)
- [ ] Global platform-agnostic RBAC (spec.approval.allowed_users)
- [ ] Approval audit log persistence
- [ ] Approval undo/rollback
- [ ] Interactive button-based approval (in addition to reactions)
