# DaemonConfig Reference

Complete reference for DaemonConfig resource specifications. The DaemonConfig resource configures the AOF webhook server that connects messaging platforms to your agents.

## Overview

A DaemonConfig defines how the AOF server runs, which platforms it connects to, and how it routes messages to agents.

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value

spec:
  server:                   # Required: Server configuration
    port: int
    host: string
  platforms:                # Required: Platform integrations
    slack: object
    telegram: object
    discord: object
    whatsapp: object
    jira: object
  agents:                   # Required: Agent discovery
    directory: string
  fleets:                   # Optional: Fleet discovery
    directory: string
  flows:                    # Optional: AgentFlow routing
    directory: string
  runtime:                  # Optional: Runtime settings
    default_agent: string
    max_concurrent_tasks: int
```

---

## Server Configuration

### `spec.server`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `port` | int | Yes | 8080 | HTTP port to listen on |
| `host` | string | No | "0.0.0.0" | Host to bind to |
| `cors` | bool | No | false | Enable CORS headers |
| `timeout_secs` | int | No | 30 | Request timeout |

**Example:**
```yaml
spec:
  server:
    port: 8080
    host: "0.0.0.0"
    cors: true
    timeout_secs: 30
```

---

## Platform Configurations

### Slack Platform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable Slack integration |
| `bot_token_env` | string | Yes | Env var for bot token (xoxb-...) |
| `signing_secret_env` | string | Yes | Env var for signing secret |
| `approval_allowed_users` | array | No | User IDs who can approve commands |

**Required OAuth Scopes:**
- `chat:write` - Send messages
- `app_mentions:read` - Respond to @mentions
- `reactions:read` - Read approval reactions
- `reactions:write` - Add approval buttons

**Required Event Subscriptions:**
- `app_mention` - Bot mentions
- `message.channels` - Channel messages
- `message.im` - Direct messages
- `reaction_added` - For approval workflow

**Example:**
```yaml
spec:
  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

      # Optional: Restrict who can approve destructive commands
      approval_allowed_users:
        - U12345678  # SRE Lead
        - U87654321  # Platform Lead
```

### Telegram Platform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable Telegram integration |
| `bot_token_env` | string | Yes | Env var for bot token from @BotFather |
| `webhook_secret` | string | No | Optional webhook verification secret |
| `allowed_users` | array | No | Telegram user IDs allowed to use bot |
| `allowed_groups` | array | No | Telegram group IDs allowed |

**Note:** Telegram is **read-only by default** for safety. Destructive commands are blocked.

**Example:**
```yaml
spec:
  platforms:
    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN

      # Optional: Restrict to specific users
      allowed_users:
        - 123456789  # Your Telegram user ID
        - 987654321  # Team member

      # Optional: Restrict to specific groups
      allowed_groups:
        - -1001234567890  # Your ops group
```

### Discord Platform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable Discord integration |
| `bot_token_env` | string | Yes | Env var for bot token |
| `application_id_env` | string | Yes | Env var for application ID |

**Example:**
```yaml
spec:
  platforms:
    discord:
      enabled: true
      bot_token_env: DISCORD_BOT_TOKEN
      application_id_env: DISCORD_APPLICATION_ID
```

### WhatsApp Platform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable WhatsApp Business integration |
| `phone_number_id_env` | string | Yes | Env var for phone number ID |
| `access_token_env` | string | Yes | Env var for access token |
| `verify_token_env` | string | Yes | Env var for webhook verify token |

**Example:**
```yaml
spec:
  platforms:
    whatsapp:
      enabled: true
      phone_number_id_env: WHATSAPP_PHONE_NUMBER_ID
      access_token_env: WHATSAPP_ACCESS_TOKEN
      verify_token_env: WHATSAPP_VERIFY_TOKEN
```

### Jira Platform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable Jira Cloud integration |
| `base_url` | string | Yes* | Jira instance URL (e.g., `https://your-domain.atlassian.net`) |
| `cloud_id_env` | string | Yes* | Env var for Jira Cloud ID (alternative to base_url) |
| `user_email_env` | string | Yes | Env var for user email for API authentication |
| `api_token_env` | string | Yes | Env var for API token |
| `webhook_secret_env` | string | Yes | Env var for webhook secret (for signature verification) |
| `bot_name` | string | No | Bot name for comments (default: "aofbot") |
| `allowed_projects` | array | No | Project keys allowed to trigger (whitelist) |
| `allowed_events` | array | No | Event types to handle (whitelist) |

*Either `base_url` or `cloud_id_env` must be provided.

**Supported Events:**
- `jira:issue_created` - Issue created
- `jira:issue_updated` - Issue updated
- `jira:issue_deleted` - Issue deleted
- `comment_created` - Comment added
- `comment_updated` - Comment updated
- `comment_deleted` - Comment deleted
- `sprint_started` - Sprint started
- `sprint_closed` - Sprint closed
- `worklog_created` - Work logged
- `worklog_updated` - Worklog updated

**Example:**
```yaml
spec:
  platforms:
    jira:
      enabled: true
      base_url: https://your-domain.atlassian.net
      user_email_env: JIRA_USER_EMAIL
      api_token_env: JIRA_API_TOKEN
      webhook_secret_env: JIRA_WEBHOOK_SECRET
      bot_name: aof-automation

      # Optional: Restrict to specific projects
      allowed_projects:
        - SCRUM
        - OPS

      # Optional: Only handle these events
      allowed_events:
        - jira:issue_created
        - jira:issue_updated
        - comment_created
```

**Setting up Jira Automation webhook URL:**

Configure your Jira Automation rules to POST to:
```
https://your-domain/webhook/jira
```

---

## Agent Discovery

### `spec.agents`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `directory` | string | Yes | - | Path to Agent YAML files |
| `watch` | bool | No | false | Hot-reload on file changes |

**Example:**
```yaml
spec:
  agents:
    directory: "./agents"
    watch: true  # Reload agents when files change
```

---

## Fleet Discovery

### `spec.fleets`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `directory` | string | No | - | Path to Fleet YAML files |
| `watch` | bool | No | false | Hot-reload on file changes |

**Example:**
```yaml
spec:
  fleets:
    directory: "./fleets"
    watch: false
```

---

## AgentFlow Routing

### `spec.flows`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `directory` | string | No | - | Path to AgentFlow YAML files |
| `enabled` | bool | No | false | Enable flow-based routing |
| `watch` | bool | No | false | Hot-reload on file changes |

**Example:**
```yaml
spec:
  flows:
    directory: "./flows"
    enabled: true
    watch: false
```

---

## Runtime Configuration

### `spec.runtime`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `default_agent` | string | No | - | Fallback agent for unmatched messages |
| `default_model` | string | No | - | Default model if agent doesn't specify |
| `max_concurrent_tasks` | int | No | 10 | Max parallel agent executions |
| `task_timeout_secs` | int | No | 300 | Timeout per task execution |
| `max_tasks_per_user` | int | No | 3 | Rate limit per user |

**Example:**
```yaml
spec:
  runtime:
    default_agent: k8s-ops
    max_concurrent_tasks: 10
    task_timeout_secs: 300
    max_tasks_per_user: 3
```

---

## Complete Examples

### Minimal Telegram Bot

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: telegram-bot

spec:
  server:
    port: 8080

  platforms:
    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN

  agents:
    directory: "./agents"

  runtime:
    default_agent: k8s-ops
```

### Production Slack Bot

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: slack-production
  labels:
    env: production

spec:
  server:
    port: 3000
    host: "0.0.0.0"
    cors: true
    timeout_secs: 30

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET
      approval_allowed_users:
        - U12345678  # SRE Lead
        - U87654321  # Platform Lead

  agents:
    directory: "/app/agents"
    watch: false

  fleets:
    directory: "/app/fleets"

  flows:
    directory: "/app/flows"
    enabled: true

  runtime:
    default_agent: devops
    max_concurrent_tasks: 20
    task_timeout_secs: 600
    max_tasks_per_user: 5
```

### Multi-Platform Configuration

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-platform

spec:
  server:
    port: 8080
    host: "0.0.0.0"

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

    telegram:
      enabled: true
      bot_token_env: TELEGRAM_BOT_TOKEN
      allowed_users:
        - 123456789

    discord:
      enabled: false
      bot_token_env: DISCORD_BOT_TOKEN
      application_id_env: DISCORD_APPLICATION_ID

  agents:
    directory: "./agents"
    watch: true

  fleets:
    directory: "./fleets"

  runtime:
    default_agent: devops
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

---

## Environment Variables

DaemonConfig references environment variables for sensitive data. Never hardcode tokens in YAML files.

**Required variables by platform:**

| Platform | Variables |
|----------|-----------|
| Slack | `SLACK_BOT_TOKEN`, `SLACK_SIGNING_SECRET` |
| Telegram | `TELEGRAM_BOT_TOKEN` |
| Discord | `DISCORD_BOT_TOKEN`, `DISCORD_APPLICATION_ID` |
| WhatsApp | `WHATSAPP_PHONE_NUMBER_ID`, `WHATSAPP_ACCESS_TOKEN`, `WHATSAPP_VERIFY_TOKEN` |
| Jira | `JIRA_USER_EMAIL`, `JIRA_API_TOKEN`, `JIRA_WEBHOOK_SECRET` (+ `JIRA_CLOUD_ID` or `base_url` in config) |

**LLM API keys:**
| Provider | Variable |
|----------|----------|
| Google | `GOOGLE_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| OpenAI | `OPENAI_API_KEY` |
| Groq | `GROQ_API_KEY` |

**Example startup:**
```bash
export TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrSTUvwxYZ
export SLACK_BOT_TOKEN=xoxb-your-slack-token
export SLACK_SIGNING_SECRET=your-signing-secret
export GOOGLE_API_KEY=your-google-api-key

# Use the built-in example config
aofctl serve --config examples/config/daemon.yaml

# Or with a custom config
aofctl serve --config config/daemon.yaml
```

---

## CLI Usage

```bash
# Start server with config file
aofctl serve --config daemon-config.yaml

# Override directories via CLI
aofctl serve \
  --config daemon-config.yaml \
  --agents-dir ./agents \
  --fleets-dir ./fleets \
  --flows-dir ./flows

# Override port
aofctl serve --config daemon-config.yaml --port 3000
```

### Expected Startup Output

When the server starts successfully, you'll see:

```
Loading configuration from: daemon-config.yaml
Starting AOF Trigger Server
  Bind address: 0.0.0.0:3000
  Default agent for natural language: devops
  Registered platform: slack
  Registered platform: telegram
Loading AgentFlows from: ./flows/
  Pre-loaded 19 agents from "./agents/"
  Loaded 3 AgentFlows: ["k8s-health-check", "docker-troubleshoot", "data-pipeline"]
Server starting...
  Health check: http://0.0.0.0:3000/health
  Webhook endpoint: http://0.0.0.0:3000/webhook/{platform}
Press Ctrl+C to stop
```

---

## Platform Safety

### Telegram Read-Only Mode

Telegram is configured as read-only by default:
- **Allowed:** `kubectl get`, `docker ps`, `aws describe-*`
- **Blocked:** `kubectl delete`, `docker rm`, `aws terminate-*`

This protects against accidental destructive commands from mobile.

### Slack Approval Workflow

Slack supports human-in-the-loop approval for destructive commands:

1. Agent detects destructive command
2. Agent outputs `requires_approval: true`
3. User sees approval message with reactions
4. User reacts with checkmark to approve or X to deny
5. Command executes only on approval

---

## Webhook Endpoints

The server exposes these endpoints for each platform:

| Platform | Webhook URL |
|----------|-------------|
| Slack | `https://your-domain/webhook/slack` |
| Telegram | `https://your-domain/webhook/telegram` |
| Discord | `https://your-domain/webhook/discord` |
| WhatsApp | `https://your-domain/webhook/whatsapp` |
| GitHub | `https://your-domain/webhook/github` |
| GitLab | `https://your-domain/webhook/gitlab` |
| Bitbucket | `https://your-domain/webhook/bitbucket` |
| Jira | `https://your-domain/webhook/jira` |

---

## Git Platform Behavior

GitHub, GitLab, and Bitbucket handle responses differently from chat platforms:

| Aspect | Chat Platforms (Slack/Telegram/Discord) | Git Platforms (GitHub/GitLab/Bitbucket) |
|--------|----------------------------------------|----------------------------------------|
| Response style | Real-time updates | Single response only |
| Progress indicators | "🤔 Thinking...", "🔄 Processing..." shown | Skipped (would create noisy comment threads) |
| Message updates | Can edit existing messages | Creates new comments |
| Best for | Interactive conversations | PR reviews, issue triage |

**Why single responses for Git platforms:**
- Each `send_response` creates a NEW comment in GitHub/GitLab/Bitbucket
- Progress indicators like "Thinking..." would flood PR threads with multiple comments
- Only the final response is posted, keeping PR reviews clean and professional

**Example: `/review` on GitHub PR**
1. User posts `/review` comment on PR
2. Bot processes (no intermediate comments)
3. Bot posts ONE comment with the complete review

---

## See Also

- [Agent Spec](agent-spec.md) - Agent resource reference
- [Fleet Spec](fleet-spec.md) - Fleet resource reference
- [AgentFlow Spec](agentflow-spec.md) - Workflow routing
- [aofctl CLI](aofctl.md) - Command reference
- [Slack Bot Tutorial](../tutorials/slack-bot.md) - Build a Slack bot
- [Telegram Bot Tutorial](../tutorials/telegram-ops-bot.md) - Build a Telegram bot
