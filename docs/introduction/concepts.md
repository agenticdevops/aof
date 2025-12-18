# Core Concepts

AOF is designed as a **composable framework** where you build agentic automation by combining modular building blocks. Think of it like Lego - each piece has a specific purpose, and you snap them together to create powerful workflows.

## The 6 Resource Types

AOF provides six Kubernetes-style resource types that work together:

```
┌────────────────────────────────────────────────────────────┐
│                    AOF Resource Model                       │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │ Agent   │  │ Fleet   │  │ Flow    │  ← Execution       │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │ Context │  │ Trigger │  │FlowBindng│  ← Configuration  │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Quick Comparison

| Resource | Purpose | Example Use Case | Kubernetes Analog |
|----------|---------|------------------|-------------------|
| **Agent** | Single AI assistant | Code review, Q&A, diagnostics | Pod |
| **Fleet** | Team of agents | Multi-reviewer, parallel analysis | Deployment |
| **Flow** | Event-driven workflow | Slack bot, incident response | Argo Workflow |
| **Context** | Environment config | Prod/staging/dev settings | ConfigMap |
| **Trigger** | Event source | Slack messages, GitHub webhooks | CronJob trigger |
| **FlowBinding** | Wire everything together | Connect trigger → flow → agent | Binding |

## 1. Agent

An **Agent** is the smallest deployable unit - a single AI assistant with specific instructions, tools, and model configuration.

### When to Use
- ✅ Simple, focused tasks (code review, Q&A)
- ✅ Single-purpose automation
- ✅ Interactive chat sessions
- ✅ Quick prototyping

### Example
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    category: infrastructure
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Kubernetes operations expert. Help users manage clusters,
    diagnose issues, and provide best practice recommendations.
  tools:
    - kubectl
    - helm
    - shell
  memory:
    type: InMemory
    config:
      max_messages: 20
```

### Key Components

| Field | Description | Example |
|-------|-------------|---------|
| `model` | LLM provider and model | `google:gemini-2.5-flash`, `openai:gpt-4o` |
| `instructions` | System prompt / personality | "You are a K8s expert..." |
| `tools` | What agent can do | kubectl, shell, HTTP, MCP servers |
| `memory` | Conversation persistence | InMemory, File, SQLite, PostgreSQL |
| `temperature` | Response creativity (0-1) | 0.3 (precise) to 0.9 (creative) |
| `max_tokens` | Response length limit | 4096, 8192 |

## 2. Fleet

A **Fleet** is a team of agents working together on a shared task, either in parallel or with coordination.

### When to Use
- ✅ Complex tasks requiring multiple perspectives
- ✅ Parallel processing of data
- ✅ Consensus-building (multiple agents vote)
- ✅ Specialized expertise (security + performance + style)

### Example
```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-team
spec:
  agents:
    - name: security-reviewer
      role: worker
      replicas: 1
      spec:
        model: google:gemini-2.5-flash
        instructions: "Focus on security vulnerabilities and auth issues"

    - name: performance-reviewer
      role: worker
      replicas: 1
      spec:
        model: google:gemini-2.5-flash
        instructions: "Focus on performance bottlenecks and optimization"

    - name: style-reviewer
      role: worker
      replicas: 1
      spec:
        model: ollama:llama3
        instructions: "Focus on code style, readability, and best practices"

  coordination:
    mode: peer              # All agents as equals
    distribution: parallel   # Run all agents simultaneously
    consensus: majority     # Use majority voting for decisions
```

### Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `hierarchical` | Manager coordinates workers | Complex workflows with delegation |
| `peer` | All agents equal | Distributed analysis, voting |
| `swarm` | Self-organizing | Dynamic, adaptive workloads |

### How It Works
1. Submit a task to the fleet
2. Each agent processes it independently (or cooperatively)
3. Results are aggregated via consensus, summary, or all responses

## 3. Flow

A **Flow** is an event-driven workflow that orchestrates agents, tools, and integrations in a directed graph with conditional branching.

### When to Use
- ✅ Chat platform bots (Slack, Discord, Telegram, WhatsApp)
- ✅ Webhook-driven automation (GitHub, PagerDuty)
- ✅ Scheduled jobs and reports (cron-based)
- ✅ Human-in-the-loop approval workflows
- ✅ Multi-step pipelines with conditional routing
- ✅ Multi-tenant bot deployments (different channels → different agents)

### Example
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-bot
spec:
  trigger:
    type: Slack
    config:
      events: [app_mention, message]
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  nodes:
    - id: parse
      type: Transform
      config:
        script: |
          export MESSAGE="${event.text}"
          export CHANNEL="${event.channel}"
          export USER="${event.user}"

    - id: agent
      type: Agent
      config:
        agent: k8s-ops
        input: ${MESSAGE}

    - id: respond
      type: Slack
      config:
        channel: ${CHANNEL}
        thread_ts: ${event.ts}
        message: ${agent.output}

  connections:
    - from: trigger
      to: parse
    - from: parse
      to: agent
    - from: agent
      to: respond
```

### Node Types

| Node | Description | Example Use |
|------|-------------|-------------|
| `Transform` | Script execution, variable extraction | Parse events, format output |
| `Agent` | Run AI agent | Diagnose issue, write code |
| `Conditional` | If/else routing | Approval status branching |
| `Slack` | Slack messaging | Send messages, request approval |
| `Discord` | Discord messaging | Notify channels |
| `Telegram` | Telegram messaging | Bot interactions |
| `HTTP` | HTTP requests | Call external APIs |
| `Parallel` | Fork execution | Run concurrent checks |
| `Join` | Merge branches | Aggregate results |
| `Approval` | Human approval gate | Critical action confirmation |
| `Wait` | Pause execution | Cooldown, rate limiting |
| `End` | Terminal node | Mark completion |

### Trigger Types

| Trigger | Events | Example Use |
|---------|--------|-------------|
| `Slack` | app_mention, message, slash_command, reactions | Bot interactions, approvals |
| `Discord` | message, slash_command | Bot responses |
| `Telegram` | message, command | Chat automation |
| `WhatsApp` | message | Customer support |
| `HTTP` | Webhook POST/GET | Generic integrations |
| `Schedule` | Cron expression | Daily reports, health checks |
| `Manual` | CLI invocation | Testing, ad-hoc runs |

## 4. Context (Future)

A **Context** defines environment-specific configuration that can be shared across flows.

> **Status**: Planned for v1alpha2 - not yet implemented. Currently, context is embedded in AgentFlow specs.

### When to Use
- ✅ Separate prod/staging/dev configurations
- ✅ Multi-cluster Kubernetes setups
- ✅ Environment-specific credentials
- ✅ Reusable configuration across flows

### Planned Example
```yaml
apiVersion: aof.dev/v1alpha2
kind: Context
metadata:
  name: prod-cluster
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  cluster: prod-us-east-1
  env:
    REQUIRE_APPROVAL: "true"
    APPROVAL_TIMEOUT: "300"
    PAGERDUTY_ROUTING_KEY: ${PD_ROUTING_KEY_PROD}
```

## 5. Trigger (Future)

A **Trigger** is a standalone event source that can be referenced by multiple flows.

> **Status**: Planned for v1alpha2 - not yet implemented. Currently, triggers are embedded in AgentFlow specs.

### When to Use
- ✅ Share one Slack bot across multiple flows
- ✅ Reuse webhook endpoints
- ✅ Centralized platform configuration

### Planned Example
```yaml
apiVersion: aof.dev/v1alpha2
kind: Trigger
metadata:
  name: slack-prod-bot
spec:
  platform: slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
    channels: [production, prod-alerts]
```

## 6. FlowBinding (Future)

A **FlowBinding** connects a Trigger → Flow → Agent with a specific Context.

> **Status**: Planned for v1alpha2 - not yet implemented. Currently achieved through AgentFlow configuration.

### When to Use
- ✅ Wire together modular components
- ✅ Apply different contexts to same flow
- ✅ A/B testing with different agents
- ✅ Enterprise multi-tenant deployments

### Planned Example
```yaml
apiVersion: aof.dev/v1alpha2
kind: FlowBinding
metadata:
  name: prod-k8s-binding
spec:
  trigger:
    ref: slack-prod-bot           # Reference to Trigger resource
  flow:
    ref: k8s-ops-flow              # Reference to Flow resource
  agent:
    ref: k8s-ops                   # Reference to Agent resource
  context:
    ref: prod-cluster              # Reference to Context resource
```

## The Reference Syntax (`ref:`)

In the future composable architecture, resources reference each other using `ref:` fields:

```yaml
# Instead of embedding configuration
spec:
  trigger:
    type: Slack
    config: { ... }           # Inline

# You'll reference external resources
spec:
  trigger:
    ref: slack-prod-bot       # Reference
```

**Benefits:**
- ✨ Reusability - Define once, use many times
- ✨ Separation of concerns - Security, config, logic are separate
- ✨ Testability - Swap components for testing
- ✨ Multi-tenancy - One trigger, multiple flows

## Current vs. Future Architecture

### Current (v1) - Monolithic
```yaml
# Everything embedded in AgentFlow
apiVersion: aof.dev/v1
kind: AgentFlow
spec:
  trigger:
    type: Slack
    config: { ... }           # Inline trigger
  context:
    kubeconfig: ${KUBECONFIG}  # Inline context
  nodes:
    - type: Agent
      config:
        agent: k8s-ops        # Agent name only
```

### Future (v1alpha2) - Composable
```yaml
# Separate resources with references
---
apiVersion: aof.dev/v1alpha2
kind: Trigger
metadata:
  name: slack-prod
spec:
  platform: slack
  config: { ... }
---
apiVersion: aof.dev/v1alpha2
kind: Context
metadata:
  name: prod-cluster
spec:
  kubeconfig: ${KUBECONFIG}
---
apiVersion: aof.dev/v1alpha2
kind: FlowBinding
metadata:
  name: prod-k8s
spec:
  trigger: { ref: slack-prod }
  flow: { ref: k8s-ops-flow }
  agent: { ref: k8s-ops }
  context: { ref: prod-cluster }
```

## Composition Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Composable AOF                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐                                                │
│  │ Trigger  │───┐                                            │
│  │ (Slack)  │   │                                            │
│  └──────────┘   │                                            │
│                 │    ┌─────────────┐                         │
│  ┌──────────┐   ├───→│ FlowBinding │                         │
│  │ Context  │───┤    └──────┬──────┘                         │
│  │ (Prod)   │   │           │                                │
│  └──────────┘   │           ↓                                │
│                 │    ┌──────────┐    ┌──────────┐            │
│  ┌──────────┐   ├───→│   Flow   │───→│  Agent   │            │
│  │  Flow    │───┘    │ (k8s-ops)│    │ (k8s-ops)│            │
│  │          │        └──────────┘    └──────────┘            │
│  └──────────┘                                                │
│                                                              │
│  Result: Event from Slack → Flow processes → Agent acts     │
│          Using prod-cluster context                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Tools System

Tools extend what agents can do. AOF supports three types:

### 1. Built-in Tools (Unified CLI)

Pre-configured tools that work out of the box:

```yaml
tools:
  - kubectl    # Any kubectl command
  - git        # Any git command
  - docker     # Any docker command
  - helm       # Any helm command
  - terraform  # Any terraform command
  - shell      # General shell commands
  - http       # HTTP requests
```

**Benefits:**
- Simple - Just list the tool name
- Flexible - Supports any subcommand
- LLM-native - Model constructs commands intelligently

### 2. MCP Servers

Model Context Protocol servers for specialized functionality:

```yaml
tools:
  - type: MCP
    config:
      server: kubectl-mcp
      command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
      env:
        KUBECONFIG: ${KUBECONFIG}
```

**Available MCP Servers:**
- `@modelcontextprotocol/server-kubectl` - Kubernetes operations
- `@modelcontextprotocol/server-github` - GitHub API access
- `@modelcontextprotocol/server-postgres` - Database queries
- Community MCP servers from npm

### 3. Custom Integrations

Platform-specific integrations:

```yaml
tools:
  - type: Slack
    config:
      token: ${SLACK_BOT_TOKEN}
  - type: PagerDuty
    config:
      api_key: ${PAGERDUTY_API_KEY}
```

## Models

AOF supports multiple LLM providers with unified `provider:model` syntax:

| Provider | Model Format | Environment Variable |
|----------|--------------|---------------------|
| Google | `google:gemini-2.5-flash` | `GOOGLE_API_KEY` |
| OpenAI | `openai:gpt-4o` | `OPENAI_API_KEY` |
| Anthropic | `anthropic:claude-3-5-sonnet-20241022` | `ANTHROPIC_API_KEY` |
| Ollama | `ollama:llama3` | None (runs locally) |
| Groq | `groq:llama-3.1-70b-versatile` | `GROQ_API_KEY` |

**Recommendation**: Use `google:gemini-2.5-flash` for best cost/performance balance in DevOps workflows.

## Memory

Memory enables conversation context across sessions:

| Type | Description | Use Case |
|------|-------------|----------|
| `InMemory` | RAM-based (default) | Testing, short sessions |
| `File` | JSON file storage | Development, local use |
| `SQLite` | Embedded database | Production, single instance |
| `PostgreSQL` | External database | Production, multi-instance |

```yaml
spec:
  memory:
    type: SQLite
    config:
      path: ./agent-memory.db
```

## Multi-Tenant Routing

AgentFlows support **trigger filtering** for multi-tenant bot deployments:

```yaml
spec:
  trigger:
    type: Slack
    config:
      channels: [production, prod-alerts]  # Only these channels
      users: [U012ADMIN, U015SRE]          # Only these users
      patterns: ["^(kubectl|k8s|deploy)"]  # Only matching messages
```

**Use Cases:**
- Route `#production` to prod-cluster agent
- Route `#staging` to staging-cluster agent
- Restrict destructive commands to admin users
- Different contexts per environment

See [Multi-Tenant Architecture](../architecture/multi-tenant-agentflows.md) for enterprise deployment patterns.

## YAML Structure

All AOF resources follow Kubernetes-style structure:

```yaml
apiVersion: aof.dev/v1          # API version
kind: Agent                     # Resource type

metadata:                       # Resource metadata
  name: my-agent                # Unique identifier
  labels:                       # Optional labels
    env: production
    team: platform
  annotations:                  # Optional annotations
    description: "My agent"

spec:                          # Resource specification
  # Resource-specific configuration
```

## kubectl-Style CLI

AOF mirrors kubectl for familiarity:

```bash
# Apply configurations
aofctl apply -f agent.yaml

# Get resources
aofctl get agents
aofctl get agent k8s-ops

# Describe details
aofctl describe agent k8s-ops

# Run agents/flows
aofctl run agent agent.yaml -i "What pods are failing?"
aofctl run fleet fleet.yaml -i '{"task": "Review code"}'

# Serve flows
aofctl serve --flows-dir ./flows --agents-dir ./agents

# View logs
aofctl logs agent k8s-ops

# Delete resources
aofctl delete agent k8s-ops

# List resource types
aofctl api-resources
```

## When to Use Each Resource Type

| Scenario | Use This | Why |
|----------|----------|-----|
| Simple Q&A chatbot | **Agent** | Single-purpose, straightforward |
| Code review needing multiple perspectives | **Fleet** | Parallel analysis by specialized agents |
| Slack bot with approval workflow | **Flow** | Event-driven with human-in-the-loop |
| Multi-environment deployment (prod/staging) | **Flow + Context** | Separate configs per environment |
| Reusable Slack bot across teams | **Trigger + FlowBinding** | Share one bot, multiple flows |
| A/B testing different agents | **FlowBinding** | Swap agents without changing flow |

## Next Steps

Now that you understand the concepts, try building something:

- **[Quickstart Guide](quickstart.md)** - Your first agent in 5 minutes
- **[Multi-Tenant Setup](../guides/enterprise-setup.md)** - Enterprise deployment patterns
- **[Agent YAML Reference](../reference/agent-spec.md)** - Complete spec docs
- **[AgentFlow Reference](../reference/agentflow-spec.md)** - Flow specification
- **[Example Agents](../examples/)** - Copy-paste configurations

---

**Ready to build?** → [Quickstart Guide](quickstart.md)
