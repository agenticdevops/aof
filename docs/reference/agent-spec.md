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
  memory: string            # Optional: "InMemory", "File:./path", etc.
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

Tools can be specified in two formats:
1. **Simple format**: Just the tool name as a string
2. **Qualified format**: Object with name, source, config, and other options

**Simple Format (Recommended):**
```yaml
tools:
  - shell
  - kubectl
  - git
  - docker
```

**Qualified Format (For Advanced Configuration):**
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

---

## Memory Configuration

### `spec.memory`
**Type:** `string`
**Required:** No
**Description:** Memory backend identifier string.

**Format:** `"Type"` or `"Type:config"`

**Examples:**

```yaml
spec:
  # In-memory (default) - cleared on restart
  memory: "InMemory"

  # File-based - persists to JSON file
  memory: "File:./agent-memory.json"

  # SQLite - embedded database
  memory: "SQLite:./agent-memory.db"

  # PostgreSQL - for production
  memory: "PostgreSQL:postgres://user:pass@localhost/aof"
```

**Available Memory Types:**

| Type | Format | Description |
|------|--------|-------------|
| `InMemory` | `"InMemory"` | RAM-based, cleared on restart (default) |
| `File` | `"File:./path.json"` | JSON file persistence |
| `SQLite` | `"SQLite:./path.db"` | Embedded SQLite database |
| `PostgreSQL` | `"PostgreSQL:connection_url"` | External PostgreSQL |

**Note:** Memory is a simple string, not a complex object. If omitted, defaults to `InMemory`.

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

  memory: "PostgreSQL:${DATABASE_URL}"
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

  memory: "SQLite:./devops-memory.db"
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
- **Production**: `"PostgreSQL:connection_url"`
- **Testing**: `"InMemory"` (clean state each run)

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
