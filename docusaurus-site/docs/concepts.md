# Core Concepts

AOF has three main building blocks: **Agents**, **AgentFleets**, and **AgentFlows**. If you know Kubernetes, these will feel familiar.

## Agent

An **Agent** is a single AI assistant with specific instructions, tools, and model configuration.

Think of it like a Kubernetes Pod - it's the smallest deployable unit.

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: my-agent
spec:
  model: openai:gpt-4
  instructions: "You are a helpful assistant"
  tools:
    - type: Shell
```

### When to Use
- Simple, focused tasks (code review, Q&A)
- Single-purpose automation
- Interactive chat sessions
- Quick prototyping

### Key Components

| Component | Description | Example |
|-----------|-------------|---------|
| `model` | LLM to use | `openai:gpt-4`, `anthropic:claude-3-5-sonnet-20241022` |
| `instructions` | System prompt | "You are a K8s expert" |
| `tools` | What the agent can do | Shell, HTTP, MCP servers |
| `memory` | Conversation persistence | In-memory, file, database |

## AgentFleet

An **AgentFleet** is a team of agents working together on a shared task.

Think of it like a Kubernetes Deployment - multiple replicas working in parallel.

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-team
spec:
  agents:
    - name: security-reviewer
      model: openai:gpt-4
      instructions: "Focus on security vulnerabilities"

    - name: performance-reviewer
      model: anthropic:claude-3-5-sonnet-20241022
      instructions: "Focus on performance issues"

    - name: style-reviewer
      model: ollama:llama3
      instructions: "Focus on code style and readability"
```

### When to Use
- Complex tasks requiring multiple perspectives
- Parallel processing of data
- Consensus-building (multiple agents vote)
- Specialized expertise (security + performance + style)

### How It Works
1. You submit a task to the fleet
2. Each agent processes it independently
3. Results are aggregated (consensus, summary, or all responses)

## AgentFlow

An **AgentFlow** is a workflow that orchestrates agents, tools, and integrations in a directed acyclic graph (DAG).

Think of it like an n8n workflow or Argo Workflow - visual automation.

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-response
spec:
  trigger:
    type: Webhook
    config:
      path: /pagerduty

  nodes:
    - id: diagnose
      type: Agent
      config:
        agent: diagnostic-agent

    - id: notify-slack
      type: Slack
      config:
        channel: "#incidents"

  connections:
    - from: diagnose
      to: notify-slack
```

### When to Use
- Event-driven automation (webhooks, schedules, file changes)
- Multi-step workflows with conditional logic
- Integration with external systems (Slack, PagerDuty, GitHub)
- Human-in-the-loop approval flows

### Node Types

| Node Type | Description | Example Use Case |
|-----------|-------------|------------------|
| `Agent` | Run an AI agent | Diagnose incident, write code |
| `Fleet` | Run agent fleet | Parallel code review |
| `HTTP` | Make HTTP request | Call external API |
| `Shell` | Execute command | Run kubectl, git |
| `Slack` | Send Slack message | Notify team |
| `GitHub` | GitHub automation | Create PR, add comment |
| `Conditional` | If/else logic | Route based on severity |
| `Transform` | Data transformation | Format output |
| `HumanApproval` | Wait for approval | Critical actions |

### Trigger Types

| Trigger | Description | Example |
|---------|-------------|---------|
| `Webhook` | HTTP endpoint | PagerDuty, GitHub webhooks |
| `Schedule` | Cron schedule | Daily reports, health checks |
| `FileWatch` | File changes | Config updates |
| `Manual` | CLI invocation | Ad-hoc runs |
| `Slack` | Slack events | Bot mentions |
| `GitHub` | GitHub events | PR created, issue opened |
| `PagerDuty` | PagerDuty events | Incidents triggered |
| `Kafka` | Kafka messages | Event streaming |

## Tools

Tools extend what agents can do. AOF supports three types:

### 1. Built-in Tools
Pre-configured tools that work out of the box:

- **Shell**: Execute terminal commands
- **HTTP**: Make HTTP/REST requests
- **FileSystem**: Read/write files

```yaml
tools:
  - type: Shell
    config:
      allowed_commands: ["kubectl", "helm"]

  - type: HTTP
    config:
      base_url: https://api.github.com
      headers:
        Authorization: "token ${GITHUB_TOKEN}"
```

### 2. MCP Servers
Model Context Protocol servers for specialized functionality:

- **kubectl-mcp**: Kubernetes operations
- **github-mcp**: GitHub API access
- **postgres-mcp**: Database queries

```yaml
tools:
  - type: MCP
    config:
      server: kubectl-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
```

### 3. Custom Integrations
Platform-specific integrations:

- Slack
- PagerDuty
- Jira
- Datadog

```yaml
tools:
  - type: Slack
    config:
      token: ${SLACK_BOT_TOKEN}
```

## Models

AOF supports multiple LLM providers. Use the format `provider:model`:

### OpenAI
```yaml
model: openai:gpt-4
model: openai:gpt-4-turbo
model: openai:gpt-3.5-turbo
```

### Anthropic
```yaml
model: anthropic:claude-3-5-sonnet-20241022
model: anthropic:claude-3-5-haiku-20241022
model: anthropic:claude-3-opus-20240229
```

### Ollama (Local)
```yaml
model: ollama:llama3
model: ollama:mistral
model: ollama:codellama
```

### Groq
```yaml
model: groq:llama-3.1-70b-versatile
model: groq:mixtral-8x7b-32768
```

### Provider Environment Variables

| Provider | Environment Variable |
|----------|---------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Groq | `GROQ_API_KEY` |
| Ollama | None (runs locally) |

## Memory

Memory lets agents remember conversation context across sessions.

### Memory Types

| Type | Description | Use Case |
|------|-------------|----------|
| `InMemory` | RAM-based (default) | Testing, short sessions |
| `File` | JSON file storage | Development, small scale |
| `SQLite` | Embedded database | Production, single instance |
| `PostgreSQL` | External database | Production, multi-instance |

### Example
```yaml
spec:
  memory:
    type: SQLite
    config:
      path: ./agent-memory.db

  # OR PostgreSQL for production
  memory:
    type: PostgreSQL
    config:
      url: postgres://user:pass@localhost/aof
```

## YAML Structure

All AOF resources follow Kubernetes-style structure:

```yaml
apiVersion: aof.dev/v1          # API version
kind: Agent                     # Resource type (Agent, AgentFleet, AgentFlow)

metadata:                       # Resource metadata
  name: my-resource             # Unique identifier
  labels:                       # Key-value labels
    env: production
    team: platform
  annotations:                  # Additional metadata
    description: "My agent"

spec:                          # Resource specification
  # Resource-specific configuration
```

## kubectl-Style CLI

AOF's CLI mirrors kubectl for familiarity:

```bash
# Apply configuration
aofctl apply -f agent.yaml

# List all agents
aofctl get agents

# Get specific agent
aofctl get agent my-agent

# Describe details
aofctl describe agent my-agent

# View logs
aofctl logs agent my-agent

# Run interactive chat
aofctl run agent my-agent

# Delete resource
aofctl delete agent my-agent
```

## Next Steps

Now that you understand the concepts, try building something:

- **[Your First Agent Tutorial](tutorials/first-agent.md)** - Hands-on guide
- **[Agent YAML Reference](reference/agent-spec.md)** - Complete spec docs
- **[Example Agents](examples/)** - Copy-paste configurations

## Quick Comparison

| Feature | Agent | AgentFleet | AgentFlow |
|---------|-------|------------|-----------|
| **Use Case** | Single task | Parallel tasks | Complex workflows |
| **Complexity** | Simple | Medium | Advanced |
| **K8s Analog** | Pod | Deployment | Workflow/Pipeline |
| **Example** | Code review | Multi-reviewer | Incident response |
| **Triggers** | Manual/CLI | Manual/CLI | Webhooks, schedules |

---

**Ready to build?** → [First Agent Tutorial](tutorials/first-agent.md)
