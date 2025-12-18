# Building a Telegram Ops Bot with AOF

This tutorial walks you through building a Telegram bot for DevOps and SRE workflows using AOF's trigger system and AgentFlow.

## What You'll Build

By the end of this tutorial, you'll have a Telegram bot that can:
- Execute runbooks on-demand (`/runbook deploy-api`)
- Trigger incident response workflows (`/incident create P1`)
- Query infrastructure status (`/status api-cluster`)
- Approve/reject deployment requests with inline buttons
- Route alerts to on-call engineers

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- A Telegram account
- A server with public HTTPS endpoint (or ngrok for testing)
- Basic familiarity with YAML

## Step 1: Create Your Telegram Bot

### 1.1 Talk to BotFather

1. Open Telegram and search for `@BotFather`
2. Send `/newbot`
3. Follow the prompts:
   - Bot name: `My Ops Bot` (display name)
   - Bot username: `my_ops_bot` (must end in `bot`)
4. Save the **HTTP API token** - you'll need this

```
Done! Congratulations on your new bot. You will find it at t.me/my_ops_bot.
Use this token to access the HTTP API:
6123456789:ABCdefGHIjklMNOpqrSTUvwxYZ123456789
```

### 1.2 Configure Bot Settings

Send these commands to @BotFather:

```
/setcommands
```

Then select your bot and paste:

```
runbook - Execute a runbook
incident - Create or manage incidents
status - Check infrastructure status
deploy - Trigger deployment
rollback - Rollback deployment
approve - Approve pending request
reject - Reject pending request
help - Show available commands
```

## Step 2: Configure AOF Trigger Server

### 2.1 Create Configuration File

Create `config/telegram-bot.yaml`:

```yaml
# AOF Trigger Server Configuration
version: v1
kind: TriggerConfig

server:
  host: "0.0.0.0"
  port: 8080
  base_path: "/webhooks"

platforms:
  telegram:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    webhook_secret: "${TELEGRAM_WEBHOOK_SECRET}"
    bot_name: "my_ops_bot"

    # Only respond to specific groups/users
    allowed_chats:
      - "-1001234567890"  # Your ops channel
      - "123456789"       # Your user ID

    # Require users to be in allowed list for sensitive commands
    allowed_users:
      - "123456789"       # Admin user
      - "987654321"       # On-call engineer

# Command routing
routing:
  default_flow: "help-flow"

  commands:
    "/runbook": "runbook-executor-flow"
    "/incident": "incident-manager-flow"
    "/status": "status-checker-flow"
    "/deploy": "deployment-flow"
    "/rollback": "rollback-flow"
    "/approve": "approval-handler-flow"
    "/reject": "approval-handler-flow"

# Flow discovery
flows:
  directory: "./flows"
  watch: true  # Hot reload on changes
```

### 2.2 Set Environment Variables

```bash
export TELEGRAM_BOT_TOKEN="6123456789:ABCdefGHIjklMNOpqrSTUvwxYZ123456789"
export TELEGRAM_WEBHOOK_SECRET="$(openssl rand -hex 32)"
```

## Step 3: Create AgentFlows

### 3.1 Runbook Executor Flow

Create `flows/runbook-executor-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: runbook-executor
  description: Execute operational runbooks on-demand

triggers:
  - platform: telegram
    type: command
    command: "/runbook"
    description: "Execute a runbook: /runbook <name> [args]"

# Input validation
input:
  required:
    - runbook_name
  schema:
    runbook_name:
      type: string
      pattern: "^[a-z0-9-]+$"
      description: "Runbook identifier"
    args:
      type: array
      items:
        type: string

# Available runbooks
variables:
  runbooks:
    deploy-api:
      description: "Deploy API to production"
      requires_approval: true
      steps:
        - "kubectl rollout restart deployment/api -n production"

    restart-workers:
      description: "Restart background workers"
      requires_approval: false
      steps:
        - "kubectl rollout restart deployment/workers -n production"

    clear-cache:
      description: "Clear Redis cache"
      requires_approval: false
      steps:
        - "redis-cli -h redis.internal FLUSHDB"

    scale-api:
      description: "Scale API replicas"
      requires_approval: true
      args: ["replicas"]
      steps:
        - "kubectl scale deployment/api --replicas=${args.replicas} -n production"

steps:
  - name: validate-runbook
    agent: validator
    action: validate
    input:
      runbook: "{{ input.runbook_name }}"
      available: "{{ variables.runbooks | keys }}"
    on_error:
      response:
        text: |
          Unknown runbook: {{ input.runbook_name }}

          Available runbooks:
          {{ variables.runbooks | keys | join('\n- ') }}

  - name: check-approval
    agent: router
    condition: "{{ variables.runbooks[input.runbook_name].requires_approval }}"
    action: request_approval
    input:
      request_id: "{{ uuid() }}"
      runbook: "{{ input.runbook_name }}"
      requester: "{{ trigger.user.username }}"
      channel: "{{ trigger.channel_id }}"
    response:
      text: |
        🔐 **Approval Required**

        Runbook: `{{ input.runbook_name }}`
        Requested by: @{{ trigger.user.username }}

        Waiting for approval...
      buttons:
        - text: "✅ Approve"
          callback: "/approve {{ steps.check-approval.output.request_id }}"
        - text: "❌ Reject"
          callback: "/reject {{ steps.check-approval.output.request_id }}"
    next: wait-for-approval

  - name: execute-runbook
    agent: shell-executor
    action: run
    input:
      commands: "{{ variables.runbooks[input.runbook_name].steps }}"
      timeout: 300
      env:
        KUBECONFIG: "/etc/kubernetes/admin.conf"
    response:
      text: |
        ✅ **Runbook Completed**

        Runbook: `{{ input.runbook_name }}`
        Duration: {{ steps.execute-runbook.duration }}s

        ```
        {{ steps.execute-runbook.output | truncate(500) }}
        ```

  - name: wait-for-approval
    agent: approval-waiter
    action: wait
    input:
      request_id: "{{ steps.check-approval.output.request_id }}"
      timeout: 1800  # 30 minutes
    on_approved:
      next: execute-runbook
    on_rejected:
      response:
        text: |
          ❌ **Request Rejected**

          Runbook: `{{ input.runbook_name }}`
          Rejected by: @{{ steps.wait-for-approval.output.approver }}
    on_timeout:
      response:
        text: |
          ⏰ **Request Expired**

          Approval request for `{{ input.runbook_name }}` has timed out.

on_error:
  response:
    text: |
      ❌ **Runbook Failed**

      Error: {{ error.message }}

      Check logs for details.
```

### 3.2 Incident Manager Flow

Create `flows/incident-manager-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: incident-manager
  description: Create and manage incidents

triggers:
  - platform: telegram
    type: command
    command: "/incident"
    description: "Manage incidents: /incident <action> [args]"

input:
  required:
    - action
  schema:
    action:
      type: string
      enum: ["create", "update", "resolve", "list"]
    severity:
      type: string
      enum: ["P1", "P2", "P3", "P4"]
    title:
      type: string
    description:
      type: string

steps:
  - name: route-action
    agent: router
    action: switch
    input:
      value: "{{ input.action }}"
    cases:
      create: create-incident
      update: update-incident
      resolve: resolve-incident
      list: list-incidents

  - name: create-incident
    agent: incident-creator
    action: create
    input:
      severity: "{{ input.severity | default('P3') }}"
      title: "{{ input.title }}"
      description: "{{ input.description }}"
      reporter: "{{ trigger.user.username }}"
      channel: "{{ trigger.channel_id }}"
    post_actions:
      - name: notify-oncall
        condition: "{{ input.severity in ['P1', 'P2'] }}"
        agent: pagerduty
        action: trigger
        input:
          service_key: "${PAGERDUTY_SERVICE_KEY}"
          description: "{{ input.title }}"
          severity: "{{ input.severity }}"

      - name: create-war-room
        condition: "{{ input.severity == 'P1' }}"
        agent: telegram
        action: create_group
        input:
          title: "🚨 INC-{{ steps.create-incident.output.id }}"
          users: "{{ oncall.current_team }}"
    response:
      text: |
        🚨 **Incident Created**

        ID: `INC-{{ steps.create-incident.output.id }}`
        Severity: {{ input.severity }}
        Title: {{ input.title }}
        Reporter: @{{ trigger.user.username }}
        Status: Open

        {% if input.severity in ['P1', 'P2'] %}
        📟 On-call has been paged
        {% endif %}

  - name: list-incidents
    agent: incident-lister
    action: list
    input:
      status: "open"
      limit: 10
    response:
      text: |
        📋 **Open Incidents**

        {% for inc in steps.list-incidents.output.incidents %}
        • `{{ inc.id }}` [{{ inc.severity }}] {{ inc.title }}
          Status: {{ inc.status }} | Age: {{ inc.age }}
        {% endfor %}

        {% if steps.list-incidents.output.total > 10 %}
        _Showing 10 of {{ steps.list-incidents.output.total }}_
        {% endif %}

  - name: resolve-incident
    agent: incident-resolver
    action: resolve
    input:
      incident_id: "{{ input.incident_id }}"
      resolution: "{{ input.resolution }}"
      resolver: "{{ trigger.user.username }}"
    response:
      text: |
        ✅ **Incident Resolved**

        ID: `{{ input.incident_id }}`
        Resolution: {{ input.resolution }}
        Resolved by: @{{ trigger.user.username }}
        Duration: {{ steps.resolve-incident.output.duration }}
```

### 3.3 Infrastructure Status Flow

Create `flows/status-checker-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: status-checker
  description: Check infrastructure and service status

triggers:
  - platform: telegram
    type: command
    command: "/status"
    description: "Check status: /status [component]"

input:
  schema:
    component:
      type: string
      default: "all"

variables:
  components:
    api:
      type: kubernetes
      namespace: production
      deployment: api
      health_endpoint: "https://api.example.com/health"

    workers:
      type: kubernetes
      namespace: production
      deployment: workers

    database:
      type: postgres
      host: "db.internal"
      port: 5432

    cache:
      type: redis
      host: "redis.internal"
      port: 6379

    queue:
      type: rabbitmq
      host: "rabbitmq.internal"
      management_port: 15672

steps:
  - name: check-status
    agent: health-checker
    action: check
    parallel: true  # Check all components in parallel
    input:
      components: |
        {% if input.component == 'all' %}
        {{ variables.components }}
        {% else %}
        {{ { input.component: variables.components[input.component] } }}
        {% endif %}
    response:
      text: |
        📊 **Infrastructure Status**

        {% for name, status in steps.check-status.output.items() %}
        {{ '✅' if status.healthy else '❌' }} **{{ name }}**
           Status: {{ status.status }}
           {% if status.replicas %}Replicas: {{ status.ready }}/{{ status.replicas }}{% endif %}
           {% if status.latency %}Latency: {{ status.latency }}ms{% endif %}
           {% if status.error %}Error: {{ status.error }}{% endif %}

        {% endfor %}

        _Last checked: {{ now() | format_time }}_
```

### 3.4 Deployment Flow with Approvals

Create `flows/deployment-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: deployment-flow
  description: Trigger and manage deployments

triggers:
  - platform: telegram
    type: command
    command: "/deploy"
    description: "Deploy: /deploy <service> <version> [environment]"

input:
  required:
    - service
    - version
  schema:
    service:
      type: string
      enum: ["api", "web", "workers", "cron"]
    version:
      type: string
      pattern: "^v?[0-9]+\\.[0-9]+\\.[0-9]+(-[a-z0-9]+)?$"
    environment:
      type: string
      enum: ["staging", "production"]
      default: "staging"

steps:
  - name: validate-deployment
    agent: deployment-validator
    action: validate
    input:
      service: "{{ input.service }}"
      version: "{{ input.version }}"
      environment: "{{ input.environment }}"
    checks:
      - name: version-exists
        type: docker-image
        image: "ghcr.io/myorg/{{ input.service }}:{{ input.version }}"
      - name: tests-passed
        type: github-checks
        repo: "myorg/{{ input.service }}"
        ref: "{{ input.version }}"
      - name: not-already-deployed
        type: kubernetes
        deployment: "{{ input.service }}"
        namespace: "{{ input.environment }}"
        current_version_ne: "{{ input.version }}"

  - name: request-approval
    condition: "{{ input.environment == 'production' }}"
    agent: approval-requester
    action: request
    input:
      type: deployment
      service: "{{ input.service }}"
      version: "{{ input.version }}"
      environment: "{{ input.environment }}"
      requester: "{{ trigger.user.username }}"
      required_approvers: 1
      allowed_approvers: ["@sre-team", "@platform-team"]
    response:
      text: |
        🚀 **Deployment Request**

        Service: `{{ input.service }}`
        Version: `{{ input.version }}`
        Environment: **{{ input.environment }}**
        Requested by: @{{ trigger.user.username }}

        ⏳ Waiting for approval from SRE or Platform team...
      buttons:
        - text: "✅ Approve"
          callback: "/approve deploy:{{ steps.request-approval.output.request_id }}"
        - text: "❌ Reject"
          callback: "/reject deploy:{{ steps.request-approval.output.request_id }}"
        - text: "📋 View Changes"
          url: "https://github.com/myorg/{{ input.service }}/compare/{{ current_version }}...{{ input.version }}"

  - name: execute-deployment
    agent: kubernetes-deployer
    action: deploy
    input:
      service: "{{ input.service }}"
      version: "{{ input.version }}"
      namespace: "{{ input.environment }}"
      strategy: rolling
      max_unavailable: "25%"
      max_surge: "25%"
    progress:
      interval: 10
      message: |
        🔄 Deployment in progress...

        {{ steps.execute-deployment.progress.ready }}/{{ steps.execute-deployment.progress.total }} pods ready
    response:
      text: |
        ✅ **Deployment Complete**

        Service: `{{ input.service }}`
        Version: `{{ input.version }}`
        Environment: {{ input.environment }}
        Duration: {{ steps.execute-deployment.duration }}s

        All {{ steps.execute-deployment.output.replicas }} replicas healthy.

  - name: notify-rollback
    agent: telegram
    trigger: on_error
    action: send
    input:
      channel: "{{ trigger.channel_id }}"
      text: |
        ❌ **Deployment Failed**

        Service: `{{ input.service }}`
        Version: `{{ input.version }}`
        Error: {{ error.message }}

        🔄 Initiating automatic rollback...
      buttons:
        - text: "📋 View Logs"
          url: "https://logs.example.com/deploy/{{ steps.execute-deployment.output.deployment_id }}"

  - name: auto-rollback
    agent: kubernetes-deployer
    trigger: on_error
    action: rollback
    input:
      service: "{{ input.service }}"
      namespace: "{{ input.environment }}"
```

## Step 4: Set Up Webhook

### 4.1 Start the Trigger Server

```bash
# Start AOF trigger server
aofctl trigger serve --config config/telegram-bot.yaml

# Server starts on http://0.0.0.0:8080
```

### 4.2 Expose with HTTPS (for production)

For production, use a reverse proxy with TLS:

```nginx
# /etc/nginx/sites-available/aof-webhook
server {
    listen 443 ssl;
    server_name webhook.example.com;

    ssl_certificate /etc/letsencrypt/live/webhook.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/webhook.example.com/privkey.pem;

    location /webhooks/telegram {
        proxy_pass http://localhost:8080/webhooks/telegram;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 4.3 Register Webhook with Telegram

```bash
# Set webhook URL
curl -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://webhook.example.com/webhooks/telegram",
    "secret_token": "'${TELEGRAM_WEBHOOK_SECRET}'",
    "allowed_updates": ["message", "callback_query"]
  }'

# Verify webhook is set
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo"
```

### 4.4 For Local Development (ngrok)

```bash
# Start ngrok tunnel
ngrok http 8080

# Use the HTTPS URL from ngrok
# Example: https://abc123.ngrok.io

# Register webhook
curl -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -d "url=https://abc123.ngrok.io/webhooks/telegram" \
  -d "secret_token=${TELEGRAM_WEBHOOK_SECRET}"
```

## Step 5: Test Your Bot

### Basic Commands

```
# In Telegram, message your bot:

/help
# Shows available commands

/status
# Shows all infrastructure status

/status api
# Shows API service status only

/runbook clear-cache
# Executes cache clearing runbook

/deploy api v1.2.3 staging
# Triggers staging deployment

/deploy api v1.2.3 production
# Triggers production deployment (requires approval)

/incident create P2 "API latency spike" "Response times >500ms"
# Creates P2 incident
```

### Testing Approvals

1. User A: `/deploy api v1.2.3 production`
2. Bot shows approval buttons
3. User B (in allowed_approvers): Clicks "✅ Approve"
4. Deployment proceeds

## Step 6: Advanced Configuration

### 6.1 Multi-Environment Setup

```yaml
# config/telegram-bot-prod.yaml
platforms:
  telegram:
    bot_token: "${TELEGRAM_BOT_TOKEN_PROD}"
    allowed_chats:
      - "-1001234567890"  # #ops-production channel

    # Production requires stricter controls
    allowed_users:
      - "111111111"  # SRE Lead
      - "222222222"  # Platform Lead

    # Audit all commands
    audit:
      enabled: true
      destination: "elasticsearch://logs.internal:9200/telegram-audit"
```

### 6.2 Rate Limiting

```yaml
platforms:
  telegram:
    rate_limits:
      per_user:
        requests: 10
        window: 60  # 10 requests per minute per user
      per_channel:
        requests: 30
        window: 60

      # Exempt certain users
      exempt_users:
        - "111111111"  # SRE Lead
```

### 6.3 Command Aliases

```yaml
routing:
  aliases:
    "/r": "/runbook"
    "/i": "/incident"
    "/s": "/status"
    "/d": "/deploy"
```

### 6.4 Scheduled Messages

```yaml
# flows/daily-status-report.yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: daily-status-report

triggers:
  - type: schedule
    cron: "0 9 * * 1-5"  # 9 AM weekdays

steps:
  - name: gather-metrics
    agent: metrics-collector
    action: collect
    input:
      sources:
        - prometheus
        - cloudwatch
        - datadog
      timerange: "24h"

  - name: send-report
    agent: telegram
    action: send
    input:
      channel: "-1001234567890"  # #ops channel
      text: |
        📊 **Daily Infrastructure Report**
        _{{ now() | format_date }}_

        **Uptime (24h)**
        • API: {{ metrics.api.uptime }}%
        • Web: {{ metrics.web.uptime }}%
        • Workers: {{ metrics.workers.uptime }}%

        **Incidents**
        • P1: {{ metrics.incidents.p1 }}
        • P2: {{ metrics.incidents.p2 }}
        • P3+: {{ metrics.incidents.other }}

        **Deployments**
        • Successful: {{ metrics.deploys.success }}
        • Failed: {{ metrics.deploys.failed }}

        **Alerts**
        • Critical: {{ metrics.alerts.critical }}
        • Warning: {{ metrics.alerts.warning }}
```

## Troubleshooting

### Bot Not Responding

```bash
# Check webhook status
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo"

# Check for errors
curl "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getUpdates"

# Verify server is receiving requests
tail -f /var/log/aof/trigger-server.log
```

### Permission Denied

- Verify user ID is in `allowed_users`
- Check user is in `allowed_chats` for group commands
- Verify bot has admin rights in group (for some features)

### Webhook SSL Errors

- Telegram requires valid SSL certificate
- Self-signed certificates won't work
- Use Let's Encrypt for free certificates

## Security Best Practices

1. **Always verify webhook secret** - AOF does this automatically
2. **Use allowed_users for sensitive commands** - Restrict who can deploy
3. **Enable audit logging** - Track all commands
4. **Use environment variables** - Never hardcode tokens
5. **Restrict bot to specific chats** - Prevent unauthorized access
6. **Require approvals for production** - Multi-person authorization

## Next Steps

- [WhatsApp Bot Tutorial](./whatsapp-ops-bot.md) - Build WhatsApp integration
- [GitHub Automation Tutorial](./github-automation.md) - PR reviews and CI/CD
- [Multi-Platform Routing](./multi-platform-routing.md) - Route between platforms
- [Custom Agent Development](../developer/building-agents.md) - Build custom agents
