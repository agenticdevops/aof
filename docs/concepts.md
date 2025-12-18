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
  model: google:gemini-2.5-flash
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
| `model` | LLM to use | `google:gemini-2.5-flash`, `google:gemini-2.5-flash` |
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
      model: google:gemini-2.5-flash
      instructions: "Focus on security vulnerabilities"

    - name: performance-reviewer
      model: google:gemini-2.5-flash
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

An **AgentFlow** is an event-driven workflow that orchestrates agents, tools, and integrations in a directed graph.

Think of it like an n8n workflow or Argo Workflow - trigger-based automation. Unlike step-based Workflows, AgentFlows are designed for real-time event handling (Slack bots, webhooks, scheduled jobs).

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-bot-flow
spec:
  trigger:
    type: Slack
    config:
      events:
        - app_mention
        - message
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  nodes:
    - id: parse-message
      type: Transform
      config:
        script: |
          export MESSAGE_TEXT="${event.text}"
          export SLACK_CHANNEL="${event.channel}"

    - id: agent-process
      type: Agent
      config:
        agent: my-assistant
        input: ${MESSAGE_TEXT}

    - id: send-response
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        message: ${agent-process.output}

  connections:
    - from: trigger
      to: parse-message
    - from: parse-message
      to: agent-process
    - from: agent-process
      to: send-response
```

### When to Use
- Chat platform bots (Slack, Discord, Telegram, WhatsApp)
- Webhook-driven automation
- Scheduled jobs and reports
- Human-in-the-loop approval flows with reactions
- Multi-step workflows with conditional routing
- **Multi-tenant bot deployments** - Route different channels to different agents/clusters

### Node Types

| Node Type | Description | Example Use Case |
|-----------|-------------|------------------|
| `Agent` | Run an AI agent | Diagnose incident, write code |
| `Transform` | Data transformation & variable export | Parse incoming events, format output |
| `Conditional` | If/else logic with expressions | Route based on approval status |
| `Slack` | Send Slack messages, wait for reactions | Notify team, request approval |
| `Discord` | Send Discord messages | Notify Discord channels |
| `HTTP` | Make HTTP requests | Call external APIs |
| `Wait` | Pause execution | Cooldown periods |
| `Parallel` | Fork into multiple branches | Run concurrent checks |
| `Join` | Wait for parallel branches | Aggregate results |
| `Approval` | Human approval gate | Critical action confirmation |
| `End` | Terminal node | Mark flow completion |

### Trigger Types

| Trigger | Description | Example |
|---------|-------------|---------|
| `Slack` | Slack events (mentions, DMs, slash commands) | Bot interactions |
| `Discord` | Discord bot events | Message/command handling |
| `Telegram` | Telegram bot events | Chat messages |
| `WhatsApp` | WhatsApp Business API | Customer messaging |
| `HTTP` | Generic webhook endpoint | External integrations |
| `Schedule` | Cron-based scheduled execution | Daily reports, health checks |
| `Manual` | CLI invocation | Ad-hoc runs, testing |

### Multi-Tenant Routing

AgentFlows support **trigger filtering** for multi-tenant bot deployments. Route messages to different flows based on:

- **Channel** - Route `#production` to prod-cluster agent, `#staging` to staging-cluster
- **User** - Restrict flows to specific users (admins, SRE team)
- **Pattern** - Match commands like `kubectl`, `deploy`, `scale`

```yaml
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention]
      channels: [production, prod-alerts]  # Only these channels
      users: [U012ADMIN]                   # Only these users
      patterns: ["^(kubectl|k8s|deploy)"]  # Only matching messages

  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: default
    cluster: prod-cluster
    env:
      REQUIRE_APPROVAL: "true"
```

Each flow can specify its own **execution context** (kubeconfig, namespace, environment variables), enabling a single daemon to serve multiple clusters.

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

### Google (Recommended)
```yaml
model: google:gemini-2.5-flash
model: google:gemini-2.0-flash
model: google:gemini-1.5-pro
```

### OpenAI
```yaml
model: openai:gpt-4o
model: openai:gpt-4o-mini
model: openai:gpt-4-turbo
```

### Anthropic
```yaml
model: anthropic:claude-3-5-sonnet-20241022
model: anthropic:claude-3-haiku-20240307
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
| Google | `GOOGLE_API_KEY` |
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

AOF's CLI mirrors kubectl for familiarity using verb-noun syntax:

```bash
# Apply configuration
aofctl apply -f agent.yaml

# Get resources
aofctl get agents
aofctl get agent my-agent

# Describe details
aofctl describe agent my-agent

# Run an agent
aofctl run agent agent.yaml -i "What pods are failing?"

# View logs
aofctl logs agent my-agent

# Delete resource
aofctl delete agent my-agent

# List all resource types
aofctl api-resources
```

## Next Steps

Now that you understand the concepts, try building something:

- **[Your First Agent Tutorial](tutorials/first-agent.md)** - Hands-on guide
- **[Agent YAML Reference](reference/agent-spec.md)** - Complete spec docs
- **[AgentFlow Routing Guide](guides/agentflow-routing.md)** - How message routing works
- **[Multi-Tenant Architecture](architecture/multi-tenant-agentflows.md)** - Enterprise deployments
- **[Example Agents](examples/)** - Copy-paste configurations

## Quick Comparison

| Feature | Agent | AgentFleet | AgentFlow |
|---------|-------|------------|-----------|
| **Use Case** | Single task | Parallel tasks | Event-driven workflows |
| **Complexity** | Simple | Medium | Advanced |
| **K8s Analog** | Pod | Deployment | Argo Workflow |
| **Example** | Code review | Multi-reviewer | Slack bot, incident response |
| **Triggers** | Manual/CLI | Manual/CLI | Slack, Discord, HTTP, Schedule |
| **Spec Type** | `kind: Agent` | `kind: AgentFleet` | `kind: AgentFlow` |
| **Multi-Tenant** | No | No | Yes (channel/user/pattern routing) |

---

**Ready to build?** → [First Agent Tutorial](tutorials/first-agent.md)
