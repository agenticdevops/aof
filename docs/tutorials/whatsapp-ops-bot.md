# Building a WhatsApp Ops Bot with AOF

This tutorial shows you how to build a WhatsApp bot for DevOps and on-call workflows using AOF's trigger system and the WhatsApp Business API.

## What You'll Build

A WhatsApp bot that enables:
- On-call alerts and acknowledgments
- Incident status updates via chat
- Quick command execution for emergencies
- Approval workflows for critical operations
- Team coordination during incidents

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Meta Business Account
- WhatsApp Business API access
- A server with public HTTPS endpoint
- Phone number for WhatsApp Business

## Step 1: Set Up WhatsApp Business API

### 1.1 Create Meta Business Account

1. Go to [Meta Business Suite](https://business.facebook.com)
2. Create or select your business account
3. Navigate to **WhatsApp** > **Getting Started**

### 1.2 Create WhatsApp Business App

1. Go to [Meta Developers](https://developers.facebook.com)
2. Click **Create App** > **Business** > **WhatsApp**
3. Select your business account
4. Note your **App ID** and **App Secret**

### 1.3 Configure WhatsApp Business API

1. In App Dashboard, go to **WhatsApp** > **API Setup**
2. Add a phone number (or use test number)
3. Generate a **Permanent Access Token**:
   - Go to **System Users** in Business Settings
   - Create system user with `whatsapp_business_messaging` permission
   - Generate token

Save these values:
- **Phone Number ID**: `1234567890`
- **Access Token**: `EAAxxxxxxx...`
- **Verify Token**: Create a random string (e.g., `my-verify-token-123`)

### 1.4 Set Up Message Templates

WhatsApp requires pre-approved templates for proactive messages. Create these in Meta Business Manager:

**Template: ops_alert**
```
🚨 *\{\{1\}\} Alert*

Service: \{\{2\}\}
Message: \{\{3\}\}
Time: \{\{4\}\}

Reply with:
• ACK - Acknowledge
• ESC - Escalate
• INFO - More details
```

**Template: deployment_approval**
```
🚀 *Deployment Request*

Service: \{\{1\}\}
Version: \{\{2\}\}
Environment: \{\{3\}\}
Requested by: \{\{4\}\}

Reply:
• APPROVE - Approve deployment
• REJECT - Reject deployment
• DETAILS - View changes
```

**Template: incident_update**
```
📋 *Incident Update*

ID: \{\{1\}\}
Status: \{\{2\}\}
Severity: \{\{3\}\}

\{\{4\}\}

Reply RESOLVE to close.
```

## Step 2: Configure AOF Trigger Server

### 2.1 Create Configuration

Create `config/whatsapp-bot.yaml`:

```yaml
version: v1
kind: TriggerConfig

server:
  host: "0.0.0.0"
  port: 8080
  base_path: "/webhooks"

platforms:
  whatsapp:
    type: whatsapp
    phone_number_id: "${WHATSAPP_PHONE_NUMBER_ID}"
    access_token: "${WHATSAPP_ACCESS_TOKEN}"
    verify_token: "${WHATSAPP_VERIFY_TOKEN}"
    app_secret: "${WHATSAPP_APP_SECRET}"

    # On-call configuration
    oncall:
      # PagerDuty integration for schedule lookup
      pagerduty_token: "${PAGERDUTY_TOKEN}"
      schedule_id: "PXXXXXX"

      # Or static on-call list
      static_oncall:
        - name: "Alice"
          phone: "+1234567890"
          hours: "09:00-18:00"
          timezone: "America/New_York"
        - name: "Bob"
          phone: "+0987654321"
          hours: "18:00-09:00"
          timezone: "America/New_York"

    # Allowed phone numbers (empty = allow all registered)
    allowed_numbers:
      - "+1234567890"  # Alice
      - "+0987654321"  # Bob
      - "+1122334455"  # SRE Team Lead

# Command routing
routing:
  default_flow: "whatsapp-help-flow"

  # Text commands (case-insensitive)
  commands:
    "ACK": "acknowledge-alert-flow"
    "ESC": "escalate-alert-flow"
    "INFO": "alert-info-flow"
    "APPROVE": "approval-handler-flow"
    "REJECT": "approval-handler-flow"
    "RESOLVE": "resolve-incident-flow"
    "STATUS": "status-check-flow"
    "HELP": "whatsapp-help-flow"

  # Keyword routing (for natural language)
  keywords:
    - pattern: "(?i)(deploy|release|ship)"
      flow: "deployment-request-flow"
    - pattern: "(?i)(incident|outage|down)"
      flow: "incident-report-flow"
    - pattern: "(?i)(scale|replicas)"
      flow: "scale-service-flow"

flows:
  directory: "./flows/whatsapp"
  watch: true
```

### 2.2 Environment Variables

```bash
export WHATSAPP_PHONE_NUMBER_ID="1234567890"
export WHATSAPP_ACCESS_TOKEN="EAAxxxxxxx..."
export WHATSAPP_VERIFY_TOKEN="my-verify-token-123"
export WHATSAPP_APP_SECRET="abcdef123456"
export PAGERDUTY_TOKEN="y_NbAkKc66ryYTWUXYEu"
```

## Step 3: Create AgentFlows

### 3.1 Alert Acknowledgment Flow

Create `flows/whatsapp/acknowledge-alert-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: acknowledge-alert
  description: Acknowledge incoming alerts

triggers:
  - platform: whatsapp
    type: message
    pattern: "^ACK$"

context:
  # Get the alert being acknowledged from conversation context
  source: conversation
  lookback: 5  # Look at last 5 messages for alert context

steps:
  - name: find-active-alert
    agent: alert-manager
    action: find_pending
    input:
      user_phone: "\{\{ trigger.user.phone \}\}"
      status: "firing"
    on_empty:
      response:
        template: "ops_notification"
        parameters:
          - "Info"
          - "No pending alerts to acknowledge"

  - name: acknowledge-alert
    agent: alert-manager
    action: acknowledge
    input:
      alert_id: "\{\{ steps.find-active-alert.output.alert_id \}\}"
      acknowledged_by: "\{\{ trigger.user.phone \}\}"
      acknowledged_at: "\{\{ now() \}\}"

  - name: update-pagerduty
    agent: pagerduty
    action: acknowledge
    input:
      incident_id: "\{\{ steps.find-active-alert.output.pagerduty_incident_id \}\}"

  - name: notify-team
    agent: whatsapp
    action: send_to_group
    input:
      group_id: "\{\{ config.oncall.team_group \}\}"
      text: |
        ✅ Alert acknowledged by \{\{ trigger.user.name \}\}

        Alert: \{\{ steps.find-active-alert.output.title \}\}
        Time: \{\{ now() | format_time \}\}

  - name: respond
    agent: whatsapp
    action: reply
    input:
      text: |
        ✅ Alert acknowledged

        *\{\{ steps.find-active-alert.output.title \}\}*

        You're now the incident commander.

        Reply:
        • RESOLVE - When fixed
        • ESC - To escalate
        • STATUS - For current status
```

### 3.2 Escalation Flow

Create `flows/whatsapp/escalate-alert-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: escalate-alert
  description: Escalate alert to next on-call

triggers:
  - platform: whatsapp
    type: message
    pattern: "^ESC$"

steps:
  - name: get-current-alert
    agent: alert-manager
    action: get_active
    input:
      user_phone: "\{\{ trigger.user.phone \}\}"

  - name: get-escalation-target
    agent: oncall-manager
    action: get_next_oncall
    input:
      current_user: "\{\{ trigger.user.phone \}\}"
      schedule_id: "\{\{ config.oncall.schedule_id \}\}"

  - name: escalate-pagerduty
    agent: pagerduty
    action: escalate
    input:
      incident_id: "\{\{ steps.get-current-alert.output.pagerduty_incident_id \}\}"
      escalation_level: "\{\{ steps.get-escalation-target.output.level \}\}"

  - name: notify-escalation-target
    agent: whatsapp
    action: send_template
    input:
      to: "\{\{ steps.get-escalation-target.output.phone \}\}"
      template: "ops_alert"
      parameters:
        - "ESCALATED"
        - "\{\{ steps.get-current-alert.output.service \}\}"
        - "\{\{ steps.get-current-alert.output.message \}\}"
        - "\{\{ now() | format_time \}\}"

  - name: confirm-escalation
    agent: whatsapp
    action: reply
    input:
      text: |
        📤 Alert escalated to \{\{ steps.get-escalation-target.output.name \}\}

        They have been notified and will respond shortly.

        You will be updated when they acknowledge.
```

### 3.3 On-Call Alert Sender Flow

Create `flows/whatsapp/send-oncall-alert-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: send-oncall-alert
  description: Send alert to current on-call engineer

triggers:
  # Triggered by external systems (PagerDuty, Prometheus, etc.)
  - type: webhook
    path: "/alert/oncall"
    method: POST

  # Or by Prometheus Alertmanager
  - type: alertmanager

input:
  required:
    - severity
    - service
    - message
  schema:
    severity:
      type: string
      enum: ["critical", "warning", "info"]
    service:
      type: string
    message:
      type: string
    runbook_url:
      type: string

steps:
  - name: get-oncall
    agent: oncall-manager
    action: get_current
    input:
      schedule_id: "\{\{ config.oncall.schedule_id \}\}"

  - name: create-alert-record
    agent: alert-manager
    action: create
    input:
      severity: "\{\{ input.severity \}\}"
      service: "\{\{ input.service \}\}"
      message: "\{\{ input.message \}\}"
      oncall_user: "\{\{ steps.get-oncall.output.phone \}\}"

  - name: send-whatsapp-alert
    agent: whatsapp
    action: send_template
    input:
      to: "\{\{ steps.get-oncall.output.phone \}\}"
      template: "ops_alert"
      parameters:
        - "\{\{ input.severity | upper \}\}"
        - "\{\{ input.service \}\}"
        - "\{\{ input.message \}\}"
        - "\{\{ now() | format_time \}\}"

  - name: start-escalation-timer
    agent: scheduler
    action: schedule
    input:
      delay: 300  # 5 minutes
      flow: "auto-escalate-flow"
      input:
        alert_id: "\{\{ steps.create-alert-record.output.alert_id \}\}"
    condition: "\{\{ input.severity == 'critical' \}\}"

  - name: log-alert
    agent: logger
    action: log
    input:
      level: "\{\{ input.severity \}\}"
      message: "Alert sent to on-call"
      metadata:
        alert_id: "\{\{ steps.create-alert-record.output.alert_id \}\}"
        oncall: "\{\{ steps.get-oncall.output.name \}\}"
        service: "\{\{ input.service \}\}"
```

### 3.4 Status Check Flow

Create `flows/whatsapp/status-check-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: status-check
  description: Quick status check via WhatsApp

triggers:
  - platform: whatsapp
    type: message
    pattern: "^STATUS\\s*(.*)$"

input:
  schema:
    service:
      type: string
      source: "match.1"
      default: "all"

steps:
  - name: check-services
    agent: health-checker
    action: check
    parallel: true
    input:
      services: |
        {% if input.service == 'all' %}
        ["api", "web", "workers", "database", "cache"]
        {% else %}
        ["\{\{ input.service \}\}"]
        {% endif %}

  - name: get-active-incidents
    agent: incident-manager
    action: list
    input:
      status: "open"
      limit: 3

  - name: format-response
    agent: whatsapp
    action: reply
    input:
      text: |
        📊 *System Status*

        {% for svc in steps.check-services.output %}
        \{\{ '✅' if svc.healthy else '❌' \}\} *\{\{ svc.name \}\}*: \{\{ svc.status \}\}
        {% if svc.latency %}  ⏱ \{\{ svc.latency \}\}ms{% endif %}
        {% endfor %}

        {% if steps.get-active-incidents.output | length > 0 %}
        ━━━━━━━━━━━━
        🚨 *Active Incidents*
        {% for inc in steps.get-active-incidents.output %}
        • \{\{ inc.id \}\}: \{\{ inc.title \}\} [\{\{ inc.severity \}\}]
        {% endfor %}
        {% else %}

        ✨ No active incidents
        {% endif %}

        _\{\{ now() | format_time \}\}_
```

### 3.5 Deployment Approval Flow

Create `flows/whatsapp/deployment-approval-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: deployment-approval-whatsapp
  description: Request deployment approval via WhatsApp

triggers:
  # Triggered by CI/CD pipeline
  - type: webhook
    path: "/deploy/request-approval"
    method: POST

input:
  required:
    - service
    - version
    - environment
    - requester
  schema:
    service:
      type: string
    version:
      type: string
    environment:
      type: string
    requester:
      type: string
    changes_url:
      type: string

steps:
  - name: get-approvers
    agent: approval-manager
    action: get_approvers
    input:
      environment: "\{\{ input.environment \}\}"
      service: "\{\{ input.service \}\}"

  - name: create-approval-request
    agent: approval-manager
    action: create
    input:
      type: "deployment"
      service: "\{\{ input.service \}\}"
      version: "\{\{ input.version \}\}"
      environment: "\{\{ input.environment \}\}"
      requester: "\{\{ input.requester \}\}"
      approvers: "\{\{ steps.get-approvers.output.users \}\}"
      expires_at: "\{\{ now() | add_hours(4) \}\}"

  - name: send-approval-requests
    agent: whatsapp
    action: send_template
    foreach: "\{\{ steps.get-approvers.output.users \}\}"
    input:
      to: "\{\{ item.phone \}\}"
      template: "deployment_approval"
      parameters:
        - "\{\{ input.service \}\}"
        - "\{\{ input.version \}\}"
        - "\{\{ input.environment \}\}"
        - "\{\{ input.requester \}\}"

  - name: notify-requester
    agent: notifier
    action: notify
    input:
      channel: "slack"
      user: "\{\{ input.requester \}\}"
      message: |
        Deployment approval requested via WhatsApp.
        Waiting for: \{\{ steps.get-approvers.output.users | map('name') | join(', ') \}\}
        Request ID: \{\{ steps.create-approval-request.output.request_id \}\}

on_response:
  - pattern: "^APPROVE$"
    flow: "process-approval-flow"
    input:
      request_id: "\{\{ steps.create-approval-request.output.request_id \}\}"
      action: "approve"
      approver_phone: "\{\{ trigger.user.phone \}\}"

  - pattern: "^REJECT$"
    flow: "process-approval-flow"
    input:
      request_id: "\{\{ steps.create-approval-request.output.request_id \}\}"
      action: "reject"
      approver_phone: "\{\{ trigger.user.phone \}\}"
```

### 3.6 Process Approval Flow

Create `flows/whatsapp/process-approval-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: process-approval
  description: Process deployment approval/rejection

input:
  required:
    - request_id
    - action
    - approver_phone

steps:
  - name: validate-approver
    agent: approval-manager
    action: validate_approver
    input:
      request_id: "\{\{ input.request_id \}\}"
      approver_phone: "\{\{ input.approver_phone \}\}"
    on_error:
      response:
        text: "❌ You are not authorized to approve this request."

  - name: get-request
    agent: approval-manager
    action: get
    input:
      request_id: "\{\{ input.request_id \}\}"

  - name: check-expired
    agent: validator
    action: check
    input:
      condition: "\{\{ steps.get-request.output.expires_at > now() \}\}"
    on_error:
      response:
        text: "❌ This approval request has expired."

  - name: process-approval
    agent: approval-manager
    action: "\{\{ input.action \}\}"
    input:
      request_id: "\{\{ input.request_id \}\}"
      approver: "\{\{ input.approver_phone \}\}"
      timestamp: "\{\{ now() \}\}"

  - name: trigger-deployment
    condition: "\{\{ input.action == 'approve' \}\}"
    agent: deployment-manager
    action: deploy
    input:
      service: "\{\{ steps.get-request.output.service \}\}"
      version: "\{\{ steps.get-request.output.version \}\}"
      environment: "\{\{ steps.get-request.output.environment \}\}"
      approved_by: "\{\{ input.approver_phone \}\}"

  - name: notify-requester-approved
    condition: "\{\{ input.action == 'approve' \}\}"
    agent: notifier
    action: notify
    input:
      channels: ["slack", "whatsapp"]
      user: "\{\{ steps.get-request.output.requester \}\}"
      message: |
        ✅ Deployment approved!

        Service: \{\{ steps.get-request.output.service \}\}
        Version: \{\{ steps.get-request.output.version \}\}
        Environment: \{\{ steps.get-request.output.environment \}\}
        Approved by: \{\{ steps.validate-approver.output.approver_name \}\}

        Deployment starting now...

  - name: notify-requester-rejected
    condition: "\{\{ input.action == 'reject' \}\}"
    agent: notifier
    action: notify
    input:
      channels: ["slack", "whatsapp"]
      user: "\{\{ steps.get-request.output.requester \}\}"
      message: |
        ❌ Deployment rejected

        Service: \{\{ steps.get-request.output.service \}\}
        Version: \{\{ steps.get-request.output.version \}\}
        Rejected by: \{\{ steps.validate-approver.output.approver_name \}\}

  - name: respond-to-approver
    agent: whatsapp
    action: reply
    input:
      text: |
        {% if input.action == 'approve' %}
        ✅ Deployment approved

        \{\{ steps.get-request.output.service \}\} v\{\{ steps.get-request.output.version \}\} is now deploying to \{\{ steps.get-request.output.environment \}\}.
        {% else %}
        ❌ Deployment rejected

        \{\{ steps.get-request.output.requester \}\} has been notified.
        {% endif %}
```

## Step 4: Set Up Webhook

### 4.1 Start AOF Server

```bash
aofctl trigger serve --config config/whatsapp-bot.yaml
```

### 4.2 Configure Webhook in Meta

1. Go to Meta Developers > Your App > WhatsApp > Configuration
2. Set Webhook URL: `https://your-domain.com/webhooks/whatsapp`
3. Set Verify Token: Same as `WHATSAPP_VERIFY_TOKEN`
4. Subscribe to fields:
   - `messages`
   - `messaging_postbacks`

### 4.3 Verify Webhook

Meta will send a verification request:

```
GET /webhooks/whatsapp?hub.mode=subscribe&hub.verify_token=your-token&hub.challenge=CHALLENGE
```

AOF automatically handles this verification.

## Step 5: Integrate with Alerting Systems

### 5.1 Prometheus Alertmanager

Add to `alertmanager.yml`:

```yaml
receivers:
  - name: 'whatsapp-oncall'
    webhook_configs:
      - url: 'https://your-domain.com/webhooks/whatsapp/alert/oncall'
        send_resolved: true
        http_config:
          bearer_token: '${AOF_WEBHOOK_TOKEN}'

route:
  receiver: 'whatsapp-oncall'
  routes:
    - match:
        severity: critical
      receiver: 'whatsapp-oncall'
      continue: true
```

### 5.2 PagerDuty Webhook

1. In PagerDuty, go to Services > Your Service > Integrations
2. Add Webhook (Generic V3)
3. URL: `https://your-domain.com/webhooks/whatsapp/pagerduty`
4. Events: Incident triggered, acknowledged, resolved

### 5.3 Datadog Integration

```yaml
# Datadog webhook payload template
{
  "severity": "$ALERT_STATUS",
  "service": "$EVENT_TITLE",
  "message": "$EVENT_MSG",
  "runbook_url": "$RUNBOOK_URL"
}
```

## Step 6: Testing

### 6.1 Send Test Alert

```bash
# Trigger a test alert
curl -X POST https://your-domain.com/webhooks/whatsapp/alert/oncall \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${AOF_WEBHOOK_TOKEN}" \
  -d '{
    "severity": "warning",
    "service": "api-server",
    "message": "High latency detected (>500ms)",
    "runbook_url": "https://runbooks.example.com/api-latency"
  }'
```

### 6.2 Test Approval Flow

```bash
# Request deployment approval
curl -X POST https://your-domain.com/deploy/request-approval \
  -H "Content-Type: application/json" \
  -d '{
    "service": "api",
    "version": "v2.1.0",
    "environment": "production",
    "requester": "alice@example.com",
    "changes_url": "https://github.com/org/api/compare/v2.0.0...v2.1.0"
  }'
```

## Best Practices

### Rate Limiting

WhatsApp has strict rate limits:
- 80 messages/second for business accounts
- Template messages count against monthly quota

```yaml
platforms:
  whatsapp:
    rate_limits:
      messages_per_second: 50
      templates_per_day: 1000

    # Queue high-volume alerts
    queue:
      enabled: true
      batch_size: 10
      batch_delay: 1000  # ms
```

### Message Templates

- Use templates for proactive messages (not replies)
- Templates must be approved by Meta (24-72 hours)
- Keep templates generic - use parameters for specifics
- Test templates in sandbox first

### Security

```yaml
platforms:
  whatsapp:
    security:
      # Verify all webhook signatures
      verify_signatures: true

      # Only accept messages from allowed numbers
      allowed_numbers:
        - "+1234567890"

      # Require phone number verification
      require_verified: true

      # Audit all messages
      audit:
        enabled: true
        include_content: false  # For privacy
```

### Error Handling

```yaml
on_error:
  # Retry transient failures
  retry:
    max_attempts: 3
    backoff: exponential

  # Fallback to other channels
  fallback:
    - channel: slack
      user: "\{\{ trigger.user.slack_id \}\}"
    - channel: email
      address: "\{\{ trigger.user.email \}\}"
```

## Troubleshooting

### Messages Not Delivering

1. Check webhook is registered:
   ```bash
   curl "https://graph.facebook.com/v18.0/${PHONE_NUMBER_ID}/webhooks"
   ```

2. Verify phone number is registered for messaging

3. Check 24-hour window for non-template messages

### Template Rejected

- Review Meta's [message template guidelines](https://developers.facebook.com/docs/whatsapp/message-templates)
- Avoid promotional content
- Include clear opt-out instructions for marketing

### Rate Limited

```bash
# Check current rate limit status
curl "https://graph.facebook.com/v18.0/${PHONE_NUMBER_ID}?fields=messaging_limit_tier"
```

## Next Steps

- [Telegram Bot Tutorial](./telegram-ops-bot.md)
- [GitHub Automation Tutorial](./github-automation.md)
- [Resource Selection Guide](../concepts/resource-selection.md)
- [Trigger Reference](../reference/trigger-spec.md)
