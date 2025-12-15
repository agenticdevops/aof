# AgentFlow YAML Specification

Complete reference for AgentFlow workflow specifications.

## Overview

An AgentFlow is a workflow that orchestrates agents, tools, and integrations in a directed acyclic graph (DAG). Think of it as n8n or Argo Workflows for AI agents.

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value

spec:
  trigger:                  # Required: What starts this flow
    type: string
    config: object

  nodes:                    # Required: Flow steps
    - id: string
      type: string
      config: object
      conditions: array

  connections:              # Optional: Explicit edges
    - from: string
      to: string
      when: string

  variables:                # Optional: Flow-level variables
    key: value

  timeout_seconds: int      # Optional: Overall timeout
```

## Metadata

### `metadata.name`
**Type:** `string`
**Required:** Yes

**Example:**
```yaml
metadata:
  name: incident-response
  labels:
    team: sre
    env: production
```

## Trigger Types

Triggers define what starts the flow execution.

### Webhook

HTTP endpoint that receives requests.

```yaml
spec:
  trigger:
    type: Webhook
    config:
      path: /my-webhook              # URL path
      methods: [POST, PUT]            # Allowed HTTP methods
      auth:                           # Optional authentication
        type: Bearer
        token: ${WEBHOOK_TOKEN}
```

**Usage:**
```bash
curl -X POST https://your-domain.com/my-webhook \
  -H "Authorization: Bearer token" \
  -d '{"data": "value"}'
```

### Schedule

Cron-based scheduling.

```yaml
spec:
  trigger:
    type: Schedule
    config:
      cron: "0 9 * * *"               # Daily at 9 AM
      timezone: America/New_York      # Optional timezone
```

**Cron Format:**
```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday=0)
│ │ │ │ │
* * * * *
```

**Examples:**
- `0 * * * *` - Every hour
- `0 0 * * *` - Daily at midnight
- `0 9 * * 1-5` - Weekdays at 9 AM
- `*/15 * * * *` - Every 15 minutes

### FileWatch

Monitor file changes.

```yaml
spec:
  trigger:
    type: FileWatch
    config:
      paths:
        - /etc/kubernetes/config.yaml
        - /tmp/deployments/*.yaml
      events: [created, modified, deleted]
      debounce_seconds: 5           # Wait 5s for batch changes
```

### Manual

Triggered explicitly via CLI.

```yaml
spec:
  trigger:
    type: Manual
    config:
      require_approval: true        # Optional approval gate
```

**Usage:**
```bash
aofctl flow run my-flow
```

### Slack

Slack events trigger the flow.

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention              # @bot-name
        - message                  # Direct messages
        - slash_command            # /command
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
```

### GitHub

GitHub webhook events.

```yaml
spec:
  trigger:
    type: GitHub
    config:
      events:
        - pull_request            # PR events
        - issues                  # Issue events
        - push                    # Push events
      repositories:
        - owner/repo1
        - owner/repo2
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
```

### PagerDuty

PagerDuty incident events.

```yaml
spec:
  trigger:
    type: PagerDuty
    config:
      events:
        - incident.triggered
        - incident.acknowledged
      webhook_token: ${PAGERDUTY_WEBHOOK_TOKEN}
```

### Kafka

Kafka message consumption.

```yaml
spec:
  trigger:
    type: Kafka
    config:
      brokers:
        - kafka1.company.com:9092
        - kafka2.company.com:9092
      topic: incidents
      consumer_group: aof-flows
      auth:
        type: SASL
        username: ${KAFKA_USERNAME}
        password: ${KAFKA_PASSWORD}
```

---

## Node Types

Nodes are the steps in your workflow.

### Agent Node

Run an AI agent.

```yaml
nodes:
  - id: diagnose
    type: Agent
    config:
      agent: diagnostic-agent       # Agent name
      input: ${trigger.data}        # Input data
      timeout_seconds: 180          # Max execution time
      context:                      # Additional context
        namespace: ${trigger.namespace}
```

**Outputs:**
- `${diagnose.output}` - Agent response
- `${diagnose.status}` - success/failed
- `${diagnose.duration}` - Execution time

### Fleet Node

Run an agent fleet (team of agents).

```yaml
nodes:
  - id: review-team
    type: Fleet
    config:
      fleet: code-review-team       # Fleet name
      input: ${code-changes}
      aggregation: consensus        # How to combine results
```

**Aggregation Methods:**
- `all` - Return all responses
- `consensus` - Majority vote
- `summary` - Summarized by meta-agent
- `first` - First successful response

### HTTP Node

Make HTTP requests.

```yaml
nodes:
  - id: notify-api
    type: HTTP
    config:
      method: POST
      url: https://api.company.com/notify
      headers:
        Content-Type: application/json
        Authorization: "Bearer ${API_TOKEN}"
      body: |
        {
          "event": "${event.type}",
          "data": ${event.data}
        }
      timeout_seconds: 30
```

### Shell Node

Execute commands.

```yaml
nodes:
  - id: backup-db
    type: Shell
    config:
      command: kubectl
      args:
        - exec
        - postgres-0
        - --
        - pg_dump
        - mydb
      working_directory: /tmp
      timeout_seconds: 300
      capture_output: true
```

### Slack Node

Send Slack messages.

```yaml
nodes:
  - id: notify-team
    type: Slack
    config:
      channel: "#incidents"
      thread_ts: ${trigger.ts}      # Reply in thread
      message: |
        🚨 **Incident Alert**

        ${diagnose.output}
      blocks:                       # Rich formatting
        - type: section
          text:
            type: mrkdwn
            text: "*Status:* ${status}"
```

**Interactive Elements:**
```yaml
- id: request-approval
  type: Slack
  config:
    channel: "#approvals"
    message: "Approve deployment?"
    wait_for_reaction: true
    reactions: [white_check_mark, x]
    timeout_seconds: 300
```

### GitHub Node

GitHub operations.

```yaml
nodes:
  - id: create-pr
    type: GitHub
    config:
      action: create_pull_request
      repository: owner/repo
      base: main
      head: feature-branch
      title: ${pr-title}
      body: ${pr-description}
```

**Available Actions:**
- `create_pull_request`
- `add_comment`
- `create_issue`
- `update_status`
- `merge_pull_request`

### Conditional Node

If/else logic.

```yaml
nodes:
  - id: check-severity
    type: Conditional
    config:
      conditions:
        - name: is_critical
          expression: ${severity} == "critical"
        - name: is_high
          expression: ${severity} == "high"
        - name: is_normal
          expression: true  # Default case
```

**Expression Syntax:**
```yaml
# Comparisons
${value} == "text"
${number} > 100
${enabled} == true

# Logical operators
${a} == true AND ${b} == false
${x} > 10 OR ${y} < 5

# String operations
${text} contains "error"
${name} startsWith "prod-"
```

### Transform Node

Data transformation.

```yaml
nodes:
  - id: parse-data
    type: Transform
    config:
      script: |
        # Extract fields
        export SEVERITY="${event.severity}"
        export NAMESPACE="${event.namespace}"

        # Transform
        export PRIORITY=$([[ "$SEVERITY" == "critical" ]] && echo "P1" || echo "P2")

        # Format output
        cat > output.json <<EOF
        {
          "priority": "$PRIORITY",
          "namespace": "$NAMESPACE"
        }
        EOF
```

**Outputs:**
Variables exported in the script are available as `${transform-node.VARIABLE}`.

### HumanApproval Node

Wait for human approval.

```yaml
nodes:
  - id: await-approval
    type: HumanApproval
    config:
      approvers:
        - user1@company.com
        - user2@company.com
      require_count: 1              # At least 1 approval
      timeout_seconds: 1800         # 30 minutes
      notification:
        type: Slack
        channel: "#approvals"
        message: "Please approve: ${action}"
```

### Parallel Node

Execute multiple nodes in parallel.

```yaml
nodes:
  - id: parallel-checks
    type: Parallel
    config:
      nodes:
        - id: check-logs
          type: Agent
          config:
            agent: log-analyzer

        - id: check-metrics
          type: Agent
          config:
            agent: metrics-analyzer

        - id: check-events
          type: Shell
          config:
            command: kubectl get events
```

---

## Connections

Define how nodes connect (optional, inferred from conditions if not specified).

```yaml
connections:
  - from: parse-alert
    to: diagnose

  - from: diagnose
    to: remediate
    when: ${severity} != "critical"

  - from: diagnose
    to: request-approval
    when: ${severity} == "critical"

  - from: request-approval
    to: remediate
```

---

## Conditions

Control when nodes execute.

```yaml
nodes:
  - id: auto-fix
    type: Agent
    config:
      agent: remediation-agent
    conditions:
      - from: check-severity
        when: severity != "critical"
```

**Condition Types:**

```yaml
# Simple condition
conditions:
  - from: previous-node
    when: ${output.success} == true

# Multiple conditions (AND)
conditions:
  - from: node1
    when: ${approved} == true
  - from: node2
    when: ${validated} == true

# Value matching
conditions:
  - from: conditional-node
    value: is_critical  # Match condition name
```

---

## Variables

Flow-level variables accessible to all nodes.

```yaml
spec:
  variables:
    NAMESPACE: production
    CLUSTER: us-east-1
    ALERT_CHANNEL: "#incidents"

  nodes:
    - id: notify
      type: Slack
      config:
        channel: ${ALERT_CHANNEL}
```

---

## Variable Interpolation

Access data from triggers, nodes, and variables.

### Trigger Data

```yaml
${trigger.data}               # Full trigger payload
${trigger.event.type}         # Nested field
${trigger.user}               # User who triggered
```

### Node Outputs

```yaml
${node-id.output}             # Node output
${node-id.status}             # success/failed
${node-id.duration}           # Execution time in seconds
${node-id.custom-field}       # Custom output field
```

### Flow Metadata

```yaml
${flow.id}                    # Flow execution ID
${flow.name}                  # Flow name
${flow.started_at}            # Start timestamp
${flow.duration_seconds}      # Current duration
```

### Environment Variables

```yaml
${NAMESPACE}                  # Flow variable
${env.HOME}                   # Environment variable
```

---

## Complete Examples

### Webhook → Agent → Slack

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: simple-alert

spec:
  trigger:
    type: Webhook
    config:
      path: /alerts

  nodes:
    - id: analyze
      type: Agent
      config:
        agent: alert-analyzer
        input: ${trigger.data}

    - id: notify
      type: Slack
      config:
        channel: "#alerts"
        message: ${analyze.output}

  connections:
    - from: analyze
      to: notify
```

### Scheduled Report

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: daily-report

spec:
  trigger:
    type: Schedule
    config:
      cron: "0 9 * * *"
      timezone: America/New_York

  nodes:
    - id: gather-metrics
      type: Shell
      config:
        command: kubectl
        args: [top, pods, --all-namespaces]

    - id: generate-report
      type: Agent
      config:
        agent: report-generator
        input: ${gather-metrics.output}

    - id: send-report
      type: Slack
      config:
        channel: "#daily-reports"
        message: |
          📊 **Daily Cluster Report**

          ${generate-report.output}
```

### Conditional Remediation

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: auto-remediation

spec:
  trigger:
    type: PagerDuty
    config:
      events: [incident.triggered]

  nodes:
    - id: diagnose
      type: Agent
      config:
        agent: diagnostic-agent
        input: ${trigger.incident.title}

    - id: check-severity
      type: Conditional
      config:
        conditions:
          - name: critical
            expression: ${diagnose.output.severity} == "critical"
          - name: normal
            expression: true

    - id: request-approval
      type: HumanApproval
      config:
        approvers: [oncall@company.com]
        timeout_seconds: 600
      conditions:
        - from: check-severity
          value: critical

    - id: remediate
      type: Agent
      config:
        agent: remediation-agent
        input: ${diagnose.output.recommended_action}

    - id: verify
      type: Agent
      config:
        agent: diagnostic-agent
        input: "Verify the fix worked"

    - id: notify-success
      type: Slack
      config:
        channel: "#incidents"
        message: "✅ Auto-resolved: ${diagnose.output.root_cause}"
```

### Parallel Processing

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: parallel-analysis

spec:
  trigger:
    type: GitHub
    config:
      events: [pull_request]

  nodes:
    - id: parallel-reviews
      type: Parallel
      config:
        nodes:
          - id: security-scan
            type: Agent
            config:
              agent: security-reviewer

          - id: performance-check
            type: Agent
            config:
              agent: performance-reviewer

          - id: style-check
            type: Agent
            config:
              agent: style-reviewer

    - id: aggregate
      type: Agent
      config:
        agent: summary-agent
        input: |
          Security: ${parallel-reviews.security-scan.output}
          Performance: ${parallel-reviews.performance-check.output}
          Style: ${parallel-reviews.style-check.output}

    - id: post-comment
      type: GitHub
      config:
        action: add_comment
        issue_number: ${trigger.pull_request.number}
        body: ${aggregate.output}
```

---

## Best Practices

### Flow Design
- ✅ Keep flows simple and focused
- ✅ Use meaningful node IDs
- ✅ Add conditions for error handling
- ❌ Don't create circular dependencies
- ❌ Don't make flows too complex (>20 nodes)

### Error Handling
```yaml
nodes:
  - id: risky-operation
    type: Agent
    config:
      agent: my-agent

  - id: on-error
    type: Slack
    config:
      channel: "#errors"
      message: "Operation failed: ${risky-operation.error}"
    conditions:
      - from: risky-operation
        when: ${status} == "failed"
```

### Timeouts
Always set timeouts to prevent hanging flows:

```yaml
spec:
  timeout_seconds: 3600  # Overall flow timeout

  nodes:
    - id: agent-task
      config:
        timeout_seconds: 180  # Per-node timeout
```

### Idempotency
Design flows to be safely re-runnable:

```yaml
nodes:
  - id: check-exists
    type: Shell
    config:
      command: kubectl get deployment my-app

  - id: create-only-if-missing
    type: Shell
    config:
      command: kubectl apply -f deployment.yaml
    conditions:
      - from: check-exists
        when: ${status} == "failed"
```

---

## Debugging

### View Flow Logs
```bash
aofctl flow logs my-flow -f
```

### Get Flow Status
```bash
aofctl flow describe my-flow
```

### Visualize Flow
```bash
aofctl flow visualize my-flow > flow.dot
dot -Tpng flow.dot > flow.png
```

### Dry Run
```bash
aofctl flow run my-flow --dry-run
```

---

## See Also

- [Agent Spec](agent-spec.md)
- [aofctl CLI](aofctl.md)
- [Examples](../examples/)
