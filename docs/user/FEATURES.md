# AOF Features Overview

AOF (Agentic Ops Framework) is a comprehensive Rust framework for building agentic applications targeting DevOps and SRE workflows.

## Table of Contents

- [Core Features](#core-features)
- [Agent System](#agent-system)
- [Tools](#tools)
- [AgentFleet](#agentfleet)
- [Workflows](#workflows)
- [Triggers](#triggers)
- [Memory System](#memory-system)
- [MCP Integration](#mcp-integration)
- [LLM Providers](#llm-providers)
- [CLI Reference](#cli-reference)

---

## Core Features

### Multi-Provider LLM Support

AOF supports multiple LLM providers out of the box:

| Provider | Model Format | Environment Variable |
|----------|--------------|---------------------|
| Anthropic | `anthropic:claude-3-5-sonnet-20241022` | `ANTHROPIC_API_KEY` |
| OpenAI | `google:gemini-2.5-flash` | `OPENAI_API_KEY` |
| Google | `google:gemini-2.0-flash` | `GOOGLE_API_KEY` |
| AWS Bedrock | `bedrock:anthropic.claude-3-5-sonnet` | AWS credentials |
| Azure OpenAI | `azure:gpt-4` | `AZURE_OPENAI_API_KEY` |
| Ollama | `ollama:llama3.2` | (local) |
| Groq | `groq:llama-3.2-90b` | `GROQ_API_KEY` |

### Agent Configuration

Agents can be configured in Kubernetes-style YAML:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: my-agent
  labels:
    team: platform
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a helpful assistant.
  tools:
    - shell
    - kubectl_get
  max_iterations: 20
  temperature: 0.3
```

---

## Agent System

### Agent Types

1. **Single Agent** - Standalone agent with tools
2. **AgentFleet** - Multi-agent coordination
3. **Workflow Agent** - Step-based execution

### Agent Lifecycle

```
┌─────────────────────────────────────────────┐
│                 Agent                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
│  │ Init    │→ │ Execute │→ │ Cleanup │     │
│  └─────────┘  └─────────┘  └─────────┘     │
│                    ↓                         │
│              ┌───────────┐                   │
│              │ Tool Loop │                   │
│              │ (max_iter)│                   │
│              └───────────┘                   │
└─────────────────────────────────────────────┘
```

---

## Tools

AOF provides a modular tool system with both **built-in tools** and **MCP tools**.

### Unified CLI Tools (Recommended)

For DevOps workflows, use the **unified CLI tools** that take a single `command` argument. The LLM constructs the appropriate command based on context:

```yaml
tools:
  - kubectl    # Run any kubectl command
  - git        # Run any git command
  - docker     # Run any docker command
  - terraform  # Run any terraform command
  - aws        # Run any AWS CLI command
  - helm       # Run any helm command
  - shell      # General shell commands
```

**Benefits:**
- Simpler configuration - fewer tools to list
- More flexible - supports any subcommand
- LLM-native - leverages model's command construction abilities

**Example:**
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-assistant
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a DevOps assistant with access to Kubernetes, Git, and Docker.
  tools:
    - kubectl
    - git
    - docker
    - shell
```

### Tool Categories

| Category | Unified Tool | Example Commands |
|----------|--------------|------------------|
| Kubernetes | `kubectl` | `get pods -n prod`, `apply -f deploy.yaml`, `logs my-pod` |
| Git | `git` | `status`, `commit -m "msg"`, `push origin main` |
| Docker | `docker` | `ps -a`, `build -t app .`, `compose up -d` |
| Terraform | `terraform` | `init`, `plan`, `apply -auto-approve` |
| AWS | `aws` | `s3 ls`, `ec2 describe-instances` |
| Helm | `helm` | `list -A`, `install app ./chart` |
| File | `read_file`, `write_file`, `list_directory`, `search_files` | - |
| Shell | `shell` | Any shell command |
| Observability | `prometheus_query`, `loki_query`, `elasticsearch_query` | - |
| HTTP | `http` | HTTP requests |

### Tool Specification (ToolSpec)

Tools support multiple formats:

```yaml
# 1. Simple string - unified CLI tools (recommended)
tools:
  - kubectl
  - git
  - docker

# 2. Qualified built-in with config
tools:
  - name: shell
    source: builtin
    config:
      blocked_commands: ["rm -rf /"]
      timeout_secs: 60

# 3. Qualified MCP tool
tools:
  - name: search_code
    source: mcp
    server: github
```

### Tool Selection Guide

| Use Case | Recommended |
|----------|-------------|
| DevOps workflows | Unified CLI tools (kubectl, git, docker, etc.) |
| Low latency operations | Built-in tools |
| Production deployments | Built-in tools |
| Prototyping | MCP tools |
| Community tool ecosystem | MCP tools |
| Language-specific tooling | MCP tools |

Full documentation: [docs/tools/index.md](tools/index.md)

---

## AgentFleet

AgentFleet enables multi-agent coordination with different modes:

### Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `hierarchical` | Manager coordinates workers | Complex workflows |
| `peer` | All agents as equals | Distributed analysis |
| `swarm` | Self-organizing | Dynamic workloads |

### Fleet Example

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: incident-response-team
spec:
  agents:
    - name: triage-agent
      role: manager
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Coordinate incident response...
        tools:
          - kubectl_get
          - prometheus_query

    - name: log-analyzer
      role: worker
      replicas: 2
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Analyze logs for errors...
        tools:
          - kubectl_logs
          - loki_query

  coordination:
    mode: hierarchical
    manager: triage-agent
```

### Consensus Algorithms

- `majority` - Simple majority voting
- `unanimous` - All agents must agree
- `weighted` - Weighted by agent role

Full documentation: [AgentFlow Design](../dev/AGENTFLOW_DESIGN.md)

---

## Workflows

Workflows provide step-based execution with state management:

### Step Types

| Type | Description |
|------|-------------|
| `agent` | LLM-based step |
| `tool` | Direct tool execution |
| `condition` | Conditional branching |
| `parallel` | Parallel execution |
| `loop` | Iterative execution |

### Workflow Example

```yaml
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: ci-cd-pipeline
spec:
  steps:
    - name: build
      type: tool
      tool: shell
      input:
        command: "cargo build --release"

    - name: test
      type: tool
      tool: shell
      input:
        command: "cargo test"
      depends_on: [build]

    - name: deploy-decision
      type: agent
      agent: deployment-agent
      depends_on: [test]
```

### Features

- **Checkpointing**: Resume from failures
- **Retry logic**: Configurable retry with backoff
- **Interrupts**: Manual intervention points
- **State management**: Pass data between steps

---

## Triggers

Triggers enable event-driven agent execution:

### Supported Platforms

| Platform | Events |
|----------|--------|
| Slack | Messages, slash commands, button clicks, reactions |
| Discord | Messages, slash commands |
| Telegram | Messages, commands |
| WhatsApp | Messages |
| Webhook | HTTP POST events |

### Trigger Example

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-incident
spec:
  platform: slack
  config:
    token: "${SLACK_BOT_TOKEN}"
    channels: ["#incidents"]
  agent: incident-responder
```

### Human-in-the-Loop Approval Workflow

AOF supports native human-in-the-loop approval for destructive operations. When an agent returns `requires_approval: true`, the system:

1. **Posts approval request** to Slack with ✅ and ❌ reactions
2. **Waits for user reaction** - approve or deny
3. **Executes command** on approval
4. **Sends result feedback** back to Slack

**Agent Configuration:**
```yaml
instructions: |
  For destructive operations (create, delete, scale), return:

  requires_approval: true
  command: "kubectl create deployment nginx --image=nginx"
```

**Example Flow:**
```
User: @bot create nginx deployment with 3 replicas

Bot: I'll create an nginx deployment.
     ⚠️ This action requires approval
     `kubectl create deployment nginx --replicas=3`
     React with ✅ to approve or ❌ to deny.

[User reacts with ✅]

Bot: ✅ Command completed successfully
     deployment.apps/nginx created
     Approved by: @user
```

Full documentation: [Approval Workflow Guide](../guides/approval-workflow.md)

---

## Memory System

AOF provides pluggable memory backends:

### Memory Types

| Backend | Use Case | Persistence |
|---------|----------|-------------|
| `inmemory` | Development, testing | No |
| `redis` | Production, distributed | Yes |
| `sqlite` | Single-node production | Yes |

### Memory Usage

```yaml
spec:
  memory: redis://localhost:6379
  # or
  memory: sqlite:///path/to/memory.db
```

---

## MCP Integration

AOF fully supports the Model Context Protocol (MCP):

### Transport Types

| Transport | Use Case |
|-----------|----------|
| `stdio` | Local MCP servers |
| `sse` | Server-Sent Events |
| `http` | HTTP REST API |

### MCP Configuration

```yaml
mcp_servers:
  - name: filesystem
    transport: stdio
    command: npx
    args: ["@modelcontextprotocol/server-filesystem", "/workspace"]
    env:
      NODE_ENV: production

  - name: remote-tools
    transport: sse
    endpoint: http://localhost:3000/mcp
```

Full documentation: [docs/MCP_CONFIGURATION.md](MCP_CONFIGURATION.md)

---

## LLM Providers

### Provider Configuration

```yaml
# Anthropic
model: anthropic:claude-3-5-sonnet-20241022

# OpenAI
model: google:gemini-2.5-flash

# Google
model: google:gemini-2.0-flash

# AWS Bedrock
model: bedrock:anthropic.claude-3-5-sonnet-20241022-v2:0

# Azure OpenAI
model: azure:gpt-4
# Requires: AZURE_OPENAI_API_KEY, AZURE_OPENAI_ENDPOINT

# Ollama (local)
model: ollama:llama3.2

# Groq
model: groq:llama-3.2-90b-text-preview
```

See the [Getting Started Guide](../getting-started.md) for provider setup.

---

## CLI Reference

### Running Agents

```bash
# Run an agent
aofctl run --file agent.yaml

# Run with input
aofctl run --file agent.yaml --input "List all pods"

# Run with streaming output
aofctl run --file agent.yaml --stream
```

### Running Workflows

```bash
# Execute a workflow
aofctl run workflow workflow.yaml

# Resume from checkpoint
aofctl run workflow workflow.yaml --resume
```

### Running Fleets

```bash
# List fleets
aofctl get fleets

# Run a fleet with input
aofctl run fleet fleet.yaml -i '{"task": "Analyze failing pods"}'

# Describe fleet
aofctl describe fleet fleet.yaml
```

### Validation

```bash
# Validate configuration
aofctl validate --file config.yaml
```

Full documentation: [docs/CLI_REFERENCE.md](CLI_REFERENCE.md)

---

## Crate Structure

```
aof/
├── aof-core       # Core traits, types, interfaces
├── aof-llm        # LLM provider abstraction
├── aof-mcp        # MCP client implementation
├── aof-runtime    # Agent execution runtime
├── aof-memory     # Memory backends
├── aof-tools      # Built-in tool implementations
├── aof-triggers   # Event trigger system
└── aofctl         # CLI binary
```

### Feature Flags

```toml
[dependencies]
aof-tools = { version = "0.1", features = ["kubectl", "docker", "git"] }

# Or enable all tools
aof-tools = { version = "0.1", features = ["all"] }
```

---

## Getting Started

1. **Install**:
   ```bash
   cargo install aofctl
   ```

2. **Create an agent**:
   ```yaml
   # hello-agent.yaml
   apiVersion: aof.dev/v1
   kind: Agent
   metadata:
     name: hello
   spec:
     model: google:gemini-2.5-flash
     instructions: |
       You are a helpful assistant.
   ```

3. **Run**:
   ```bash
   export OPENAI_API_KEY=your-key
   aofctl run --file hello-agent.yaml
   ```

---

## Examples

See the `examples/` directory for:

- `examples/agents/` - Agent configurations
- `examples/fleets/` - AgentFleet examples
- `examples/workflows/` - Workflow definitions

---

## Contributing

See [CONTRIBUTING.md](../dev/CONTRIBUTING.md) for guidelines.

## License

Apache 2.0 - See the LICENSE file in the repository root.
