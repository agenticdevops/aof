# AgentFlow YAML Specification

Complete reference for AgentFlow workflow specifications.

## Overview

An AgentFlow is a **multi-step workflow** that orchestrates agents, tools, and integrations in a directed graph. Flows define the sequence of operations with nodes and connections.

**Key Characteristics:**
- **Flows are pure workflows** - no embedded triggers
- **Triggers reference flows** - via command bindings in Trigger CRDs
- **Entry point is "start"** - connections begin from the "start" node
- **Nodes execute sequentially or conditionally** - based on connections

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
  description: string       # Optional: Human-readable description

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
        fleet: string         # For Fleet nodes: fleet name
        input: string         # Input to pass
        tools: []             # Optional: Override agent tools
        mcp_servers: []       # Optional: MCP server configs
      conditions: []

  connections:              # Required: Node edges
    - from: string          # Source node (or "start")
      to: string            # Target node
      condition: string     # Optional: Condition expression

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

Unique identifier for the flow. Used in CLI commands and referenced by Trigger command bindings.

### `metadata.labels`
**Type:** `map[string]string`
**Required:** No

Key-value pairs for categorization and filtering.

**Example:**
```yaml
metadata:
  name: deploy-flow
  labels:
    purpose: deployment
    team: platform
    environment: production
```

---

## Spec Fields

### `spec.description`
**Type:** `string`
**Required:** No

Human-readable description of what this flow does.

```yaml
spec:
  description: "Production deployment workflow with approval gates"
```

### `spec.context`
**Type:** `object`
**Required:** No

Execution context for the flow including environment variables and Kubernetes configuration.

```yaml
spec:
  context:
    kubeconfig: ~/.kube/config
    namespace: production
    cluster: prod-cluster
    env:
      ENVIRONMENT: production
      LOG_LEVEL: info
    working_dir: /app
```

---

## Node Types

Nodes define what happens at each step of the flow.

### Agent Node

Execute a single agent. Agent nodes support two configuration methods:

#### Option 1: Reference External Agent

Reference an agent defined in a separate YAML file:

```yaml
nodes:
  - id: analyze
    type: Agent
    config:
      agent: k8s-agent           # Agent name or path
      input: "${event.text}"     # Input from trigger event
      timeout_seconds: 120
```

#### Option 2: Inline Agent Definition

Define the agent directly in the flow (recommended for simple, single-use agents):

```yaml
nodes:
  - id: check-status
    type: Agent
    config:
      inline:
        name: status-checker
        model: google:gemini-2.5-flash
        instructions: |
          Check the system status and report any issues.
          Format your response as a summary.
        tools:
          - shell
        temperature: 0.1
      input: "${event.text}"
```

**Inline Agent Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Agent identifier |
| `model` | string | Yes | Model to use (e.g., `google:gemini-2.5-flash`) |
| `instructions` | string | No | System prompt / instructions |
| `tools` | list | No | Tools available to the agent |
| `mcp_servers` | list | No | MCP server configurations |
| `temperature` | float | No | Model temperature (default: 0.7) |
| `max_tokens` | int | No | Maximum response tokens |

**When to use each approach:**

| Use Case | Recommended |
|----------|-------------|
| Reusable agent across multiple flows | External agent file |
| Complex agent with many tools | External agent file |
| Simple, single-purpose step | Inline definition |
| Self-contained flow (no external deps) | Inline definition |

### Fleet Node

Execute a fleet of agents (multi-agent coordination).

```yaml
nodes:
  - id: diagnose
    type: Fleet
    config:
      fleet: rca-fleet           # Fleet name or path
      input: "${event.text}"
```

### Script Node

Execute shell commands or native tools **without LLM involvement**. Script nodes are ideal for deterministic operations that don't require intelligence, such as:

- Running shell commands (like Jenkins pipelines)
- Executing docker, kubectl, or other CLI tools
- Parsing logs or data files
- Making HTTP requests
- File operations

Script nodes are faster and more predictable than Agent nodes since they don't use LLM tokens.

#### Option 1: Shell Command Execution

```yaml
nodes:
  - id: check-containers
    type: Script
    config:
      scriptConfig:
        command: docker ps -a --format "{{json .}}"
        parse: json              # Parse output as JSON
        timeout_seconds: 30
        fail_on_error: true      # Fail flow if command fails
```

#### Option 2: Native Tool Execution

Use built-in native tools for common operations:

```yaml
nodes:
  - id: get-pods
    type: Script
    config:
      scriptConfig:
        tool: kubectl            # Built-in tool
        action: get              # Tool action
        args:
          resource: pods
          namespace: production
```

**Available Native Tools:**

| Tool | Actions | Description |
|------|---------|-------------|
| `docker` | `ps`, `logs`, `inspect`, `stats` | Container operations |
| `kubectl` | `get`, `logs`, `describe` | Kubernetes operations |
| `http` | `get`, `post`, `put`, `delete` | HTTP requests |
| `json` | `parse`, `extract`, `merge` | JSON transformation |
| `file` | `read`, `write`, `exists`, `list` | File operations |

**Script Configuration Fields:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `command` | string | * | - | Shell command to execute |
| `tool` | string | * | - | Native tool to use |
| `action` | string | No | "run" | Tool action |
| `args` | object | No | {} | Arguments for tool |
| `working_dir` | string | No | - | Working directory |
| `env` | object | No | {} | Environment variables |
| `timeout_seconds` | int | No | 60 | Execution timeout |
| `parse` | string | No | "text" | Output parsing mode |
| `pattern` | string | No | - | Regex pattern (for parse: regex) |
| `fail_on_error` | bool | No | true | Fail on non-zero exit |

\* Either `command` or `tool` is required.

**Output Parsing Modes:**

| Mode | Description |
|------|-------------|
| `text` | Raw text output (default) |
| `json` | Parse as JSON object |
| `lines` | Split into array of lines |
| `regex` | Apply regex pattern with named groups |

**Example: Docker Diagnostics with Script Nodes**

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: docker-diagnostics
spec:
  description: "Docker health check using Script nodes"

  nodes:
    # Native tool: Get container status
    - id: check-status
      type: Script
      config:
        scriptConfig:
          tool: docker
          action: ps
          args:
            all: true

    # Shell command: Get logs
    - id: get-logs
      type: Script
      config:
        scriptConfig:
          command: |
            docker ps -a --filter "status=exited" --format "{{.Names}}" | \
            while read name; do
              echo "=== $name ==="
              docker logs --tail 20 "$name" 2>&1
            done
          parse: text

    # LLM analysis of Script outputs
    - id: analyze
      type: Agent
      config:
        inline:
          name: analyzer
          model: google:gemini-2.5-flash
          instructions: "Analyze container diagnostics and provide recommendations."
        input: |
          Status: ${check-status.output}
          Logs: ${get-logs.output}

  connections:
    - from: start
      to: check-status
    - from: check-status
      to: get-logs
    - from: get-logs
      to: analyze
```

**When to use Script vs Agent:**

| Use Case | Recommended |
|----------|-------------|
| Running CLI commands | Script |
| Fetching data from APIs | Script |
| Parsing logs or files | Script |
| Intelligent analysis | Agent |
| Decision making | Agent |
| Natural language generation | Agent |

### HumanApproval Node

Wait for human approval before proceeding.

```yaml
nodes:
  - id: approval
    type: HumanApproval
    config:
      message: "Approve deployment to production?"
      timeout_seconds: 300       # 5 minute timeout
      fallback: reject           # Action if timeout: reject | approve
```

### Conditional Node

Branch based on conditions.

```yaml
nodes:
  - id: check-env
    type: Conditional
    config:
      expression: "${context.env} == 'production'"
      true_target: production-deploy
      false_target: staging-deploy
```

### End Node

Terminal node that produces final output.

```yaml
nodes:
  - id: complete
    type: End
    config:
      message: "${previous.output}"
```

### Response Node

Send response to the triggering platform (Slack, Telegram, etc.).

```yaml
nodes:
  - id: notify
    type: Response
    config:
      message: "${analyze.output}"
      format: markdown           # text | markdown | blocks
```

---

## Connections

Connections define the flow between nodes. Every flow starts from the implicit "start" node.

### Basic Connection

```yaml
connections:
  - from: start
    to: validate
  - from: validate
    to: deploy
  - from: deploy
    to: notify
```

### Conditional Connection

```yaml
connections:
  - from: approval
    to: deploy
    condition: approved          # Only if approved
  - from: approval
    to: notify-rejected
    condition: rejected          # Only if rejected
```

### Multiple Outputs

A node can connect to multiple targets based on conditions:

```yaml
connections:
  - from: analyze
    to: critical-path
    condition: severity == "critical"
  - from: analyze
    to: normal-path
    condition: severity != "critical"
```

---

## Complete Examples

### Deployment Flow with Approval

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: deploy-flow
  labels:
    purpose: deployment
spec:
  description: "Production deployment with approval gate"

  nodes:
    - id: validate
      type: Agent
      config:
        agent: validator-agent
        input: "Validate deployment: ${event.text}"

    - id: approval
      type: HumanApproval
      config:
        message: |
          Deployment validated. Approve deployment?
          Version: ${validate.output.version}
          Target: ${validate.output.target}
        timeout_seconds: 600

    - id: deploy
      type: Agent
      config:
        agent: k8s-agent
        input: "Deploy ${validate.output.manifest}"

    - id: notify-success
      type: Response
      config:
        message: "✅ Deployment complete: ${deploy.output}"

    - id: notify-rejected
      type: Response
      config:
        message: "❌ Deployment rejected by user"

  connections:
    - from: start
      to: validate
    - from: validate
      to: approval
    - from: approval
      to: deploy
      condition: approved
    - from: approval
      to: notify-rejected
      condition: rejected
    - from: deploy
      to: notify-success
```

### Root Cause Analysis Flow

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: rca-flow
spec:
  description: "Multi-model root cause analysis with consensus"

  nodes:
    - id: collect
      type: Fleet
      config:
        fleet: rca-fleet
        input: "${event.text}"

    - id: respond
      type: Response
      config:
        message: "${collect.output}"
        format: markdown

  connections:
    - from: start
      to: collect
    - from: collect
      to: respond
```

### Incident Response Flow

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-flow
spec:
  description: "Automated incident response workflow"

  nodes:
    - id: assess
      type: Agent
      config:
        agent: monitoring-agent
        input: "Assess incident: ${event.text}"

    - id: route
      type: Conditional
      config:
        expression: "${assess.output.severity}"
        cases:
          critical: escalate
          high: page-oncall
          default: log-ticket

    - id: escalate
      type: Agent
      config:
        agent: pagerduty-agent
        input: "Create P1 incident: ${assess.output}"

    - id: page-oncall
      type: Agent
      config:
        agent: slack-agent
        input: "Page on-call: ${assess.output}"

    - id: log-ticket
      type: Agent
      config:
        agent: jira-agent
        input: "Create ticket: ${assess.output}"

  connections:
    - from: start
      to: assess
    - from: assess
      to: route
    - from: route
      to: escalate
      condition: critical
    - from: route
      to: page-oncall
      condition: high
    - from: route
      to: log-ticket
      condition: default
```

---

## Variable Substitution

Flows support variable substitution using `${...}` syntax.

### Event Variables

From the triggering event (via Trigger):

| Variable | Description |
|----------|-------------|
| `${event.text}` | Message text |
| `${event.user.id}` | User ID |
| `${event.user.name}` | User display name |
| `${event.channel}` | Channel/chat ID |
| `${event.platform}` | Platform name (slack, telegram) |
| `${event.timestamp}` | Event timestamp |

### Node Output Variables

Access output from previous nodes:

| Variable | Description |
|----------|-------------|
| `${node_id.output}` | Full output from node |
| `${node_id.output.field}` | Specific field from output |
| `${previous.output}` | Output from previous node |

### Context Variables

From flow context:

| Variable | Description |
|----------|-------------|
| `${context.namespace}` | Kubernetes namespace |
| `${context.cluster}` | Cluster name |
| `${context.env.KEY}` | Environment variable |

---

## Retry Configuration

Configure retry behavior for failed nodes:

```yaml
spec:
  config:
    retry:
      max_attempts: 3
      initial_delay: 1s
      backoff_multiplier: 2.0
      max_delay: 30s
```

Per-node retry:

```yaml
nodes:
  - id: deploy
    type: Agent
    config:
      agent: k8s-agent
      retry:
        max_attempts: 5
        initial_delay: 2s
```

---

## Timeout Configuration

Global timeout:

```yaml
spec:
  config:
    default_timeout_seconds: 300
```

Per-node timeout:

```yaml
nodes:
  - id: long-operation
    type: Agent
    config:
      agent: batch-agent
      timeout_seconds: 600
```

---

## Usage with Triggers

Flows are invoked via Trigger command bindings:

```yaml
# In a Trigger CRD
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
    /deploy:
      flow: deploy-flow          # References this AgentFlow
      description: "Deploy to production"
    /rca:
      flow: rca-flow
      description: "Root cause analysis"

  default_agent: devops
```

---

## CLI Usage

Run a flow directly:

```bash
# Execute a flow with input
aofctl run flow examples/flows/deploy-flow.yaml -i "deploy v2.1.0 to production"

# Describe a flow
aofctl describe flow examples/flows/deploy-flow.yaml

# Validate flow YAML
aofctl validate flows/deploy-flow.yaml
```

---

## Best Practices

1. **Keep flows focused** - One flow per logical workflow
2. **Use descriptive node IDs** - `validate`, `deploy`, not `step1`, `step2`
3. **Add descriptions** - Document what the flow does
4. **Handle failure paths** - Use conditional connections for error cases
5. **Set appropriate timeouts** - Prevent runaway executions
6. **Use fleets for complex analysis** - Multi-agent coordination for RCA
7. **Use Script nodes for deterministic operations** - Save LLM tokens by using Script nodes for data collection, API calls, and file operations. Reserve Agent nodes for tasks requiring intelligence.
8. **Combine Script + Agent nodes** - Use Script nodes to gather data, then pass results to Agent nodes for analysis
