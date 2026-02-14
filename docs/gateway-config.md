# Gateway Configuration Guide

## Overview

The messaging gateway connects AOF agents to Slack, Discord, Telegram, and WhatsApp. This guide explains how to configure the gateway for your environment.

## Quick Start

```bash
# 1. Create gateway.yaml
cat > gateway.yaml << 'EOF'
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: my-gateway

spec:
  # Runtime connection (Phase 1 infrastructure)
  runtime:
    websocket_url: "ws://localhost:8080/ws"
    session_id: "${SESSION_ID}"  # Auto-generated if not set

  # Platform adapters
  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${SLACK_BOT_TOKEN}"       # xoxb-...
        app_token: "${SLACK_APP_TOKEN}"       # xapp-1-...
        signing_secret: "${SLACK_SIGNING_SECRET}"
        bot_user_id: "${SLACK_BOT_USER_ID}"   # U...
        allowed_channels:
          - "C01234567"  # #ops-team
          - "C89012345"  # #incidents
      rate_limit:
        requests_per_second: 1
        burst_size: 5

  # Squad definitions
  squads:
    - name: ops-team
      description: "Operations team agents"
      agents:
        - k8s-monitor
        - incident-responder
        - log-analyzer
      channels:
        slack: "C01234567"
        discord: "987654321098765432"
        telegram: "-1001234567890"
EOF

# 2. Set environment variables
export SLACK_BOT_TOKEN="xoxb-your-token"
export SLACK_APP_TOKEN="xapp-your-token"
export SLACK_BOT_USER_ID="U01234567"

# 3. Start gateway
aofctl serve --gateway-config gateway.yaml
```

## Configuration Schema

### Top-Level Structure

```yaml
apiVersion: aof.dev/v1  # Required: Must be "aof.dev/v1"
kind: Gateway           # Required: Must be "Gateway"
metadata:
  name: string          # Required: Gateway name (unique identifier)

spec:
  runtime:              # Required: Runtime connection config
    websocket_url: string        # Required: WebSocket URL to agent runtime
    session_id: string           # Optional: Session ID (auto-generated)

  adapters:             # Required: List of platform adapters
    - platform: string           # Required: slack | discord | telegram | whatsapp
      enabled: boolean           # Required: Whether adapter is enabled
      config: object             # Required: Platform-specific configuration
      rate_limit:                # Required: Rate limit configuration
        requests_per_second: number
        burst_size: number

  squads:               # Optional: Squad definitions
    - name: string               # Required: Squad name (unique)
      description: string        # Required: Human-readable description
      agents:                    # Required: List of agent IDs
        - string
      channels:                  # Required: Platform channel mappings
        slack: string            # Optional: Slack channel ID (C...)
        discord: string          # Optional: Discord channel ID (numeric)
        telegram: string         # Optional: Telegram chat ID (numeric or -...)
        whatsapp: string         # Optional: WhatsApp phone number
```

### Runtime Configuration

```yaml
runtime:
  # WebSocket URL to AOF agent runtime (Phase 1 infrastructure)
  websocket_url: "ws://localhost:8080/ws"

  # Session ID (optional, auto-generated if not set)
  session_id: "${SESSION_ID}"
```

### Adapter Configuration

Each adapter has:
- **platform**: Platform type (slack, discord, telegram, whatsapp)
- **enabled**: Whether adapter is active
- **config**: Platform-specific JSON configuration
- **rate_limit**: Requests per second and burst size

### Squad Configuration

Squads define groups of agents that monitor specific channels:

```yaml
squads:
  - name: ops-team                    # Unique squad name
    description: "Operations team"    # Human-readable description
    agents:                           # Agent IDs in this squad
      - k8s-monitor
      - incident-responder
    channels:                         # Channel mappings per platform
      slack: "C01234567"              # Slack channel ID
      discord: "987654321098765432"   # Discord channel ID
      telegram: "-1001234567890"      # Telegram group ID
```

**Validation rules:**
- Squad names must be unique
- At least one channel must be configured per squad
- Channel IDs must be non-empty strings
- Agent IDs can reference non-existent agents (warning only)

## Platform-Specific Setup

### Slack

**Connection:** Socket Mode (NAT-transparent WebSocket)

**Setup:**
1. Create app at https://api.slack.com/apps
2. Enable Socket Mode (Settings → Socket Mode)
3. Add bot scopes: `channels:history`, `chat:write`, `reactions:read`
4. Install app to workspace
5. Copy Bot Token (xoxb-...) and App Token (xapp-...)

**Configuration:**
```yaml
- platform: slack
  enabled: true
  config:
    bot_token: "${SLACK_BOT_TOKEN}"          # Bot User OAuth Token (xoxb-...)
    app_token: "${SLACK_APP_TOKEN}"          # App-Level Token (xapp-...)
    signing_secret: "${SLACK_SIGNING_SECRET}" # Signing Secret
    bot_user_id: "${SLACK_BOT_USER_ID}"      # Bot User ID (U...)
    allowed_channels:                         # Optional: Restrict to channels
      - "C01234567"
      - "C89012345"
  rate_limit:
    requests_per_second: 1   # Slack rate limit: 1 req/sec
    burst_size: 5
```

**Rate limits:** 1 req/sec (Tier 1), burst up to 5 messages

### Discord

**Connection:** Gateway API (NAT-transparent WebSocket)

**Setup:**
1. Create bot at https://discord.com/developers/applications
2. Enable MESSAGE_CONTENT intent (Bot → Privileged Gateway Intents)
3. Add bot to server (OAuth2 → URL Generator → bot scope → permissions)
4. Copy Bot Token

**Configuration:**
```yaml
- platform: discord
  enabled: true
  config:
    bot_token: "${DISCORD_BOT_TOKEN}"        # Bot Token
    application_id: "${DISCORD_APP_ID}"      # Application ID
    public_key: "${DISCORD_PUBLIC_KEY}"      # Public Key
    guild_ids:                                # Server (guild) IDs
      - "123456789012345678"
  rate_limit:
    requests_per_second: 10  # Discord rate limit: 10 req/sec per channel
    burst_size: 20
```

**Rate limits:** 10 req/sec per channel, burst up to 20 messages

### Telegram

**Connection:** Long polling (NAT-transparent HTTP)

**Setup:**
1. Create bot with @BotFather
2. Copy Bot Token
3. Add bot to group/channel

**Configuration:**
```yaml
- platform: telegram
  enabled: true
  config:
    bot_token: "${TELEGRAM_BOT_TOKEN}"       # Bot Token from @BotFather
    connection_mode: long_polling             # long_polling (default) or webhook
  rate_limit:
    messages_per_second: 30  # Telegram rate limit: 30 msg/sec
    burst_size: 50
```

**Rate limits:** 30 msg/sec to group, burst up to 50 messages

### WhatsApp (Future)

WhatsApp integration coming in future release.

## Environment Variables

### Variable Substitution

The gateway supports environment variable substitution in configuration:

```yaml
config:
  bot_token: "${SLACK_BOT_TOKEN}"  # Replaced with env var value
  app_token: "${SLACK_APP_TOKEN}"
```

**Pattern:** `${VARIABLE_NAME}` (uppercase, numbers, underscores)

**Behavior:**
- Missing variables trigger error: "Missing required environment variables: SLACK_BOT_TOKEN"
- Variables are resolved before YAML parsing
- Empty string substituted if variable unset (with warning)

### Using .env Files

For local development, use `.env` file:

```bash
# .env (add to .gitignore!)
SLACK_BOT_TOKEN=xoxb-your-token
SLACK_APP_TOKEN=xapp-your-token
SLACK_BOT_USER_ID=U01234567
```

The gateway automatically loads `.env` file if present.

## Security Best Practices

### Never Commit Tokens

```bash
# Add to .gitignore
.env
gateway.yaml  # If it contains tokens
```

### Use Secret Management in Production

**Kubernetes:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: gateway-secrets
type: Opaque
stringData:
  SLACK_BOT_TOKEN: xoxb-...
  SLACK_APP_TOKEN: xapp-...
```

**AWS Secrets Manager:**
```bash
export SLACK_BOT_TOKEN=$(aws secretsmanager get-secret-value \
  --secret-id slack-bot-token --query SecretString --output text)
```

### Rotate Tokens Regularly

Regenerate platform tokens every 90 days.

### Sanitized Logging

The gateway automatically sanitizes tokens in logs:
- Only first 8 characters logged: `xoxb-123...`
- Full tokens never appear in logs or error messages

## Validation

Validate configuration without starting server:

```bash
aofctl serve --gateway-config gateway.yaml --validate-config
```

Expected output:
```
✓ Gateway config is valid
  Adapters: 3
  Squads: 2
```

**Validation checks:**
- apiVersion is "aof.dev/v1"
- kind is "Gateway"
- All required fields present
- Environment variables resolved
- Squad names unique
- At least one channel per squad
- Channel IDs non-empty

## Examples

### Single Platform (Slack Only)

```yaml
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: slack-only-gateway

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${SLACK_BOT_TOKEN}"
        app_token: "${SLACK_APP_TOKEN}"
        bot_user_id: "${SLACK_BOT_USER_ID}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

  squads:
    - name: default
      description: "Default squad"
      agents:
        - default-agent
      channels:
        slack: "C01234567"
```

### Multi-Platform (Slack + Discord + Telegram)

```yaml
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: multi-platform-gateway

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${SLACK_BOT_TOKEN}"
        app_token: "${SLACK_APP_TOKEN}"
        bot_user_id: "${SLACK_BOT_USER_ID}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

    - platform: discord
      enabled: true
      config:
        bot_token: "${DISCORD_BOT_TOKEN}"
        application_id: "${DISCORD_APP_ID}"
        public_key: "${DISCORD_PUBLIC_KEY}"
        guild_ids:
          - "123456789012345678"
      rate_limit:
        requests_per_second: 10
        burst_size: 20

    - platform: telegram
      enabled: true
      config:
        bot_token: "${TELEGRAM_BOT_TOKEN}"
        connection_mode: long_polling
      rate_limit:
        messages_per_second: 30
        burst_size: 50

  squads:
    - name: ops-team
      description: "Operations team agents"
      agents:
        - k8s-monitor
        - incident-responder
      channels:
        slack: "C01234567"
        discord: "987654321098765432"
        telegram: "-1001234567890"

    - name: dev-team
      description: "Development team agents"
      agents:
        - code-reviewer
        - ci-cd-manager
      channels:
        slack: "C98765432"
        discord: "123456789012345678"
```

### Development Setup (Disabled Adapters)

```yaml
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: dev-gateway

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${SLACK_BOT_TOKEN}"
        app_token: "${SLACK_APP_TOKEN}"
        bot_user_id: "${SLACK_BOT_USER_ID}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

    # Discord disabled for local development
    - platform: discord
      enabled: false
      config: {}
      rate_limit:
        requests_per_second: 10
        burst_size: 20

  squads:
    - name: dev-squad
      description: "Development squad"
      agents:
        - test-agent
      channels:
        slack: "C01234567"
```

## See Also

- [Troubleshooting Guide](troubleshooting/gateway-issues.md)
- [Internal Architecture](internal/03-messaging-gateway-architecture.md)
- [AOF Documentation](https://docs.aof.sh)
