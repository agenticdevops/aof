# AgentFlow YAML Specification

Complete reference for AgentFlow workflow specifications.

## Overview

An AgentFlow is an **event-driven workflow** that orchestrates agents, tools, and integrations in a directed graph. Unlike sequential Workflows, AgentFlows are trigger-based and designed for real-time event handling (Slack bots, webhooks, scheduled jobs).

**Key Differences from Workflow:**
- **Workflow** (`kind: Workflow`): Step-based sequential/parallel execution with entrypoint
- **AgentFlow** (`kind: AgentFlow`): Trigger-based event-driven execution with nodes and connections

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: string              # Required: Unique identifier
  namespace: string         # Optional: Namespace for isolation
  labels:                   # Optional: Key-value labels
    key: value
  annotations:              # Optional: Additional metadata
    key: value

spec:
  trigger:                  # Required: What starts this flow
    type: TriggerType
    config: TriggerConfig

  triggers:                 # Optional: Additional triggers
    - type: TriggerType
      config: TriggerConfig

  context:                  # Optional: Execution context
    kubeconfig: string      # Path to kubeconfig
    namespace: string       # Default K8s namespace
    cluster: string         # Cluster name
    env:                    # Environment variables
      KEY: value
    working_dir: string

  nodes:                    # Required: Flow steps
    - id: string
      type: NodeType
      config:
        agent: string         # For Agent nodes: agent name
        input: string         # Input to pass
        tools: []             # Optional: Override agent tools
        mcp_servers: []       # Optional: MCP server configs
      conditions: []

  connections:              # Required: Node edges
    - from: string
      to: string
      when: string          # Optional: Condition expression

  config:                   # Optional: Global flow config
    default_timeout_seconds: int
    verbose: bool
    retry:
      max_attempts: int
      initial_delay: string
      backoff_multiplier: float
```

## Metadata

### `metadata.name`
**Type:** `string`
**Required:** Yes

Unique identifier for the flow. Used in CLI commands and logs.

### `metadata.labels`
**Type:** `map[string]string`
**Required:** No

Key-value pairs for categorization and filtering.

**Example:**
```yaml
metadata:
  name: slack-k8s-bot-flow
  labels:
    platform: slack
    purpose: operations
    team: sre
```

---

## Trigger Types

Triggers define what starts the flow execution. The `trigger` field is required, and additional triggers can be specified in `triggers` array.

### Slack

Listen for Slack events (mentions, messages, slash commands).

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention           # @bot-name mentions
        - message               # Direct messages
        - slash_command         # /command invocations
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
```

**Available Events:**
- `app_mention` - When someone @mentions your bot
- `message` - Messages in channels where bot is present
- `message.im` - Direct messages to bot
- `slash_command` - Slash command invocations

### Discord

Listen for Discord events.

```yaml
spec:
  trigger:
    type: Discord
    config:
      events:
        - message_create
        - slash_command
      bot_token: ${DISCORD_BOT_TOKEN}
```

### Telegram

Listen for Telegram bot events.

```yaml
spec:
  trigger:
    type: Telegram
    config:
      events:
        - message
        - callback_query
      bot_token: ${TELEGRAM_BOT_TOKEN}
```

### WhatsApp

Listen for WhatsApp Business API events.

```yaml
spec:
  trigger:
    type: WhatsApp
    config:
      events:
        - message
      webhook_verify_token: ${WHATSAPP_VERIFY_TOKEN}
```

### HTTP

Generic HTTP webhook endpoint.

```yaml
spec:
  trigger:
    type: HTTP
    config:
      method: POST
      path: /webhook/alerts
```

**Trigger data available:**
- `${event.method}` - HTTP method
- `${event.path}` - Request path
- `${event.headers}` - Request headers
- `${event.body}` - Request body

### Schedule

Cron-based scheduled execution.

```yaml
spec:
  trigger:
    type: Schedule
    config:
      cron: "0 9 * * *"           # Daily at 9 AM
      timezone: America/New_York  # Optional timezone
```

**Cron Format:**
```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6)
* * * * *
```

**Examples:**
- `0 * * * *` - Every hour
- `0 9 * * 1-5` - Weekdays at 9 AM
- `*/15 * * * *` - Every 15 minutes

### Manual

Triggered explicitly via CLI.

```yaml
spec:
  trigger:
    type: Manual
```

**Usage:**
```bash
aofctl run flow my-flow.yaml --input '{"key": "value"}'
```

---

## Trigger Filtering (Multi-Tenant Routing)

AgentFlows support filtering triggers by channel, user, and message patterns. This enables **multi-tenant bot architecture** where different flows handle different contexts.

### Channel Filtering

Route to flows based on Slack/Discord channel:

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
      channels:                # Only respond in these channels
        - production
        - prod-alerts
        - sre-oncall
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
```

### User Filtering

Restrict flows to specific users:

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
      users:                   # Only respond to these users
        - U012PLATFORM1        # Platform team lead
        - U012PLATFORM2        # SRE team
      bot_token: ${SLACK_BOT_TOKEN}
```

### Pattern Matching

Filter based on message content (regex):

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
      patterns:                # Only match these patterns
        - "^(kubectl|k8s|kubernetes|pod|deploy)"
        - "prod(uction)?"
      bot_token: ${SLACK_BOT_TOKEN}
```

### Combined Filtering

Combine all filters for precise routing:

```yaml
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
        - message
      channels:
        - production
        - staging
      users:
        - U012PLATFORM1
      patterns:
        - "^(kubectl|scale|restart)"
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
```

---

## Execution Context

The `context` field defines the runtime environment for agent execution. This is crucial for multi-cluster setups where different flows connect to different Kubernetes clusters.

### Context Configuration

```yaml
spec:
  context:
    # Kubernetes cluster connection
    kubeconfig: ${KUBECONFIG_PROD:-~/.kube/prod-config}
    namespace: default
    cluster: prod-cluster

    # Environment variables for agent execution
    env:
      ENVIRONMENT: production
      CLUSTER_NAME: prod-cluster
      KUBECTL_READONLY: "true"
      REQUIRE_APPROVAL: "true"

    # Working directory for tool execution
    working_dir: /workspace
```

### Multi-Cluster Example

Different flows for different clusters:

**Production Flow:**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-prod-k8s-bot
  labels:
    environment: production
    cluster: prod-cluster

spec:
  trigger:
    type: Slack
    config:
      channels:
        - production
        - prod-alerts

  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: default
    cluster: prod-cluster
    env:
      KUBECTL_READONLY: "false"
      REQUIRE_APPROVAL: "true"

  nodes:
    # ... nodes ...
```

**Staging Flow:**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-staging-k8s-bot
  labels:
    environment: staging
    cluster: staging-cluster

spec:
  trigger:
    type: Slack
    config:
      channels:
        - staging
        - dev-test

  context:
    kubeconfig: ${KUBECONFIG_STAGING}
    namespace: staging
    cluster: staging-cluster
    env:
      KUBECTL_READONLY: "false"
      REQUIRE_APPROVAL: "false"
      ALLOW_DELETE: "true"

  nodes:
    # ... nodes ...
```

---

## Flow Router

When running with `--flows-dir`, the daemon automatically routes incoming messages to matching flows using the FlowRouter.

### Routing Priority

Messages are matched to flows based on:

1. **Platform** - Slack, WhatsApp, Discord, etc.
2. **Channels** - More specific channel matches win
3. **Users** - User restrictions increase priority
4. **Patterns** - Pattern specificity matters

### Daemon Configuration

```yaml
# daemon-config.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: multi-tenant-bot

spec:
  server:
    port: 3000

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

  agents:
    directory: ./agents/

  # AgentFlow-based routing
  flows:
    directory: ./flows/
    enabled: true

  runtime:
    default_agent: default-bot  # Fallback if no flow matches
```

### CLI Usage

```bash
# Start server with flows directory
aofctl serve --flows-dir ./flows --agents-dir ./agents --port 3000

# Or use config file
aofctl serve --config daemon-config.yaml
```

---

## Node Types

Nodes are the steps in your workflow graph.

### Transform

Data transformation and variable extraction.

```yaml
nodes:
  - id: parse-message
    type: Transform
    config:
      script: |
        # Extract fields from trigger event
        export MESSAGE_TEXT="${event.text}"
        export SLACK_USER="${event.user}"
        export SLACK_CHANNEL="${event.channel}"
        export SLACK_TIMESTAMP="${event.ts}"
```

**Outputs:** Variables exported in the script are available as `${node-id.VARIABLE}`.

### Agent

Execute an AI agent.

```yaml
nodes:
  - id: process-request
    type: Agent
    config:
      agent: my-agent-name      # Required: Agent name
      input: ${MESSAGE_TEXT}    # Input to agent
      context:                  # Optional: Additional context
        channel: ${SLACK_CHANNEL}
        user: ${SLACK_USER}
```

**Outputs:**
- `${node-id.output}` - Agent response
- `${node-id.output.requires_approval}` - If agent requests approval
- `${node-id.output.command}` - Command to execute (if any)

#### Inline Tools and MCP Configuration

Agent nodes support inline tool and MCP server configuration that override or extend the referenced agent's defaults:

```yaml
nodes:
  - id: code-analyzer
    type: Agent
    config:
      agent: base-analyzer
      input: ${MESSAGE_TEXT}

      # Override/extend agent's tools
      tools:
        - git
        - shell

      # Add MCP servers for this node
      mcp_servers:
        - name: filesystem
          transport: stdio
          command: npx
          args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
        - name: github
          transport: stdio
          command: npx
          args: ["-y", "@modelcontextprotocol/server-github"]
          env:
            GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

**Tool Configuration:**
- `tools` - List of tool names available to the agent node
- Tools override the agent's default tools when specified
- See [Built-in Tools](../tools/builtin-tools.md) for available tools

**MCP Server Configuration:**
- `mcp_servers` - List of MCP server configurations
- Each server specifies: `name`, `transport`, `command`, `args`, and optional `env`
- See [MCP Integration](../tools/mcp-integration.md) for configuration details

### Conditional

Evaluate conditions for routing.

```yaml
nodes:
  - id: check-approval
    type: Conditional
    config:
      condition: ${agent-process.output.requires_approval} == true
```

**Expression Syntax:**
```yaml
# Comparisons
${value} == "text"
${number} > 100
${enabled} == true

# Available operators
==, !=, >, <, >=, <=
```

### Slack

Send messages to Slack.

```yaml
nodes:
  - id: send-response
    type: Slack
    config:
      channel: ${SLACK_CHANNEL}
      thread_ts: ${SLACK_TIMESTAMP}    # Reply in thread
      message: ${agent-process.output}
```

**Interactive Elements (wait for reaction):**
```yaml
nodes:
  - id: request-approval
    type: Slack
    config:
      channel: ${SLACK_CHANNEL}
      message: |
        :warning: **Approval Required**

        User: <@${SLACK_USER}>
        Command: `${agent-process.output.command}`

        React with :white_check_mark: to approve or :x: to deny
      wait_for_reaction: true
      timeout_seconds: 300
```

**Block Kit Support:**
```yaml
nodes:
  - id: rich-message
    type: Slack
    config:
      channel: "#channel"
      blocks:
        - type: section
          text:
            type: mrkdwn
            text: "*Status:* ${status}"
        - type: actions
          elements:
            - type: button
              text: "Approve"
              action_id: approve
              value: "${task_id}"
```

### Discord

Send messages to Discord.

```yaml
nodes:
  - id: discord-notify
    type: Discord
    config:
      channel_id: ${DISCORD_CHANNEL}
      message: ${output}
```

### HTTP

Make HTTP requests.

```yaml
nodes:
  - id: call-api
    type: HTTP
    config:
      method: POST
      url: https://api.example.com/endpoint
      headers:
        Content-Type: application/json
        Authorization: "Bearer ${API_TOKEN}"
      body:
        event: ${event.type}
        data: ${event.data}
      timeout_seconds: 30
```

### Wait

Pause execution for a duration.

```yaml
nodes:
  - id: cooldown
    type: Wait
    config:
      duration: "30s"    # Supports: 30s, 5m, 1h
```

### Parallel

Execute multiple branches in parallel.

```yaml
nodes:
  - id: parallel-checks
    type: Parallel
    config:
      branches:
        - check-logs
        - check-metrics
        - check-events
```

### Join

Wait for parallel branches to complete.

```yaml
nodes:
  - id: aggregate
    type: Join
    config:
      strategy: all      # all | any | majority
```

### Approval

Human approval gate.

```yaml
nodes:
  - id: await-approval
    type: Approval
    config:
      approvers:
        - user@company.com
        - oncall-team
      timeout_seconds: 1800
```

### End

Terminal node (marks flow completion).

```yaml
nodes:
  - id: complete
    type: End
```

---

## Connections

Define how nodes connect (graph edges).

```yaml
connections:
  # Simple connection
  - from: trigger
    to: parse-message

  # Sequential flow
  - from: parse-message
    to: agent-process

  # Conditional routing
  - from: check-approval
    to: request-approval
    when: requires_approval == true

  - from: check-approval
    to: send-response
    when: requires_approval == false

  # After approval, execute
  - from: request-approval
    to: execute-command
```

**Note:** The special node ID `trigger` represents the flow trigger entry point.

---

## Node Conditions

Control when nodes execute based on previous node outputs.

```yaml
nodes:
  - id: execute-command
    type: Agent
    config:
      agent: executor
      input: "Execute: ${command}"
    conditions:
      - from: request-approval
        reaction: white_check_mark    # Wait for this reaction
```

**Condition Types:**

```yaml
# Wait for specific value
conditions:
  - from: conditional-node
    value: true

# Wait for Slack reaction
conditions:
  - from: slack-node
    reaction: white_check_mark
```

---

## Variable Interpolation

Access data from triggers, nodes, and environment.

### Trigger Data
```yaml
${event.text}           # Message text
${event.user}           # User ID
${event.channel}        # Channel ID
${event.ts}             # Timestamp
${event.type}           # Event type
```

### Node Outputs
```yaml
${node-id.output}                    # Full output
${node-id.output.field}              # Nested field
${node-id.EXPORTED_VAR}              # Transform exports
```

### Environment Variables
```yaml
${SLACK_BOT_TOKEN}      # Env var (resolved at runtime)
${env.HOME}             # Explicit env var
```

---

## Flow Configuration

Global configuration for the flow.

```yaml
spec:
  config:
    default_timeout_seconds: 60
    verbose: true
    retry:
      max_attempts: 3
      initial_delay: "1s"
      backoff_multiplier: 2.0
    error_handler: error-node    # Node to handle errors
```

---

## Complete Examples

### Slack Bot with Approval Flow

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-bot-flow
  labels:
    platform: slack
    purpose: operations

spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
        - message
        - slash_command
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  nodes:
    - id: parse-message
      type: Transform
      config:
        script: |
          export MESSAGE_TEXT="${event.text}"
          export SLACK_USER="${event.user}"
          export SLACK_CHANNEL="${event.channel}"
          export SLACK_TIMESTAMP="${event.ts}"

    - id: agent-process
      type: Agent
      config:
        agent: slack-k8s-bot
        input: ${MESSAGE_TEXT}
        context:
          slack_channel: ${SLACK_CHANNEL}
          slack_user: ${SLACK_USER}

    - id: check-approval
      type: Conditional
      config:
        condition: ${agent-process.output.requires_approval} == true

    - id: request-approval
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        thread_ts: ${SLACK_TIMESTAMP}
        message: |
          :warning: **Approval Required**

          User: <@${SLACK_USER}>
          Command: `${agent-process.output.command}`

          React with :white_check_mark: to approve
        wait_for_reaction: true
        timeout_seconds: 300

    - id: send-response
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        thread_ts: ${SLACK_TIMESTAMP}
        message: ${agent-process.output.output}

    - id: execute-command
      type: Agent
      config:
        agent: slack-k8s-bot
        input: "Execute: ${agent-process.output.command}"
      conditions:
        - from: request-approval
          reaction: white_check_mark

    - id: send-result
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        thread_ts: ${SLACK_TIMESTAMP}
        message: |
          :white_check_mark: **Executed**

          ${execute-command.output.output}

  connections:
    - from: trigger
      to: parse-message
    - from: parse-message
      to: agent-process
    - from: agent-process
      to: check-approval
    - from: check-approval
      to: request-approval
      when: requires_approval == true
    - from: check-approval
      to: send-response
      when: requires_approval == false
    - from: request-approval
      to: execute-command
    - from: execute-command
      to: send-result

  config:
    default_timeout_seconds: 60
    verbose: true
    retry:
      max_attempts: 2
      initial_delay: "1s"
      backoff_multiplier: 2.0
```

### Scheduled Daily Report

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: daily-cluster-report

spec:
  trigger:
    type: Schedule
    config:
      cron: "0 9 * * *"
      timezone: America/New_York

  nodes:
    - id: generate-report
      type: Agent
      config:
        agent: report-generator
        input: |
          Generate a daily cluster health report:
          - Total pods and their status
          - Any failing deployments
          - Resource usage summary

    - id: send-to-slack
      type: Slack
      config:
        channel: "#platform-daily"
        message: |
          :chart_with_upwards_trend: **Daily Cluster Report**

          ${generate-report.output}

  connections:
    - from: trigger
      to: generate-report
    - from: generate-report
      to: send-to-slack
```

### HTTP Webhook Handler

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: alert-handler

spec:
  trigger:
    type: HTTP
    config:
      method: POST
      path: /alerts

  nodes:
    - id: parse-alert
      type: Transform
      config:
        script: |
          export ALERT_NAME="${event.body.alertname}"
          export SEVERITY="${event.body.severity}"
          export DESCRIPTION="${event.body.description}"

    - id: analyze
      type: Agent
      config:
        agent: alert-analyzer
        input: |
          Alert: ${ALERT_NAME}
          Severity: ${SEVERITY}
          Description: ${DESCRIPTION}

    - id: notify
      type: Slack
      config:
        channel: "#alerts"
        message: |
          :rotating_light: **Alert: ${ALERT_NAME}**

          ${analyze.output}

  connections:
    - from: trigger
      to: parse-alert
    - from: parse-alert
      to: analyze
    - from: analyze
      to: notify
```

---

## CLI Commands

### Describe Flow
```bash
aofctl describe flow my-flow.yaml
```

### Run Flow
```bash
# Run with manual trigger
aofctl run flow my-flow.yaml

# Run with input data
aofctl run flow my-flow.yaml --input '{"event": {"text": "hello"}}'

# Run with JSON output
aofctl run flow my-flow.yaml --output json
```

### Serve Flow (HTTP/Webhook Mode)
```bash
# Start server for webhook triggers
aofctl serve --port 3000 --config my-flow.yaml
```

---

## Best Practices

### Flow Design
- Keep flows focused on a single purpose
- Use meaningful node IDs (`parse-message` not `step1`)
- Add conditions for error handling paths
- Set appropriate timeouts

### Error Handling
```yaml
nodes:
  - id: error-handler
    type: Slack
    config:
      channel: "#errors"
      message: "Flow failed: ${error.message}"

spec:
  config:
    error_handler: error-handler
```

### Security
- Never hardcode tokens in YAML files
- Use environment variable references: `${SLACK_BOT_TOKEN}`
- Set appropriate timeouts to prevent hanging flows

### Testing
```bash
# Test with mock input
aofctl run flow my-flow.yaml --input '{"event": {"text": "test", "user": "U123", "channel": "C123"}}'
```

---

## See Also

- [Agent Spec](agent-spec.md) - Agent configuration reference
- [Fleet Spec](fleet-spec.md) - Multi-agent fleet configuration
- [aofctl CLI](aofctl.md) - CLI command reference
- [Slack Bot Tutorial](../tutorials/slack-bot.md) - Step-by-step tutorial
