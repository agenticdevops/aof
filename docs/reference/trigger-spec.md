# Trigger Resource Reference

Complete reference for Trigger resource specifications. Triggers define message sources and command routing - they are self-contained units that include platform configuration and command bindings.

## Overview

A Trigger represents a message source (Slack, Telegram, HTTP, etc.) with embedded command routing. Each trigger includes:
- **Platform configuration** - Authentication and connection settings
- **Command bindings** - Maps slash commands to agents, fleets, or flows
- **Default agent** - Fallback for natural language messages

This enables:
- Self-contained platform configuration
- Direct command-to-handler routing
- Clear separation of concerns between platforms

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value
  annotations:              # Optional: Additional metadata
    key: value

spec:
  type: TriggerType         # Required: Platform type
  config:                   # Required: Platform-specific config
    bot_token: string
    signing_secret: string
    # ... platform-specific fields
  commands:                 # Optional: Command bindings
    /command:
      agent: string         # Route to agent
      fleet: string         # Route to fleet
      flow: string          # Route to flow
      description: string   # Help text
  default_agent: string     # Optional: Fallback for natural language
  enabled: bool             # Optional: Enable/disable trigger
```

---

## Trigger Types Overview

AOF supports triggers across four categories:

### Chat Platforms
Real-time messaging platforms for interactive ops bots.

| Type | Description | Use Case |
|------|-------------|----------|
| `Slack` | Slack workspace bot | Primary ChatOps for teams |
| `Telegram` | Telegram bot | Mobile-first on-call |
| `Discord` | Discord server bot | Community/gaming ops |
| `WhatsApp` | WhatsApp Business API | Mobile incident response |

### Webhooks & Integrations
Event-driven triggers from external systems.

| Type | Description | Use Case |
|------|-------------|----------|
| `HTTP` | Generic HTTP endpoint | CI/CD, custom integrations |
| `GitHub` | GitHub repository events | PR automation, deployments |
| `Jira` | Jira issue events | Issue-to-incident workflow |

### Incident Management
Alert and incident handling triggers.

| Type | Description | Use Case |
|------|-------------|----------|
| `PagerDuty` | PagerDuty incidents | Incident auto-remediation |

### Scheduling & CLI
Time-based and manual triggers.

| Type | Description | Use Case |
|------|-------------|----------|
| `Schedule` | Cron-based trigger | Daily reports, health checks |
| `Manual` | CLI invocation | Testing, ad-hoc execution |

---

## Platform Configurations

### Slack Trigger

Slack is the most common platform for enterprise ChatOps. AOF connects via Slack's Socket Mode for real-time communication.

**Setup Requirements:**
1. Create Slack App at [api.slack.com/apps](https://api.slack.com/apps)
2. Enable Socket Mode and get App-Level Token (`xapp-...`)
3. Add Bot User with scopes: `app_mentions:read`, `chat:write`, `channels:history`
4. Install to workspace and copy Bot Token (`xoxb-...`)

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod-channel
spec:
  type: Slack
  config:
    # Required
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}

    # Optional filters
    channels:
      - production
      - prod-alerts
    users:
      - U12345678  # Only respond to specific users
    events:
      - app_mention
      - message
    patterns:
      - "kubectl.*"  # Regex patterns
      - "k8s"

  # Command routing
  commands:
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /diagnose:
      fleet: rca-fleet
      description: "Root cause analysis"

  default_agent: devops
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Bot token (xoxb-...) or env var reference |
| `signing_secret` | string | No | Signing secret for verification |
| `channels` | array | No | Channel names or IDs to listen on |
| `users` | array | No | User IDs to respond to |
| `events` | array | No | Event types (app_mention, message) |
| `patterns` | array | No | Message patterns (regex) |

**Environment Variables:**
```bash
export SLACK_BOT_TOKEN="xoxb-your-bot-token"
export SLACK_SIGNING_SECRET="your-signing-secret"
```

### Telegram Trigger

Telegram is ideal for mobile-first on-call workflows. Bots support native command menus and inline keyboards.

**Setup Requirements:**
1. Message [@BotFather](https://t.me/botfather) on Telegram
2. Send `/newbot` and follow prompts to create bot
3. Copy the bot token provided
4. Optional: Send `/setcommands` to define command menu

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-oncall
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}

    # Optional filters
    chat_ids:
      - -1001234567890  # Group chat ID (negative for groups)
    users:
      - "123456789"     # User IDs as strings

  # Command routing (also appears in Telegram command menu)
  commands:
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /pods:
      agent: k8s-agent
      description: "List pods"
    /logs:
      agent: k8s-agent
      description: "View pod logs"

  default_agent: devops
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Bot token from @BotFather |
| `chat_ids` | array | No | Chat/group IDs to listen on |
| `users` | array | No | User IDs to respond to |
| `patterns` | array | No | Message patterns (regex) |

**Environment Variables:**
```bash
export TELEGRAM_BOT_TOKEN="123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
```

**Getting Chat IDs:**
1. Add bot to group/channel
2. Send a message in the chat
3. Visit: `https://api.telegram.org/bot<TOKEN>/getUpdates`
4. Find `"chat":{"id":-1001234567890}` in JSON response

### Discord Trigger

Discord is popular for gaming and community operations. AOF supports both slash commands and message-based interactions.

**Setup Requirements:**
1. Create application at [discord.com/developers](https://discord.com/developers/applications)
2. Add Bot under "Bot" tab
3. Enable Message Content Intent if reading messages
4. Copy Bot Token
5. Use OAuth2 URL Generator to invite bot with required permissions

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: discord-ops-bot
spec:
  type: Discord
  config:
    bot_token: ${DISCORD_BOT_TOKEN}
    app_secret: ${DISCORD_APP_SECRET}

    guild_ids:
      - "123456789012345678"    # Server (guild) IDs
    channels:
      - ops-channel
      - "987654321098765432"    # Channel ID

  commands:
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /status:
      agent: health-agent
      description: "System status check"

  default_agent: devops
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Discord bot token |
| `app_secret` | string | No | Application secret |
| `guild_ids` | array | No | Server IDs to listen on |
| `channels` | array | No | Channel names/IDs |

**Environment Variables:**
```bash
export DISCORD_BOT_TOKEN="your-discord-bot-token"
export DISCORD_APP_SECRET="your-app-secret"
```

### HTTP Trigger

Generic HTTP webhook for custom integrations, CI/CD pipelines, and any system that can send HTTP requests.

**Use Cases:**
- CI/CD deployment triggers (Jenkins, GitLab CI)
- Custom application webhooks
- Monitoring system alerts (Prometheus, Datadog)
- IoT device events

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: ci-cd-webhook
spec:
  type: HTTP
  config:
    path: /webhook/deploy
    methods:
      - POST
    webhook_secret: ${WEBHOOK_SECRET}
    required_headers:
      X-Deploy-Token: "*"

  commands:
    /deploy:
      flow: deploy-flow
      description: "Trigger deployment"
    /rollback:
      flow: rollback-flow
      description: "Rollback deployment"

  default_agent: devops
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | No | URL path pattern (default: /) |
| `methods` | array | No | HTTP methods (GET, POST, etc.) |
| `webhook_secret` | string | No | Secret for HMAC signature verification |
| `required_headers` | map | No | Headers required for authentication |
| `port` | int | No | Port to listen on |
| `host` | string | No | Host to bind to |

**Environment Variables:**
```bash
export WEBHOOK_SECRET="your-secret-key"
```

**Calling the Webhook:**
```bash
curl -X POST https://your-aof-server.com/webhook/deploy \
  -H "Content-Type: application/json" \
  -H "X-Deploy-Token: your-token" \
  -d '{"service": "api", "version": "v2.1.0"}'
```

### Schedule Trigger

Cron-based triggers for automated, time-based operations like daily reports, health checks, and maintenance tasks.

**Use Cases:**
- Daily/weekly status reports
- Periodic health checks
- Scheduled maintenance tasks
- Database cleanup jobs
- Certificate expiration checks

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: daily-report
spec:
  type: Schedule
  config:
    cron: "0 9 * * *"           # 9 AM daily
    timezone: "America/New_York"

  commands:
    /report:
      agent: reporting-agent
      description: "Generate daily report"

  default_agent: reporting-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cron` | string | Yes | Standard cron expression (5 or 6 fields) |
| `timezone` | string | No | IANA timezone (default: UTC) |

**Cron Expression Reference:**
```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

**Common Cron Patterns:**
| Pattern | Description |
|---------|-------------|
| `0 9 * * *` | Every day at 9:00 AM |
| `0 * * * *` | Every hour |
| `*/15 * * * *` | Every 15 minutes |
| `0 9 * * 1-5` | Weekdays at 9:00 AM |
| `0 0 1 * *` | First day of each month at midnight |
| `0 6,18 * * *` | At 6:00 AM and 6:00 PM daily |

### PagerDuty Trigger

Integrates with PagerDuty for incident-driven automation and auto-remediation workflows.

**Setup Requirements:**
1. Log in to [PagerDuty](https://app.pagerduty.com)
2. Go to **Integrations → API Access Keys**
3. Create a new API key with read/write access
4. For Events API v2, get a routing key from **Services → Service → Integrations**

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  type: PagerDuty
  config:
    api_key: ${PAGERDUTY_API_KEY}
    routing_key: ${PAGERDUTY_ROUTING_KEY}
    service_ids:
      - P123ABC
      - P456DEF
    urgencies:
      - high
      - low
    statuses:
      - triggered
      - acknowledged

  commands:
    /ack:
      agent: incident-agent
      description: "Acknowledge incident"
    /resolve:
      agent: incident-agent
      description: "Resolve incident"
    /runbook:
      flow: incident-runbook-flow
      description: "Execute incident runbook"

  default_agent: incident-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string | Yes* | PagerDuty REST API key |
| `routing_key` | string | Yes* | Events API v2 routing key |
| `service_ids` | array | No | Filter by service IDs |
| `urgencies` | array | No | Filter by urgency (high, low) |
| `statuses` | array | No | Filter by status (triggered, acknowledged, resolved) |

*One of `api_key` or `routing_key` required.

**Environment Variables:**
```bash
export PAGERDUTY_API_KEY="u+abcdefghijklmnop"
export PAGERDUTY_ROUTING_KEY="R0123456789ABCDEF0123456789ABCDEF"
```

**Incident Context:**
When a PagerDuty incident triggers, the following context is available:
- `incident.id` - PagerDuty incident ID
- `incident.title` - Incident title
- `incident.urgency` - high or low
- `incident.status` - triggered, acknowledged, or resolved
- `incident.service` - Service name and ID
- `incident.created_at` - Timestamp

### GitHub Trigger

Responds to GitHub repository events for PR automation, CI/CD integration, and issue management.

**Setup Requirements:**
1. Go to repository **Settings → Webhooks → Add webhook**
2. Set Payload URL to your AOF server endpoint
3. Set Content type to `application/json`
4. Create a webhook secret and save it
5. Select events to trigger on (or "Send me everything")

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-pr-events
spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    github_events:
      - pull_request
      - push
      - issues
      - issue_comment
      - pull_request_review
    repositories:
      - myorg/myrepo
      - myorg/other-repo
    branches:
      - main
      - "release/*"

  commands:
    /deploy:
      flow: deploy-flow
      description: "Deploy changes"
    /review:
      agent: code-review-agent
      description: "AI code review"
    /test:
      agent: test-runner-agent
      description: "Run test suite"

  default_agent: github-ops-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `webhook_secret` | string | No | HMAC secret for signature verification |
| `github_events` | array | No | Event types to listen for |
| `repositories` | array | No | Repository filter (owner/repo) |
| `branches` | array | No | Branch filter (supports wildcards) |

**Environment Variables:**
```bash
export GITHUB_WEBHOOK_SECRET="your-webhook-secret"
```

**Supported GitHub Events:**
| Event | Description |
|-------|-------------|
| `push` | Code pushed to repository |
| `pull_request` | PR opened, closed, merged, etc. |
| `pull_request_review` | PR review submitted |
| `issues` | Issue opened, closed, labeled, etc. |
| `issue_comment` | Comment on issue or PR |
| `release` | Release published or edited |
| `workflow_run` | GitHub Actions workflow completed |
| `deployment` | Deployment created or updated |

**Event Context:**
GitHub events provide rich context:
- `github.event` - Event type (push, pull_request, etc.)
- `github.repo` - Repository full name
- `github.branch` - Branch name
- `github.pr.number` - PR number (for PR events)
- `github.sender` - User who triggered the event

### WhatsApp Trigger

Mobile-first incident response via WhatsApp Business API. Ideal for on-call engineers who need to respond from their phones.

**Setup Requirements:**
1. Create a [Meta Business Account](https://business.facebook.com)
2. Set up [WhatsApp Business API](https://developers.facebook.com/docs/whatsapp/cloud-api/get-started)
3. Create a WhatsApp Business App in Meta Developer Console
4. Get your Phone Number ID and Access Token
5. Configure webhook URL for incoming messages

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: whatsapp-oncall
spec:
  type: WhatsApp
  config:
    bot_token: ${WHATSAPP_ACCESS_TOKEN}
    phone_number_id: ${WHATSAPP_PHONE_NUMBER_ID}
    verify_token: ${WHATSAPP_VERIFY_TOKEN}
    business_account_id: ${WHATSAPP_BUSINESS_ID}
    allowed_numbers:
      - "+1234567890"      # On-call engineer
      - "+0987654321"      # SRE lead

  commands:
    /status:
      agent: health-agent
      description: "Check system status"
    /pods:
      agent: k8s-agent
      description: "List Kubernetes pods"
    /logs:
      agent: k8s-agent
      description: "View pod logs"

  default_agent: devops-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | WhatsApp Cloud API access token |
| `phone_number_id` | string | Yes | WhatsApp phone number ID |
| `verify_token` | string | Yes | Webhook verification token |
| `business_account_id` | string | No | WhatsApp Business Account ID |
| `allowed_numbers` | array | No | Phone numbers allowed to interact |

**Environment Variables:**
```bash
export WHATSAPP_ACCESS_TOKEN="EAAxxxxxx..."
export WHATSAPP_PHONE_NUMBER_ID="1234567890123456"
export WHATSAPP_VERIFY_TOKEN="your-verify-token"
export WHATSAPP_BUSINESS_ID="9876543210"
```

**WhatsApp Message Types:**
WhatsApp supports rich message types:
- Text messages with formatting
- Quick reply buttons (up to 3)
- List messages (up to 10 items)
- Location sharing
- Media attachments

### Jira Trigger

Integrates with Atlassian Jira for issue-to-incident workflows and automated ticket management.

**Setup Requirements:**
1. Go to **Jira Settings → System → WebHooks**
2. Create a new webhook pointing to your AOF server
3. Select events to listen for (issue created, updated, etc.)
4. Optional: Create an API token for outbound operations

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-incidents
spec:
  type: Jira
  config:
    webhook_secret: ${JIRA_WEBHOOK_SECRET}
    base_url: https://yourcompany.atlassian.net
    api_token: ${JIRA_API_TOKEN}
    email: ${JIRA_EMAIL}
    projects:
      - OPS
      - INCIDENT
    issue_types:
      - Bug
      - Incident
      - Task
    jql_filter: "priority = Highest AND status = Open"

  commands:
    /triage:
      agent: triage-agent
      description: "Triage and prioritize issue"
    /assign:
      agent: assignment-agent
      description: "Auto-assign to team member"
    /investigate:
      fleet: rca-fleet
      description: "Investigate root cause"

  default_agent: jira-ops-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `webhook_secret` | string | No | Webhook secret for verification |
| `base_url` | string | Yes | Jira instance URL |
| `api_token` | string | No | API token for Jira operations |
| `email` | string | No | Email for API authentication |
| `projects` | array | No | Project keys to monitor |
| `issue_types` | array | No | Issue types to trigger on |
| `jql_filter` | string | No | JQL query for filtering issues |

**Environment Variables:**
```bash
export JIRA_WEBHOOK_SECRET="your-webhook-secret"
export JIRA_API_TOKEN="ATATT3xFfGF0..."
export JIRA_EMAIL="bot@yourcompany.com"
```

**Jira Event Types:**
| Event | Description |
|-------|-------------|
| `jira:issue_created` | New issue created |
| `jira:issue_updated` | Issue fields updated |
| `jira:issue_deleted` | Issue deleted |
| `comment_created` | Comment added to issue |
| `sprint_started` | Sprint started |
| `sprint_closed` | Sprint completed |

### Manual Trigger

CLI-based trigger for testing, ad-hoc execution, and scripted automation.

**Use Cases:**
- Testing agents and flows locally
- Ad-hoc operations from terminal
- Integration with shell scripts
- CI/CD pipeline integration
- Debugging and development

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: cli-manual
spec:
  type: Manual
  config:
    require_confirmation: true
    allowed_commands:
      - /deploy
      - /rollback
      - /diagnose

  commands:
    /deploy:
      flow: deploy-flow
      description: "Deploy to environment"
    /rollback:
      flow: rollback-flow
      description: "Rollback deployment"
    /diagnose:
      fleet: rca-fleet
      description: "Run diagnostics"
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"

  default_agent: devops-agent
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `require_confirmation` | bool | No | Prompt before execution (default: false) |
| `allowed_commands` | array | No | Restrict to specific commands |
| `timeout` | int | No | Execution timeout in seconds |

**CLI Usage:**
```bash
# Run a command through manual trigger
aofctl run trigger cli-manual --command "/deploy --env production"

# Run with input
aofctl run trigger cli-manual --command "/kubectl get pods" --input "namespace=prod"

# Interactive mode
aofctl run trigger cli-manual --interactive

# With confirmation disabled
aofctl run trigger cli-manual --command "/diagnose" --yes
```

**Script Integration:**
```bash
#!/bin/bash
# deploy.sh - Scripted deployment with AOF

ENV=${1:-staging}

echo "Deploying to $ENV..."
aofctl run trigger cli-manual \
  --command "/deploy" \
  --input "environment=$ENV" \
  --yes \
  --output json | jq '.result'
```

---

## Environment Variables

Triggers support `${VAR_NAME}` syntax for environment variable expansion:

```yaml
spec:
  config:
    bot_token: ${SLACK_BOT_TOKEN}  # Expanded at runtime
```

**Security:** Never hardcode tokens in YAML files. Always use environment variable references.

---

## Matching & Filtering

Triggers support multiple levels of filtering:

### Channel Filter
```yaml
config:
  channels:
    - production      # Exact match
    - "#prod-alerts"  # Slack channel name format
```

### User Filter
```yaml
config:
  users:
    - U12345678       # Specific user IDs
    - "*"             # Any user (default)
```

### Pattern Filter (Regex)
```yaml
config:
  patterns:
    - "kubectl.*"     # Commands starting with kubectl
    - "^/deploy"      # Commands starting with /deploy
    - "(?i)help"      # Case-insensitive "help"
```

### Match Scoring

When multiple triggers match, the most specific wins:

| Filter | Score |
|--------|-------|
| Channel match | +100 |
| User match | +80 |
| Pattern match | +60 |
| Platform only | +10 |

---

## Complete Examples

### Production Slack Bot

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod
  labels:
    environment: production
    team: platform
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
    channels:
      - production
      - prod-alerts
      - C0123456789   # Channel ID
    events:
      - app_mention
      - message
  enabled: true
```

### Multi-Pattern Telegram Bot

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-k8s
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
    patterns:
      - "^/kubectl"
      - "^/k8s"
      - "^/pods"
      - "^/deploy"
    users:
      - "123456789"   # On-call engineer
      - "987654321"   # SRE lead
```

### Scheduled Health Check

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: hourly-health-check
spec:
  type: Schedule
  config:
    cron: "0 * * * *"        # Every hour
    timezone: "UTC"
```

---

## Validation

Triggers are validated when loaded:

```bash
# Validate trigger YAML
aofctl validate -f trigger.yaml

# List loaded triggers
aofctl get triggers
```

### Validation Rules
- Name is required and must be DNS-compatible
- Type must be a valid trigger type
- Platform-specific required fields must be present
- Environment variable references are validated at runtime

---

## Command Bindings

Each command maps to one handler: agent, fleet, or flow.

### Command Structure

```yaml
commands:
  /command-name:
    agent: agent-name       # Route to single agent
    fleet: fleet-name       # Route to agent fleet
    flow: flow-name         # Route to multi-step flow
    description: "Help text for this command"
```

**Note:** Only one of `agent`, `fleet`, or `flow` should be specified per command.

### When to Use Each

| Target | Use When | Example |
|--------|----------|---------|
| `agent` | Single-purpose task | `/kubectl` → k8s-agent |
| `fleet` | Multi-agent coordination | `/diagnose` → rca-fleet |
| `flow` | Multi-step workflow | `/deploy` → deploy-flow |

### Example: Full Command Bindings

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}

  commands:
    # Route to single agents
    /kubectl:
      agent: k8s-agent
      description: "Kubernetes operations"
    /docker:
      agent: docker-agent
      description: "Docker container management"

    # Route to fleets (multi-agent)
    /diagnose:
      fleet: rca-fleet
      description: "Multi-model root cause analysis"
    /devops:
      fleet: devops-fleet
      description: "Full DevOps operations"

    # Route to flows (workflows)
    /deploy:
      flow: deploy-flow
      description: "Production deployment with approval"
    /incident:
      flow: incident-flow
      description: "Incident response workflow"

  # Fallback for @mentions and natural language
  default_agent: devops
```

---

## Usage with DaemonConfig

Triggers are loaded from a directory configured in DaemonConfig:

```yaml
# daemon.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-daemon
spec:
  triggers:
    directory: ./examples/triggers/
    watch: false  # Enable for hot-reload
```

When the daemon starts:
1. All triggers are loaded from the directory
2. Platform connections are established
3. Commands are registered for routing
4. Messages are matched and routed to handlers

---

## See Also

- [AgentFlow Reference](./agentflow-spec.md) - Multi-step workflow definitions
- [Fleet Reference](./fleet-spec.md) - Agent fleet specifications
- [Context Reference](./context-spec.md) - Execution environment configuration
- [DaemonConfig Reference](./daemon-config.md) - Server configuration
