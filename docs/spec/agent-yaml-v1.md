# Agent YAML v1 Specification

**apiVersion:** `openagentix.dev/v1`
**kind:** `Agent`
**Spec version:** v1 (2026-03-12)

---

## Overview

An Agent is the fundamental unit of automation in OpenAgentiX. Each agent is defined as a YAML file that describes what the agent does, how it thinks, what tools it can use, and when it runs.

Agent YAML follows Kubernetes-style conventions: a required `apiVersion` and `kind` header, a `metadata` block for identity and organization, and a `spec` block for all runtime behavior.

---

## Top-Level Structure

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: <string>           # required
  namespace: <string>      # optional, default: "default"
  version: <string>        # optional, semver format
  labels: <map>            # optional
  annotations: <map>       # optional
spec:
  model: <string>          # required (or inherited from workspace)
  mode: <enum>             # optional, default: "autonomous"
  system_prompt: <string>  # optional (mutually exclusive with system_prompt_file)
  system_prompt_file: <string>  # optional (mutually exclusive with system_prompt)
  max_iterations: <int>    # optional, default: 10
  timeout: <string>        # optional, default: "5m"
  tools: [...]             # optional
  mcp_servers: [...]       # optional
  triggers: [...]          # optional
  notifications: [...]     # optional
  approval: {...}          # optional
  budget: {...}            # optional
  telemetry: {...}         # optional
  env: <map>               # optional
```

---

## metadata Fields

### `metadata.name` (required)

The unique identifier for this agent within its namespace.

- **Type:** string
- **Validation:** Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, maximum 63 characters
- **Examples:** `cost-analyzer`, `db-optimizer`, `security-scanner`

### `metadata.namespace` (optional)

Logical grouping for agents, enabling multi-team deployments on a single gateway.

- **Type:** string
- **Default:** `"default"`
- **Validation:** Same format as `name` — `^[a-z][a-z0-9-]*[a-z0-9]$`, max 63 chars
- **Examples:** `"production"`, `"staging"`, `"team-platform"`

### `metadata.version` (optional)

Configuration version for this agent definition. Useful for tracking changes to agent behavior over time in version control.

- **Type:** string
- **Format:** Semver — `MAJOR.MINOR.PATCH` (e.g., `"1.0.0"`, `"2.1.3"`)
- **Default:** none (unversioned)

### `metadata.labels` (optional)

Arbitrary key-value metadata for filtering, grouping, and querying agents.

- **Type:** map[string]string
- **Examples:**
  ```yaml
  labels:
    team: platform
    environment: production
    pack: dba-optimizer
  ```

### `metadata.annotations` (optional)

Non-identifying metadata — human-readable notes, tooling hints, links to runbooks.

- **Type:** map[string]string
- **Examples:**
  ```yaml
  annotations:
    description: "Analyzes slow queries and recommends indexes"
    runbook: "https://wiki.example.com/agents/db-optimizer"
  ```

---

## spec Fields

### `spec.model` (required*)

The LLM model the agent uses. Format: `provider/model`.

- **Type:** string
- **Format:** `<provider>/<model-name>` — exactly one `/` separator required
- **Can be inherited from workspace defaults** — if omitted here, the workspace `defaults.model` is used. Error if both are absent.
- **Supported providers:** `anthropic`, `openai`, `google`, `ollama`
- **Examples:**
  - `anthropic/claude-sonnet-4-6`
  - `openai/gpt-4o`
  - `openai/gpt-4o-mini`
  - `google/gemini-2.0-flash`
  - `google/gemini-1.5-pro`
  - `ollama/llama3`
  - `ollama/mistral`

### `spec.mode` (optional)

Determines how much autonomous authority the agent has. Controls approval requirements.

- **Type:** enum
- **Default:** `autonomous`
- **Values:**
  - `autonomous` — Agent executes all tools without human approval (full automation)
  - `semi-autonomous` — Agent executes read-only tools freely; write/destructive tools require approval
  - `manual` — Every tool call requires human approval before execution
- **Note:** Approval behavior is fully implemented in Phase 20 (Human-in-the-Loop). This field is parsed and stored now; enforcement gates come later.

### `spec.system_prompt` (optional)

Inline system prompt text defining the agent's persona, goals, and constraints.

- **Type:** string (multiline YAML string)
- **Mutually exclusive with** `system_prompt_file`
- **Example:**
  ```yaml
  system_prompt: |
    You are a database optimization specialist.
    Analyze query performance and recommend index improvements.
    Always explain your reasoning before making changes.
  ```

### `spec.system_prompt_file` (optional)

Path to a Markdown file containing the system prompt. Relative paths are resolved from the workspace root.

- **Type:** string (file path)
- **Mutually exclusive with** `system_prompt`
- **Supported formats:** `.md`, `.txt`
- **Example:**
  ```yaml
  system_prompt_file: prompts/db-optimizer.md
  ```

### `spec.max_iterations` (optional)

Maximum number of ReAct loop iterations (plan → act → observe → reflect cycles) before the agent stops.

- **Type:** integer
- **Default:** `10`
- **Validation:** 1–100 (inclusive)
- **Guidance:** Set lower (3–5) for simple tasks, higher (20–50) for complex multi-step workflows.

### `spec.timeout` (optional)

Maximum wall-clock time for a single agent run. The agent is terminated if this duration is exceeded.

- **Type:** string (duration)
- **Default:** `"5m"`
- **Format:** Duration string — number followed by unit suffix:
  - `s` — seconds (e.g., `"30s"`)
  - `m` — minutes (e.g., `"5m"`, `"30m"`)
  - `h` — hours (e.g., `"1h"`, `"2h"`)
- **Examples:** `"30s"`, `"5m"`, `"1h"`, `"90m"`

### `spec.tools` (optional)

Unified list of tools available to the agent. All tool types are defined under one key with a `type` discriminator field.

```yaml
tools:
  - name: <string>   # required, unique within agent
    type: <enum>     # required: "cli" | "mcp" | "shell"
    description: <string>  # optional but recommended — shown to the LLM
    # ... type-specific fields
```

**Tool type: `cli`**

Execute a specific CLI binary with defined arguments.

```yaml
- name: kubectl-get-pods
  type: cli
  description: "List all pods in a namespace"
  command: kubectl
  args:
    - get
    - pods
    - -n
    - "{{namespace}}"
```

| Field | Required | Description |
|-------|----------|-------------|
| `command` | yes | Binary name or full path |
| `args` | no | Static argument list (template vars supported via `{{var}}`) |
| `description` | no | Human-readable description for LLM context |

**Tool type: `mcp`**

Expose tools from an MCP server. References a server by name from `spec.mcp_servers`.

```yaml
- name: file-operations
  type: mcp
  description: "Read and write files via the filesystem MCP server"
  server: filesystem
```

| Field | Required | Description |
|-------|----------|-------------|
| `server` | yes | Name of the MCP server (must exist in `spec.mcp_servers`) |
| `description` | no | Human-readable description for LLM context |

**Tool type: `shell`**

Execute an arbitrary shell command. Supports template variables.

```yaml
- name: check-disk-usage
  type: shell
  description: "Check disk usage on a host"
  command: "df -h {{path}}"
```

| Field | Required | Description |
|-------|----------|-------------|
| `command` | yes | Shell command string (template vars via `{{var}}`) |
| `description` | no | Human-readable description for LLM context |

### `spec.mcp_servers` (optional)

List of MCP (Model Context Protocol) server configurations. These are referenced by `tools` entries with `type: mcp`.

```yaml
mcp_servers:
  - name: <string>         # required, unique within agent
    transport: <enum>      # required: "stdio" | "sse" | "http"
    command: <string>      # for stdio transport
    args: [<string>]       # for stdio transport
    url: <string>          # for sse/http transport
    env: <map>             # optional environment variables
```

**Transport: `stdio`**

Launch a local MCP server process communicating over standard I/O.

```yaml
mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args:
      - -y
      - "@modelcontextprotocol/server-filesystem"
      - /workspace
```

**Transport: `sse` or `http`**

Connect to a remote MCP server over Server-Sent Events or HTTP.

```yaml
mcp_servers:
  - name: remote-tools
    transport: sse
    url: https://tools.example.com/mcp
    env:
      API_KEY: "${REMOTE_TOOLS_API_KEY}"
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Unique name within this agent |
| `transport` | yes | `stdio`, `sse`, or `http` |
| `command` | for stdio | Executable to launch |
| `args` | no | Arguments for stdio command |
| `url` | for sse/http | Remote server URL |
| `env` | no | Environment variables passed to stdio process or sent as headers |

### `spec.triggers` (optional)

When the agent should run automatically. Each trigger has a `type` field plus type-specific configuration.

```yaml
triggers:
  - type: <enum>   # "cron" | "webhook" | "github" | "jira" | "mention" | "agent" | "manual"
  # ... type-specific fields
```

> **Note:** Trigger implementations are defined in Phase 15. This field is parsed and stored by the Phase 13 runtime; the actual trigger dispatch logic ships in Phase 15.

**Trigger types:**

| Type | Description | Key Fields |
|------|-------------|------------|
| `cron` | Schedule-based execution | `schedule` (cron expression) |
| `webhook` | HTTP webhook receives payload | `path`, `secret` |
| `github` | GitHub event (PR, issue, push) | `event`, `repo` |
| `jira` | Jira issue created/updated | `event`, `project` |
| `mention` | Agent mentioned in chat channel | `channel` |
| `agent` | Another agent triggers this one | `source_agent` |
| `manual` | No automatic trigger, only manual invocation | — |

### `spec.notifications` (optional)

Where to send results after the agent completes a run.

```yaml
notifications:
  - channel: <enum>     # "slack" | "telegram" | "discord" | "email" | "webhook"
    target: <string>    # channel ID, chat ID, email address, or URL
```

> **Note:** Notification delivery is implemented in Phase 16.

**Examples:**

```yaml
notifications:
  - channel: slack
    target: "#ops-alerts"
  - channel: telegram
    target: "-1001234567890"
  - channel: webhook
    target: "https://hooks.example.com/notify"
```

### `spec.approval` (optional)

Human-in-the-loop approval configuration. Defines which actions require approval and who approves them.

```yaml
approval:
  required_for:
    - "kubectl delete *"
    - "terraform apply"
  approvers:
    - "@ops-lead"
    - "@platform-team"
  timeout: "30m"
```

> **Note:** Approval enforcement is implemented in Phase 20. This field is parsed and stored now.

| Field | Type | Description |
|-------|------|-------------|
| `required_for` | list[string] | Glob patterns matching tool calls that require approval |
| `approvers` | list[string] | User or group identities authorized to approve |
| `timeout` | string | How long to wait for approval before auto-rejecting |

### `spec.budget` (optional)

Cost control limits for this agent.

```yaml
budget:
  daily_limit: "$0.50"
  max_tokens_per_run: 50000
```

> **Note:** Budget enforcement is implemented in Phase 17.

| Field | Type | Description |
|-------|------|-------------|
| `daily_limit` | string | Maximum spend per day (USD, format: `"$0.50"`) |
| `max_tokens_per_run` | integer | Maximum tokens consumed in a single run |

### `spec.telemetry` (optional)

Observability configuration for this agent.

```yaml
telemetry:
  enabled: true
  exporter: otlp
```

> **Note:** Telemetry export is implemented in Phase 18.

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | boolean | Whether to emit telemetry events |
| `exporter` | string | Exporter type (`otlp`, `prometheus`, `datadog`) |

### `spec.env` (optional)

Environment variables made available to all tools this agent runs. Supports `${VAR}` syntax for pulling from the host environment.

```yaml
env:
  DATABASE_URL: "${DATABASE_URL}"
  LOG_LEVEL: "info"
  REGION: "us-east-1"
```

---

## Validation Rules

The OpenAgentiX runtime validates agent YAML at load time and reports errors with precise field paths (rustc-style output).

| Field | Rule | Error |
|-------|------|-------|
| `metadata.name` | Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, max 63 chars | `metadata.name: invalid format — must be lowercase alphanumeric with hyphens` |
| `metadata.namespace` | Same format as `metadata.name` | `metadata.namespace: invalid format` |
| `metadata.version` | Must be valid semver if present (`MAJOR.MINOR.PATCH`) | `metadata.version: must be semver format (e.g., "1.0.0")` |
| `spec.model` | Must contain exactly one `/` | `spec.model: must be "provider/model" format (e.g., "anthropic/claude-sonnet-4-6")` |
| `spec.mode` | Must be one of: `autonomous`, `semi-autonomous`, `manual` | `spec.mode: unknown value — expected "autonomous", "semi-autonomous", or "manual"` |
| `spec.system_prompt` + `spec.system_prompt_file` | Mutually exclusive — cannot specify both | `spec: system_prompt and system_prompt_file are mutually exclusive` |
| `spec.max_iterations` | Integer in range 1–100 | `spec.max_iterations: must be between 1 and 100` |
| `spec.timeout` | Must match `\d+(s\|m\|h)` | `spec.timeout: invalid duration — use format like "30s", "5m", "1h"` |
| `spec.tools[*].type` | Must be one of: `cli`, `mcp`, `shell` | `spec.tools[N].type: unknown tool type` |
| `spec.tools[*].server` (mcp) | Must reference a name in `spec.mcp_servers` | `spec.tools[N].server: "X" not found in mcp_servers` |
| `spec.mcp_servers[*].transport` | Must be one of: `stdio`, `sse`, `http` | `spec.mcp_servers[N].transport: unknown transport` |

---

## Examples

### Example 1: Minimal Agent

The simplest valid agent — a name, a model, and a system prompt.

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: hello-world
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: |
    You are a helpful assistant. Answer questions concisely.
```

### Example 2: Full-Featured Agent

A database optimization agent with tools, MCP server, triggers, and all optional fields.

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: db-optimizer
  namespace: production
  version: "1.2.0"
  labels:
    team: platform
    pack: dba-optimizer
  annotations:
    description: "Analyzes slow PostgreSQL queries and recommends index improvements"
    runbook: "https://wiki.example.com/agents/db-optimizer"
spec:
  model: anthropic/claude-sonnet-4-6
  mode: semi-autonomous
  system_prompt: |
    You are a PostgreSQL performance expert.
    Your goal is to identify slow queries and recommend index improvements.
    Always explain your reasoning. Never DROP indexes without explicit approval.
    Focus on queries with execution time > 1s or seq scans on tables > 10K rows.
  max_iterations: 20
  timeout: "15m"

  tools:
    - name: run-explain
      type: cli
      description: "Run EXPLAIN ANALYZE on a query to see the execution plan"
      command: psql
      args:
        - -U
        - "${DB_USER}"
        - -d
        - "{{database}}"
        - -c
        - "EXPLAIN ANALYZE {{query}}"
    - name: list-indexes
      type: cli
      description: "List all indexes for a table"
      command: psql
      args:
        - -U
        - "${DB_USER}"
        - -c
        - "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = '{{table}}'"
    - name: db-schema
      type: mcp
      description: "Inspect database schema, table structure, and column types"
      server: postgres-mcp

  mcp_servers:
    - name: postgres-mcp
      transport: stdio
      command: npx
      args:
        - -y
        - "@modelcontextprotocol/server-postgres"
      env:
        DATABASE_URL: "${DATABASE_URL}"

  triggers:
    - type: cron
      schedule: "0 2 * * *"  # 2am daily
    - type: manual

  notifications:
    - channel: slack
      target: "#db-alerts"
    - channel: webhook
      target: "https://hooks.example.com/db-optimizer"

  approval:
    required_for:
      - "psql * CREATE INDEX *"
      - "psql * DROP INDEX *"
    approvers:
      - "@db-admin"
    timeout: "1h"

  budget:
    daily_limit: "$2.00"
    max_tokens_per_run: 100000

  telemetry:
    enabled: true
    exporter: otlp

  env:
    DB_USER: "${DB_USER}"
    DATABASE_URL: "${DATABASE_URL}"
    LOG_LEVEL: info
```

### Example 3: Agent with External System Prompt File

An agent that loads its system prompt from a Markdown file in version control.

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: security-scanner
  namespace: security
  version: "2.0.1"
  labels:
    team: security
    severity: critical
spec:
  model: openai/gpt-4o
  mode: autonomous
  system_prompt_file: prompts/security-scanner.md
  max_iterations: 30
  timeout: "30m"

  tools:
    - name: scan-dependencies
      type: shell
      description: "Scan project dependencies for known vulnerabilities"
      command: "trivy fs {{path}} --format json"
    - name: check-secrets
      type: shell
      description: "Check for accidentally committed secrets or credentials"
      command: "trufflehog filesystem {{path}} --json"

  triggers:
    - type: cron
      schedule: "0 6 * * 1"  # Monday 6am
    - type: github
      event: pull_request
      repo: "example-org/*"

  notifications:
    - channel: slack
      target: "#security-alerts"
    - channel: email
      target: "security@example.com"

  env:
    TRIVY_CACHE_DIR: "/tmp/trivy-cache"
```

---

## Field Reference Table

| Field | Type | Required | Default | Phase |
|-------|------|----------|---------|-------|
| `apiVersion` | string | yes | — | 13 |
| `kind` | string | yes | — | 13 |
| `metadata.name` | string | yes | — | 13 |
| `metadata.namespace` | string | no | `"default"` | 13 |
| `metadata.version` | string | no | — | 13 |
| `metadata.labels` | map | no | — | 13 |
| `metadata.annotations` | map | no | — | 13 |
| `spec.model` | string | no* | workspace default | 13 |
| `spec.mode` | enum | no | `autonomous` | 13 |
| `spec.system_prompt` | string | no | — | 13 |
| `spec.system_prompt_file` | string | no | — | 13 |
| `spec.max_iterations` | integer | no | `10` | 13 |
| `spec.timeout` | string | no | `"5m"` | 13 |
| `spec.tools` | list | no | — | 13 |
| `spec.mcp_servers` | list | no | — | 13 |
| `spec.triggers` | list | no | — | 15 |
| `spec.notifications` | list | no | — | 16 |
| `spec.approval` | object | no | — | 20 |
| `spec.budget` | object | no | — | 17 |
| `spec.telemetry` | object | no | — | 18 |
| `spec.env` | map | no | — | 13 |

*Required unless `defaults.model` is set in the workspace `agentix.yaml`.

---

## Related Specifications

- [Workspace Configuration](workspace-config.md) — workspace-level defaults and provider configuration
- Agent YAML is parsed by `agentix-core` crate (`AgentSpec` type)
- Validation uses `serde_path_to_error` for precise field-level error reporting
