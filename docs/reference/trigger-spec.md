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

| Type | Status | Description | Use Case |
|------|--------|-------------|----------|
| `HTTP` | ✅ Stable | Generic HTTP endpoint | CI/CD, custom integrations |
| `GitHub` | ✅ Stable | GitHub repository events | PR automation, deployments |
| `GitLab` | 🧪 Experimental | GitLab repository events | PR automation (untested) |
| `Bitbucket` | 🧪 Experimental | Bitbucket repository events | PR automation (untested) |
| `Jira` | ✅ Stable | Jira issue events | Issue-to-incident workflow |

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

**Status: ✅ Stable (Production Ready)**

GitHub webhooks for repository events including PRs, issues, pushes, and workflow runs. AOF verifies webhook signatures and routes events to the appropriate agents, fleets, or flows.

**Webhook URL:** `https://your-aof-server/webhook/github`

**Setup Requirements:**
1. Go to repository **Settings → Webhooks → Add webhook**
2. Set Payload URL to `https://your-aof-server/webhook/github` (or custom path)
3. Set Content type to `application/json`
4. Create a webhook secret and save it securely
5. Select individual events or "Send me everything"
6. Ensure webhook is active

**Configuration Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `webhook_secret` | string | Yes | HMAC-SHA256 secret for signature verification |
| `github_events` | array | No | Events to handle (default: all) |
| `repositories` | array | No | Repository filter (owner/repo format, supports wildcards) |
| `branches` | array | No | Branch filter (supports wildcards like `release/*`) |
| `path` | string | No | Custom webhook path (default: `/webhook/github`) |

**Supported Events:**

| Event | Description | Common Actions |
|-------|-------------|----------------|
| `pull_request` | PR lifecycle events | `opened`, `synchronize`, `closed`, `reopened`, `edited`, `assigned`, `labeled`, `ready_for_review` |
| `pull_request_review` | PR review submitted | `submitted`, `edited`, `dismissed` |
| `pull_request_review_comment` | Comment on PR review | `created`, `edited`, `deleted` |
| `push` | Code pushed to branch/tag | N/A (no action field) |
| `issues` | Issue lifecycle events | `opened`, `edited`, `closed`, `reopened`, `assigned`, `labeled`, `transferred` |
| `issue_comment` | Comment on issue/PR | `created`, `edited`, `deleted` |
| `workflow_run` | GitHub Actions workflow | `requested`, `in_progress`, `completed` |
| `workflow_job` | GitHub Actions job | `queued`, `in_progress`, `completed` |
| `check_run` | CI check status | `created`, `completed`, `rerequested` |
| `check_suite` | CI check suite | `completed`, `requested`, `rerequested` |
| `release` | Release published | `published`, `created`, `edited`, `deleted`, `released` |
| `deployment` | Deployment created/updated | `created` |
| `deployment_status` | Deployment status change | `created` |
| `status` | Commit status updated | N/A (no action field) |
| `create` | Branch/tag created | N/A (no action field) |
| `delete` | Branch/tag deleted | N/A (no action field) |

**Command Bindings:**

Commands can be bound to specific `event.action` combinations for fine-grained routing:

```yaml
commands:
  # Event.action format (most common)
  pull_request.opened:
    fleet: pr-review-fleet
    description: "Auto-review new PRs"

  pull_request.synchronize:
    fleet: pr-review-fleet
    description: "Re-review on push"

  pull_request.closed:
    agent: pr-cleanup-agent
    description: "Clean up PR resources"

  # Event-only format (matches all actions)
  push:
    agent: ci-agent
    description: "Trigger CI pipeline"

  issues.opened:
    agent: triage-agent
    description: "Triage new issues"

  issue_comment.created:
    agent: comment-handler-agent
    description: "Process issue comments"

  workflow_run.completed:
    agent: ci-status-agent
    description: "Report CI results"

  release.published:
    flow: release-deployment-flow
    description: "Deploy release to production"
```

**Basic Example:**

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
    repositories:
      - myorg/myrepo
      - myorg/other-repo

  commands:
    pull_request.opened:
      fleet: pr-review-fleet
      description: "Auto-review new PRs"
    pull_request.synchronize:
      fleet: pr-review-fleet
      description: "Re-review on push"
    push:
      agent: ci-agent
      description: "Trigger CI pipeline"

  default_agent: github-ops-agent
```

**Advanced Example with Filters:**

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-release-automation
spec:
  type: GitHub
  config:
    webhook_secret: ${GITHUB_WEBHOOK_SECRET}
    path: /webhook/github-releases  # Custom path
    github_events:
      - pull_request
      - push
      - release
      - workflow_run
    repositories:
      - myorg/backend-*  # Wildcard matching
      - myorg/frontend
    branches:
      - main
      - "release/*"  # Match release/v1.0, release/v2.0, etc.

  commands:
    # PR automation
    pull_request.opened:
      fleet: pr-review-fleet
      description: "Auto-review and test new PRs"

    pull_request.ready_for_review:
      agent: ci-trigger-agent
      description: "Run full CI suite when PR ready"

    pull_request.closed:
      agent: cleanup-agent
      description: "Clean up preview environments"

    # Push events
    push:
      agent: ci-agent
      description: "Run CI checks on push"

    # Release automation
    release.published:
      flow: production-deploy-flow
      description: "Deploy release to production"

    # CI/CD monitoring
    workflow_run.completed:
      agent: ci-reporter-agent
      description: "Report CI/CD results to Slack"

    check_run.completed:
      agent: status-checker-agent
      description: "Update PR status checks"

  default_agent: github-ops-agent
```

**Environment Variables:**
```bash
export GITHUB_WEBHOOK_SECRET="your-strong-webhook-secret"
```

**Event Context:**

GitHub events provide rich context to agents:
- `github.event` - Event type (e.g., `pull_request`, `push`)
- `github.action` - Event action (e.g., `opened`, `closed`, `synchronize`)
- `github.repo` - Repository full name (owner/repo)
- `github.branch` - Branch name (for push/PR events)
- `github.pr.number` - PR number (for PR events)
- `github.pr.title` - PR title
- `github.pr.author` - PR author username
- `github.sender` - User who triggered the event
- `github.ref` - Git ref (for push events)
- `github.sha` - Commit SHA

**Webhook Security:**

AOF validates GitHub webhook signatures using HMAC-SHA256:
1. GitHub signs each webhook with your secret
2. AOF verifies the `X-Hub-Signature-256` header
3. Requests with invalid signatures are rejected

**GitLab and Bitbucket Support:**

| Platform | Status | Notes |
|----------|--------|-------|
| **GitLab** | 🧪 Experimental | Implemented but untested - contributions welcome |
| **Bitbucket** | 🧪 Experimental | Implemented but untested - contributions welcome |

GitLab and Bitbucket webhook adapters exist in the codebase following the same patterns as GitHub, but have not been tested in production. The implementation includes:
- Webhook signature verification
- Event parsing for PRs, issues, pushes
- API client stubs for posting comments and reviews

**Community contributions needed:**
- Real-world testing with GitLab/Bitbucket webhooks
- API integration validation
- Bug reports and fixes
- Documentation improvements

If you use GitLab or Bitbucket and would like to help validate these integrations, please open an issue or submit a PR with your findings.

**See Also:**
- [GitHub Webhooks Documentation](https://docs.github.com/en/webhooks)
- [GitHub Webhook Events](https://docs.github.com/en/webhooks/webhook-events-and-payloads)

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

### Built-in Command Handlers

Use `agent: builtin` to invoke AOF's built-in interactive command handlers instead of routing to an LLM agent. This is useful for commands that need rich interactive menus.

```yaml
commands:
  /help:
    agent: builtin          # Uses built-in help menu with fleet/agent selection
    description: "Show available commands"
  /agent:
    agent: builtin          # Uses built-in agent selection menu
    description: "Switch active agent"
  /fleet:
    agent: builtin          # Uses built-in fleet selection menu
    description: "Switch active fleet"
```

**Available built-in handlers:**
| Command | Description |
|---------|-------------|
| `/help` | Interactive help menu with fleet/agent selection buttons |
| `/agent` | Agent selection menu with inline keyboard |
| `/fleet` | Fleet selection menu with inline keyboard |
| `/info` | System information display |
| `/flows` | List available flows |

**When to use `builtin` vs agent:**
- Use `agent: builtin` for interactive menus and system commands
- Use `agent: <name>` when you want the LLM to handle the command

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
