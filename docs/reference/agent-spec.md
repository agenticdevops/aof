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
  max_context_messages: int # Optional: Max history messages (default: 10)
  tools:                    # Optional: List of tools
    - string                # Simple format: just tool name
    # OR qualified format:
    # - name: string
    #   source: builtin|mcp
    #   config: object
  mcp_servers:              # Optional: MCP server configs
    - name: string
      transport: stdio|sse|http
      command: string
      args: []
      env: {}
  memory: string|object     # Optional: "InMemory", "File:./path", or structured config
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
| `openai` | gpt-4, gpt-4-turbo, gpt-3.5-turbo | `google:gemini-2.5-flash` |
| `anthropic` | claude-3-5-sonnet-20241022, claude-3-5-haiku-20241022, claude-3-opus-20240229 | `anthropic:claude-3-5-sonnet-20241022` |
| `ollama` | llama3, mistral, codellama, etc. | `ollama:llama3` |
| `groq` | llama-3.1-70b-versatile, mixtral-8x7b-32768 | `groq:llama-3.1-70b-versatile` |

**Example:**
```yaml
spec:
  model: google:gemini-2.5-flash
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

### `spec.max_context_messages`
**Type:** `int`
**Required:** No
**Default:** `10`
**Description:** Maximum number of conversation messages to include in context when using persistent memory.

This controls token usage by limiting how much conversation history is sent to the LLM. When history exceeds this limit, oldest messages are pruned (keeping system messages).

**Trade-offs:**
- **Lower values (5-10)**: Less token usage, cheaper, but agent has shorter memory
- **Higher values (50-100)**: More context, better continuity, but more expensive

**Example:**
```yaml
spec:
  # Short memory - good for simple Q&A
  max_context_messages: 5

  # Longer memory - good for complex multi-turn conversations
  max_context_messages: 50
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

Tools can be specified in three formats:
1. **Simple format**: Just the tool name as a string
2. **Type-based format**: Object with `type` (Shell/MCP/HTTP) and `config`
3. **Qualified format**: Object with name, source, config, and other options

#### Simple Format
```yaml
tools:
  - shell
  - kubectl
  - git
  - docker
```

#### Type-Based Format (Recommended for explicit configuration)

Use `type: Shell`, `type: MCP`, or `type: HTTP` with a `config` object:

```yaml
tools:
  # Shell tool with command restrictions
  - type: Shell
    config:
      allowed_commands:
        - kubectl
        - helm
      working_directory: /tmp
      timeout_seconds: 30

  # MCP server tool
  - type: MCP
    config:
      name: kubectl-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
      env:
        KUBECONFIG: "${KUBECONFIG}"

  # HTTP API tool
  - type: HTTP
    config:
      base_url: http://localhost:8080
      timeout_seconds: 10
      allowed_methods: [GET, POST]
```

**Type-Based Tool Fields:**

| Type | Config Fields | Description |
|------|---------------|-------------|
| `Shell` | `allowed_commands`, `working_directory`, `timeout_seconds` | Shell command execution with restrictions |
| `MCP` | `name`, `command`, `args`, `env` | MCP server tool |
| `HTTP` | `base_url`, `timeout_seconds`, `allowed_methods` | HTTP API calls |

#### Qualified Format (Legacy)
```yaml
tools:
  - name: shell
    source: builtin
    config:
      allowed_commands: ["kubectl", "helm"]
    timeout_secs: 60

  - name: read_file
    source: mcp
    server: filesystem
```

**Qualified Tool Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Tool name |
| `source` | string | No | `builtin` or `mcp` (default: builtin) |
| `server` | string | MCP only | MCP server name for this tool |
| `config` | object | No | Tool-specific configuration |
| `enabled` | bool | No | Enable/disable tool (default: true) |
| `timeout_secs` | int | No | Timeout override for this tool |

---

## Built-in Tools Reference

AOF provides 40+ built-in tools. Here are the most commonly used:

### CLI Tools (Unified Interface)

These tools call system CLIs with a `command` argument:

| Tool | Description | Example |
|------|-------------|---------|
| `shell` | Execute shell commands | General command execution |
| `kubectl` | Kubernetes CLI | `kubectl get pods -n default` |
| `git` | Git version control | `git status`, `git log` |
| `docker` | Docker container CLI | `docker ps`, `docker logs` |
| `helm` | Helm package manager | `helm list`, `helm upgrade` |
| `terraform` | Infrastructure as Code | `terraform plan` |
| `aws` | AWS CLI | `aws s3 ls` |

**Example:**
```yaml
tools:
  - kubectl
  - git
  - docker
  - helm
```

### File Operations

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents |
| `write_file` | Write to files |
| `list_directory` | List directory contents |

### Observability Tools

| Tool | Description |
|------|-------------|
| `prometheus_query` | Query Prometheus metrics |
| `loki_query` | Query Loki logs |

For the complete list of 40+ tools, see [Built-in Tools Reference](../tools/builtin-tools.md).

---

## MCP Servers

For external tools via MCP servers, configure them separately using `mcp_servers`:

```yaml
spec:
  tools:
    - shell
    - git

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

**MCP Server Configuration Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Server identifier |
| `transport` | string | Yes | `stdio`, `sse`, or `http` |
| `command` | string | Yes (stdio) | Command to start server |
| `args` | array | No | Command arguments |
| `env` | map | No | Environment variables |
| `url` | string | Yes (sse/http) | Server URL |
| `timeout_secs` | int | No | Connection timeout |

**Popular MCP Servers:**
- `@modelcontextprotocol/server-filesystem` - File operations
- `@modelcontextprotocol/server-github` - GitHub API
- `@modelcontextprotocol/server-postgres` - PostgreSQL queries
- `@modelcontextprotocol/server-slack` - Slack integration

For more details, see [MCP Integration Guide](../tools/mcp-integration.md).

### Graceful Degradation

When MCP servers fail to initialize (e.g., unavailable server, network issues, missing packages), the agent will:

1. **Log a warning** with detailed error information
2. **Continue loading** with any successfully initialized tools
3. **Fall back to builtin tools** if configured alongside MCP

This ensures agents remain functional even when some external tools are unavailable.

**Example with fallback:**
```yaml
spec:
  tools:
    # Builtin Shell tool - always available
    - type: Shell
      config:
        allowed_commands: [kubectl, helm]

    # MCP tool - optional, agent continues if unavailable
    - type: MCP
      config:
        name: kubernetes-mcp
        command: ["npx", "-y", "@example/mcp-server-kubernetes"]
```

If the MCP server fails to start, the agent will still load with the Shell tool available.

---

## Memory Configuration

### `spec.memory`
**Type:** `string | object`
**Required:** No
**Description:** Memory backend configuration. Supports both simple string format and structured object format.

#### Simple String Format (Backward Compatible)

**Format:** `"Type"` or `"Type:config"` or `"Type:config:options"`

```yaml
spec:
  # In-memory (default) - cleared on restart
  memory: "InMemory"

  # File-based - persists to JSON file
  memory: "File:./agent-memory.json"

  # File-based with max entries limit (keeps last 100 conversations)
  memory: "File:./agent-memory.json:100"

  # Alternative formats (case-insensitive type)
  memory: "file:./agent-memory.json"
  memory: "in_memory"
```

#### Structured Object Format

For more explicit configuration, use the structured format with `type` and `config` fields:

```yaml
spec:
  memory:
    type: File
    config:
      path: ./k8s-helper-memory.json
      max_messages: 50

  # Or for in-memory:
  memory:
    type: InMemory
    config:
      max_messages: 100
```

**Structured Format Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Memory backend type: `File`, `InMemory` |
| `config` | object | No | Backend-specific configuration |
| `config.path` | string | File only | Path to the JSON file |
| `config.max_messages` | int | No | Maximum number of entries to retain |

**Available Memory Types:**

| Type | Format | Description |
|------|--------|-------------|
| `InMemory` | `"InMemory"` or `{type: InMemory}` | RAM-based, cleared on restart (default) |
| `File` | `"File:./path.json"` or `{type: File, config: {path: ...}}` | JSON file persistence |
| `SQLite` | `"SQLite:./path.db"` | *Planned for future release* |
| `PostgreSQL` | `"PostgreSQL:url"` | *Planned for future release* |

### File Memory with Entry Limits

To prevent unbounded file growth, you can specify a maximum number of entries. When the limit is exceeded, the oldest entries (by creation time) are automatically removed.

**Simple format:**
```yaml
spec:
  # Keep only the last 50 conversation turns
  memory: "File:./conversations.json:50"
```

**Structured format:**
```yaml
spec:
  memory:
    type: File
    config:
      path: ./conversations.json
      max_messages: 50
```

**Note:** If omitted, memory defaults to `InMemory`.

**Future Backends:** SQLite and PostgreSQL backends are planned for future releases. Use `InMemory` for development/testing and `File` for persistent storage.

---

## Complete Examples

### Minimal Agent

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: simple-assistant
spec:
  model: google:gemini-2.5-flash
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
  model: google:gemini-2.5-flash

  model_config:
    temperature: 0.3
    max_tokens: 2000

  instructions: |
    You are an expert Kubernetes operations assistant.
    Help DevOps engineers manage their clusters safely.

  # Simple tool format - just names
  tools:
    - kubectl
    - helm
    - shell

  # Persistent memory with conversation limit
  memory: "File:./k8s-agent-memory.json:100"
  max_context_messages: 20  # Keep last 20 messages for context
```

### Multi-Tool Agent with MCP

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-assistant

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You are a DevOps automation assistant.
    You can manage K8s, GitHub, and files.

  # Built-in tools
  tools:
    - kubectl
    - git
    - docker
    - shell

  # MCP servers for extended capabilities
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

  memory: "File:./devops-memory.json:200"
  max_context_messages: 30  # More context for complex DevOps tasks
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
- **Development**: `"InMemory"` or `"File:./memory.json"`
- **Production**: `"File:./memory.json:1000"` (with entry limit)
- **Testing**: `"InMemory"` (clean state each run)
- **Conversation History**: Use `"File:./path.json:N"` to keep last N interactions

---

## Environment Variables

Agents can reference environment variables with `${VAR_NAME}` syntax.

**Example:**
```yaml
spec:
  mcp_servers:
    - name: github
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

Set variables:
```bash
export GITHUB_TOKEN=ghp_your_token
aofctl run agent agent.yaml --input "list my repos"
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

- AgentFleet Spec (coming soon) - Multi-agent teams
- [AgentFlow Spec](agentflow-spec.md) - Workflow automation
- [aofctl CLI](aofctl.md) - Command reference
- [Examples](../examples/) - Copy-paste configurations
