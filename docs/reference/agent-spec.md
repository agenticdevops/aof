# Agent YAML Specification

Complete reference for Agent resource specifications.

## Overview

An Agent is a single AI assistant with specific instructions, tools, and model configuration.

## Basic Structure

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: string              # Required: Unique identifier
  labels:                   # Optional: Key-value labels
    key: value
  annotations:              # Optional: Additional metadata
    key: value

spec:
  model: string             # Required: provider:model
  model_config:             # Optional: Model parameters
    temperature: float
    max_tokens: int
  instructions: string      # Required: System prompt
  tools:                    # Optional: List of tools
    - type: string
      config: object
  memory:                   # Optional: Memory configuration
    type: string
    config: object
```

## Metadata Fields

### `metadata.name`
**Type:** `string`
**Required:** Yes
**Description:** Unique identifier for the agent. Must be DNS-compatible (lowercase, alphanumeric, hyphens).

**Example:**
```yaml
metadata:
  name: k8s-helper
```

### `metadata.labels`
**Type:** `map[string]string`
**Required:** No
**Description:** Key-value pairs for organizing and selecting agents.

**Example:**
```yaml
metadata:
  labels:
    env: production
    team: platform
    purpose: operations
```

### `metadata.annotations`
**Type:** `map[string]string`
**Required:** No
**Description:** Additional metadata for documentation, not used for selection.

**Example:**
```yaml
metadata:
  annotations:
    description: "Kubernetes operations assistant"
    owner: "platform-team@company.com"
    version: "1.2.0"
```

## Spec Fields

### `spec.model`
**Type:** `string`
**Required:** Yes
**Format:** `provider:model`
**Description:** Specifies the LLM provider and model to use.

**Supported Providers:**

| Provider | Models | Example |
|----------|--------|---------|
| `openai` | gpt-4, gpt-4-turbo, gpt-3.5-turbo | `openai:gpt-4` |
| `anthropic` | claude-3-5-sonnet-20241022, claude-3-5-haiku-20241022, claude-3-opus-20240229 | `anthropic:claude-3-5-sonnet-20241022` |
| `ollama` | llama3, mistral, codellama, etc. | `ollama:llama3` |
| `groq` | llama-3.1-70b-versatile, mixtral-8x7b-32768 | `groq:llama-3.1-70b-versatile` |

**Example:**
```yaml
spec:
  model: openai:gpt-4
```

**Environment Variables:**
- OpenAI: `OPENAI_API_KEY`
- Anthropic: `ANTHROPIC_API_KEY`
- Groq: `GROQ_API_KEY`
- Ollama: None (runs locally)

### `spec.model_config`
**Type:** `object`
**Required:** No
**Description:** Fine-tune model behavior.

**Fields:**

| Field | Type | Range | Default | Description |
|-------|------|-------|---------|-------------|
| `temperature` | float | 0.0-2.0 | 1.0 | Randomness (0=deterministic, 2=creative) |
| `max_tokens` | int | 1-∞ | 4096 | Maximum response length |
| `top_p` | float | 0.0-1.0 | 1.0 | Nucleus sampling threshold |
| `frequency_penalty` | float | -2.0-2.0 | 0.0 | Penalize repeated tokens |
| `presence_penalty` | float | -2.0-2.0 | 0.0 | Penalize existing topics |

**Example:**
```yaml
spec:
  model_config:
    temperature: 0.3      # More deterministic
    max_tokens: 2000      # Concise responses
    top_p: 0.9
```

### `spec.instructions`
**Type:** `string`
**Required:** Yes
**Description:** System prompt that defines the agent's behavior, role, and guidelines.

**Best Practices:**
- Start with role definition
- List specific responsibilities
- Include guidelines and constraints
- Specify output format if needed
- Keep focused and concise

**Example:**
```yaml
spec:
  instructions: |
    You are a Kubernetes expert assistant for DevOps engineers.

    Your role:
    - Help users run kubectl commands safely
    - Troubleshoot cluster issues
    - Explain K8s concepts clearly

    Guidelines:
    - Always explain commands before running them
    - Ask for namespace if not specified
    - Use --dry-run for destructive operations
```

### `spec.tools`
**Type:** `array`
**Required:** No
**Description:** List of tools the agent can use to interact with external systems.

**Tool Types:**
1. Shell
2. HTTP
3. MCP (Model Context Protocol)
4. FileSystem
5. Slack
6. GitHub
7. PagerDuty

---

## Tool: Shell

Execute terminal commands.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allowed_commands` | array | Yes | Whitelist of allowed commands |
| `working_directory` | string | No | Directory to execute from |
| `timeout_seconds` | int | No | Max execution time (default: 30) |
| `env` | map | No | Environment variables |

**Example:**
```yaml
tools:
  - type: Shell
    config:
      allowed_commands:
        - kubectl
        - helm
        - git
      working_directory: /tmp
      timeout_seconds: 60
      env:
        KUBECONFIG: /path/to/kubeconfig
```

**Security Note:** Only commands in `allowed_commands` can be executed. Paths and arguments are validated.

---

## Tool: HTTP

Make HTTP/REST API requests.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `base_url` | string | No | Base URL for requests |
| `headers` | map | No | Default headers |
| `timeout_seconds` | int | No | Request timeout (default: 10) |
| `allowed_methods` | array | No | HTTP methods (default: all) |
| `auth` | object | No | Authentication config |

**Example:**
```yaml
tools:
  - type: HTTP
    config:
      base_url: https://api.github.com
      headers:
        Authorization: "token ${GITHUB_TOKEN}"
        Accept: "application/vnd.github.v3+json"
      timeout_seconds: 30
      allowed_methods: [GET, POST, PUT, DELETE]
```

**Authentication Types:**
```yaml
# Bearer token
auth:
  type: Bearer
  token: ${API_TOKEN}

# Basic auth
auth:
  type: Basic
  username: ${USERNAME}
  password: ${PASSWORD}

# API key
auth:
  type: ApiKey
  header: X-API-Key
  value: ${API_KEY}
```

---

## Tool: MCP (Model Context Protocol)

Connect to MCP servers for specialized functionality.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Server identifier |
| `command` | array | Yes | Command to start server |
| `env` | map | No | Environment variables |
| `args` | array | No | Server arguments |

**Example:**
```yaml
tools:
  # kubectl MCP server
  - type: MCP
    config:
      name: kubectl-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
      env:
        KUBECONFIG: "${KUBECONFIG}"

  # GitHub MCP server
  - type: MCP
    config:
      name: github-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"

  # Postgres MCP server
  - type: MCP
    config:
      name: postgres-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-postgres"]
      env:
        DATABASE_URL: "${DATABASE_URL}"
```

**Available MCP Servers:**
- `@modelcontextprotocol/server-kubectl` - Kubernetes operations
- `@modelcontextprotocol/server-github` - GitHub API
- `@modelcontextprotocol/server-postgres` - PostgreSQL queries
- `@modelcontextprotocol/server-filesystem` - File operations
- Custom servers (see MCP documentation)

---

## Tool: FileSystem

Read and write files.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allowed_paths` | array | Yes | Whitelist of paths |
| `read_only` | bool | No | Disable writes (default: false) |
| `max_file_size` | int | No | Max file size in bytes |

**Example:**
```yaml
tools:
  - type: FileSystem
    config:
      allowed_paths:
        - /etc/kubernetes
        - /tmp
      read_only: false
      max_file_size: 10485760  # 10MB
```

---

## Tool: Slack

Send Slack messages and handle interactions.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_token` | string | Yes | Slack bot token (xoxb-...) |
| `signing_secret` | string | Yes | Slack signing secret |
| `default_channel` | string | No | Default channel |

**Example:**
```yaml
tools:
  - type: Slack
    config:
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}
      default_channel: "#platform"
```

---

## Tool: GitHub

Interact with GitHub API.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `token` | string | Yes | GitHub personal access token |
| `default_repo` | string | No | Default repository (owner/repo) |

**Example:**
```yaml
tools:
  - type: GitHub
    config:
      token: ${GITHUB_TOKEN}
      default_repo: myorg/myrepo
```

---

## Tool: PagerDuty

Manage PagerDuty incidents.

**Configuration:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string | Yes | PagerDuty API key |
| `service_id` | string | No | Default service ID |

**Example:**
```yaml
tools:
  - type: PagerDuty
    config:
      api_key: ${PAGERDUTY_API_KEY}
      service_id: ${PAGERDUTY_SERVICE_ID}
```

---

## Memory Configuration

### `spec.memory`
**Type:** `object`
**Required:** No
**Description:** Configure conversation memory persistence.

**Memory Types:**

#### InMemory (Default)
RAM-based storage, cleared on restart.

```yaml
spec:
  memory:
    type: InMemory
    config:
      max_messages: 100  # Keep last 100 messages
```

#### File
JSON file storage.

```yaml
spec:
  memory:
    type: File
    config:
      path: ./agent-memory.json
      max_messages: 50
```

#### SQLite
Embedded database.

```yaml
spec:
  memory:
    type: SQLite
    config:
      path: ./agent-memory.db
      max_messages: 1000
```

#### PostgreSQL
External database for production.

```yaml
spec:
  memory:
    type: PostgreSQL
    config:
      url: postgres://user:pass@localhost/aof
      table: agent_memory
      max_messages: 10000
```

**Context Key (Optional):**
Use different memory contexts for different sessions.

```yaml
spec:
  memory:
    type: SQLite
    config:
      path: ./memory.db
      context_key: "user_${USER_ID}"  # Separate memory per user
```

---

## Complete Examples

### Minimal Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: simple-assistant
spec:
  model: openai:gpt-4
  instructions: "You are a helpful assistant."
```

### Production K8s Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops-agent
  labels:
    env: production
    team: platform
  annotations:
    owner: platform@company.com

spec:
  model: anthropic:claude-3-5-sonnet-20241022

  model_config:
    temperature: 0.3
    max_tokens: 2000

  instructions: |
    You are an expert Kubernetes operations assistant.
    Help DevOps engineers manage their clusters safely.

  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, helm]
        timeout_seconds: 60

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]

    - type: HTTP
      config:
        base_url: http://localhost

  memory:
    type: PostgreSQL
    config:
      url: ${DATABASE_URL}
      max_messages: 1000
```

### Multi-Tool Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-assistant

spec:
  model: openai:gpt-4

  instructions: |
    You are a DevOps automation assistant.
    You can manage K8s, GitHub, and Slack.

  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, git, docker]

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]

    - type: GitHub
      config:
        token: ${GITHUB_TOKEN}

    - type: Slack
      config:
        bot_token: ${SLACK_BOT_TOKEN}
        signing_secret: ${SLACK_SIGNING_SECRET}

    - type: HTTP
      config:
        base_url: https://api.company.com
        headers:
          Authorization: "Bearer ${API_TOKEN}"

  memory:
    type: SQLite
    config:
      path: ./devops-memory.db
```

---

## Best Practices

### Instructions
- ✅ Be specific about the agent's role
- ✅ Include clear guidelines and constraints
- ✅ Specify output format when needed
- ❌ Don't make instructions too long (>500 words)
- ❌ Don't include example conversations

### Model Selection
- **GPT-4**: Best for complex reasoning, expensive
- **Claude Sonnet**: Great balance, good for ops
- **GPT-3.5**: Fast and cheap, simpler tasks
- **Ollama**: Local, no API costs, requires setup

### Temperature
- `0.0-0.3`: Deterministic (ops, diagnostics)
- `0.4-0.7`: Balanced (general purpose)
- `0.8-1.5`: Creative (brainstorming, writing)

### Tools
- ✅ Only add tools the agent needs
- ✅ Use MCP servers when available
- ✅ Whitelist commands explicitly
- ❌ Don't give unrestricted shell access

### Memory
- **Development**: InMemory or File
- **Production**: PostgreSQL
- **Testing**: InMemory (clean state)

---

## Environment Variables

Agents can reference environment variables with `${VAR_NAME}` syntax.

**Example:**
```yaml
spec:
  tools:
    - type: HTTP
      config:
        headers:
          Authorization: "Bearer ${API_TOKEN}"
```

Set variables:
```bash
export API_TOKEN=secret
aofctl agent apply -f agent.yaml
```

---

## Validation

Before applying, validate your YAML:

```bash
# Validate syntax
aofctl agent validate -f agent.yaml

# Dry-run (check without applying)
aofctl agent apply -f agent.yaml --dry-run

# Check applied config
aofctl agent get my-agent -o yaml
```

---

## See Also

- [AgentFleet Spec](agentfleet-spec.md) - Multi-agent teams
- [AgentFlow Spec](agentflow-spec.md) - Workflow automation
- [aofctl CLI](aofctl.md) - Command reference
- [Examples](../examples/) - Copy-paste configurations
