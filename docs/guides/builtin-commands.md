# Built-in Command Handlers

Configure slash commands to use AOF's built-in interactive handlers instead of routing to LLM agents.

## Overview

By default, slash commands in trigger configurations route to LLM agents. However, for system commands like `/help`, `/agent`, and `/fleet`, you often want rich interactive menus with buttons rather than LLM-generated text responses.

The `agent: builtin` configuration tells AOF to use its built-in command handlers, which provide:
- Interactive inline keyboards (Telegram, Slack)
- Fleet and agent selection menus
- System information display
- Consistent, instant responses (no LLM latency)

## Quick Start

Add `agent: builtin` to any command that should use built-in handlers:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-bot
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}

  commands:
    /help:
      agent: builtin          # Built-in interactive menu
      description: "Show available commands"
    /agent:
      agent: builtin          # Agent selection buttons
      description: "Switch active agent"
    /fleet:
      agent: builtin          # Fleet selection buttons
      description: "Switch active fleet"
    /status:
      agent: devops           # Routes to LLM agent
      description: "Check system status"

  default_agent: devops
```

## Available Built-in Handlers

| Command | Description | Platform Support |
|---------|-------------|------------------|
| `/help` | Interactive help menu with command list and selection buttons | Telegram, Slack, Discord |
| `/agent` | Agent selection menu with inline keyboard | Telegram, Slack, Discord |
| `/fleet` | Fleet selection menu with inline keyboard | Telegram, Slack, Discord |
| `/info` | System information (version, loaded agents, platforms) | All platforms |
| `/flows` | List available flows with descriptions | All platforms |

## User Experience

### Telegram Example

```
User: /help

Bot: 📋 Available Commands

     /help - Show this menu
     /agent - Switch agent
     /fleet - Switch fleet
     /status - System status
     /kubectl - Kubernetes ops

     [🤖 Agents]  [👥 Fleets]
     [ℹ️ Info]    [📊 Flows]

User: *taps Agents button*

Bot: Select Agent
     Current: devops

     [devops]      [k8s-agent]
     [docker-ops]  [security]
```

### Slack Example

```
User: /help

Bot: 📋 AOF Help
     Select a category:

     • /status - Check system status
     • /kubectl - Kubernetes operations
     • /diagnose - Run diagnostics

     [Agents ▼] [Fleets ▼] [Info]
```

## Configuration Examples

### Basic Setup

```yaml
commands:
  /help:
    agent: builtin
    description: "Show help menu"
```

### Mixed Built-in and LLM Commands

```yaml
commands:
  # Built-in interactive handlers
  /help:
    agent: builtin
    description: "Show available commands"
  /agent:
    agent: builtin
    description: "Switch active agent"
  /fleet:
    agent: builtin
    description: "Switch active fleet"

  # LLM-powered commands
  /kubectl:
    agent: k8s-agent
    description: "Kubernetes operations"
  /diagnose:
    fleet: rca-fleet
    description: "Root cause analysis"
  /deploy:
    flow: deploy-flow
    description: "Deployment workflow"
```

### Platform-Specific Configuration

Built-in handlers adapt to platform capabilities:

```yaml
# Telegram - Full interactive buttons
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-interactive
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
  commands:
    /help:
      agent: builtin    # Shows inline keyboard buttons
    /agent:
      agent: builtin    # Agent selection with buttons
```

```yaml
# WhatsApp - Text-based menus (no inline buttons)
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: whatsapp-mobile
spec:
  type: WhatsApp
  config:
    bot_token: ${WHATSAPP_ACCESS_TOKEN}
    phone_number_id: ${WHATSAPP_PHONE_NUMBER_ID}
  commands:
    /help:
      agent: builtin    # Text menu with numbered options
```

## When to Use Builtin vs Agent

| Scenario | Use | Why |
|----------|-----|-----|
| Help menu | `agent: builtin` | Instant, consistent, interactive buttons |
| Agent/fleet switching | `agent: builtin` | Rich selection UI |
| System info | `agent: builtin` | Deterministic, no LLM needed |
| Natural language queries | `agent: <name>` | Requires LLM reasoning |
| Tool execution | `agent: <name>` | Needs MCP tools |
| Multi-step workflows | `fleet: <name>` or `flow: <name>` | Complex coordination |

## How It Works

When a message arrives:

1. **Command Parsing**: AOF extracts the command (e.g., `/help`)
2. **Binding Lookup**: Checks `commands` section for matching binding
3. **Builtin Check**: If `agent: builtin`, routes to built-in handler
4. **Handler Execution**: Built-in handler generates response with platform-appropriate UI
5. **Response**: Interactive menu sent to user

```
User: /help
    │
    ▼
┌─────────────────┐
│ Command Parser  │
└────────┬────────┘
         │ /help
         ▼
┌─────────────────┐
│ Binding Lookup  │ ← commands: { /help: { agent: builtin } }
└────────┬────────┘
         │ agent: builtin
         ▼
┌─────────────────┐
│ Built-in Handler│ ← HelpHandler
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Platform Adapter│ ← Telegram: inline keyboard
└────────┬────────┘   Slack: button blocks
         │            WhatsApp: text menu
         ▼
    Interactive Response
```

## Extending Built-in Handlers

Built-in handlers automatically discover:
- **Agents**: Loaded from `--agents-dir` or agent library
- **Fleets**: Loaded from `--fleets-dir`
- **Flows**: Loaded from `--flows-dir`

To add more agents to the selection menu, simply add more agent YAML files to your agents directory.

## Troubleshooting

### Buttons Not Appearing

**Problem**: `/help` shows text but no buttons

**Solutions**:
1. Verify platform supports interactive elements (Telegram, Slack, Discord do)
2. Check bot has required permissions for inline keyboards
3. For Telegram: Ensure bot is using webhook mode, not polling

### Command Routes to LLM Instead of Menu

**Problem**: `/help` gives LLM response instead of menu

**Solutions**:
1. Verify `agent: builtin` (not `agent: help` or agent name)
2. Check trigger is loaded (daemon logs show loaded triggers)
3. Ensure command binding exists in trigger YAML

### Menu Shows No Agents

**Problem**: Agent selection shows empty list

**Solutions**:
1. Check `--agents-dir` points to correct directory
2. Verify agent YAML files are valid
3. Look for loading errors in daemon logs

## Complete Example

```yaml
# examples/triggers/telegram-with-builtins.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-full-featured
  labels:
    platform: telegram
    environment: production

spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}

  # Built-in handlers for system commands
  commands:
    /help:
      agent: builtin
      description: "Show available commands with interactive menu"
    /agent:
      agent: builtin
      description: "Switch between agents using selection buttons"
    /fleet:
      agent: builtin
      description: "Switch between fleets using selection buttons"
    /info:
      agent: builtin
      description: "Show system information"

    # LLM-powered commands
    /status:
      agent: devops
      description: "Check system status"
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /pods:
      agent: k8s-agent
      description: "List pods in namespace"
    /logs:
      agent: k8s-agent
      description: "View pod logs"
    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis with multiple agents"

  # Fallback for natural language
  default_agent: devops
```

## See Also

- [Trigger Specification](../reference/trigger-spec.md) - Full trigger configuration reference
- [Agent Switching Guide](agent-switching.md) - How fleet/agent switching works
- [Quickstart: Telegram](quickstart-telegram.md) - Set up a Telegram bot
