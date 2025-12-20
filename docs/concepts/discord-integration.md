# Discord Integration

AOF integrates with Discord through the Interactions API for community and team-based AI agent interactions. Perfect for developer communities, gaming studios, open source projects, and teams who prefer Discord for collaboration.

## Why Discord?

- **200M+ monthly active users** - Popular with developers and gaming communities
- **Free to use** - No per-message costs unlike some enterprise platforms
- **Slash commands** - Native command system with autocomplete
- **Rich embeds** - Beautiful formatted responses
- **Components** - Buttons, select menus, and modals
- **Threads** - Organized conversation management

## How It Works

```
User sends command → Discord API → AOF Webhook
                                        ↓
                                   Verify Signature
                                        ↓
                                   Parse Interaction
                                        ↓
                                   Execute Agent
                                        ↓
User receives response ← Discord API ← Format Response
```

AOF receives interactions via Discord webhooks, verifies Ed25519 signatures, routes them to agents, and sends responses back through the Discord API.

## Key Features

### Slash Commands

Discord's native command system with autocomplete:

| Command | Description | Example |
|---------|-------------|---------|
| `/agent` | Manage AOF agents | `/agent action:run agent_id:k8s-ops` |
| `/task` | Create and manage tasks | `/task action:create description:check pods` |
| `/fleet` | Manage agent fleets | `/fleet action:status` |
| `/flow` | Execute workflows | `/flow workflow:incident-response` |

### Message Components

Interactive elements in responses:

| Type | Description | Use Case |
|------|-------------|----------|
| **Buttons** | Clickable action buttons | Quick actions, confirmations |
| **Select Menus** | Dropdown selections | Environment, namespace selection |
| **Modals** | Form popups | Input parameters, feedback |

### Rich Embeds

Beautifully formatted responses with:
- Title and description
- Color-coded status bars
- Field layouts (inline and stacked)
- Thumbnails and images
- Footers with timestamps

## Architecture

### Platform Adapter

The Discord platform adapter implements `TriggerPlatform`:

```
DiscordPlatform
├── parse_message()      # Parse interaction payloads
├── send_response()      # Send embeds/components
├── verify_signature()   # Ed25519 verification
├── register_commands()  # Register slash commands
└── create_*_command()   # Command definitions
```

### Interaction Flow

1. **Interaction Received** - Discord sends interaction to webhook
2. **Signature Verification** - Ed25519 signature check
3. **Parse Interaction** - Extract command/component data
4. **Execute Agent** - Route to appropriate agent
5. **Send Response** - Reply via Discord API

### Interaction Types

| Type | Value | Description |
|------|-------|-------------|
| PING | 1 | Webhook verification |
| APPLICATION_COMMAND | 2 | Slash command invocation |
| MESSAGE_COMPONENT | 3 | Button/select interaction |
| MODAL_SUBMIT | 5 | Modal form submission |

## Use Cases

### Developer Community Bot

```
👤 User: /agent action:run agent_id:code-review

🤖 Bot: 📋 Code Review Agent Started

         Status: ✅ Running
         Agent: code-review
         Task: Analyzing repository...

         [View Progress] [Stop Agent] [View Logs]
```

### DevOps Operations

```
👤 User: /fleet action:status

🤖 Bot: 📊 Fleet Status

         ┌────────────────────────────────┐
         │ Active Fleets: 3               │
         │ Total Agents: 12               │
         │ Tasks Running: 5               │
         └────────────────────────────────┘

         • rca-fleet: 4 agents (analyzing)
         • deploy-fleet: 3 agents (idle)
         • monitor-fleet: 5 agents (watching)

         [Scale Up] [View Details] [Refresh]
```

### Gaming/Community Management

Perfect for:
- Moderation automation
- Server statistics
- Event coordination
- Bot management

## Comparison with Slack

| Feature | Discord | Slack |
|---------|---------|-------|
| User Base | 200M+ (community-focused) | 54M+ (enterprise) |
| Pricing | Free | Free tier + paid |
| Commands | Slash commands | Slash commands |
| Interactive | Buttons, Selects, Modals | Block Kit |
| Threading | Native threads | Native threads |
| Reactions | Emoji reactions | Emoji reactions |
| Signature | Ed25519 | HMAC-SHA256 |
| Bots | Invite-based | Workspace install |

## Security

- **Ed25519 signature verification** - All interactions cryptographically verified
- **Role-based access** - Restrict commands to specific roles
- **Guild restrictions** - Limit to specific servers
- **Rate limiting** - Built-in Discord rate limits

## Getting Started

1. **Discord Developer Portal** - Create app at [discord.com/developers](https://discord.com/developers)
2. **Create Bot** - Add bot to your application
3. **Get Credentials** - Note Application ID, Bot Token, Public Key
4. **Configure AOF** - Add Discord platform to daemon config
5. **Set Webhook** - Configure interactions endpoint URL
6. **Register Commands** - Run command registration

See the [Discord Quickstart Guide](../guides/quickstart-discord.md) for step-by-step setup.

## Next Steps

- [Discord Quickstart](../guides/quickstart-discord.md) - 10-minute setup guide
- [Discord Tutorial](../tutorials/discord-ops-bot.md) - Build a complete ops bot
- [Discord Reference](../reference/discord-integration.md) - Full API reference
- [Slack Tutorial](../tutorials/slack-bot.md) - Enterprise alternative
