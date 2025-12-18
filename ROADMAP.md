# AOF Roadmap

This document tracks feature implementation status for AOF (Agentic Ops Framework).

## Architecture Philosophy: Lego Blocks for Agentic Automation

AOF is designed as a **modular, pluggable framework** where components are reusable building blocks. Users combine these "lego pieces" to build their own agentic automations.

### Core Building Blocks

```
┌─────────────────────────────────────────────────────────────────┐
│                         AOF FRAMEWORK                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TRIGGERS (Input)          NODES (Processing)    OUTPUTS         │
│  ┌─────────────┐          ┌──────────────┐      ┌────────────┐  │
│  │ Slack       │──────────│ Transform    │──────│ Slack      │  │
│  │ WhatsApp    │          │ Agent        │      │ Discord    │  │
│  │ Telegram    │          │ Conditional  │      │ HTTP       │  │
│  │ GitHub      │          │ Parallel     │      │ Email      │  │
│  │ HTTP        │          │ Join         │      │ File       │  │
│  │ Schedule    │          │ Wait         │      │ ...        │  │
│  │ PagerDuty   │          │ Approval     │      └────────────┘  │
│  │ Kafka       │          │ Loop         │                       │
│  │ ...         │          │ ...          │                       │
│  └─────────────┘          └──────────────┘                       │
│                                                                  │
│  AGENTS                    MEMORY                TOOLS           │
│  ┌─────────────┐          ┌──────────────┐      ┌────────────┐  │
│  │ LLM Provider│          │ InMemory     │      │ Shell      │  │
│  │ Instructions│          │ File         │      │ HTTP       │  │
│  │ Context     │          │ SQLite       │      │ FileSystem │  │
│  │ Tools       │          │ Redis        │      │ MCP        │  │
│  └─────────────┘          └──────────────┘      │ kubectl    │  │
│                                                  └────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                    ORCHESTRATION                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ AgentFlow (Workflow)  │  AgentFleet (Multi-Agent)           ││
│  │ - nodes + connections │  - Parallel execution               ││
│  │ - Multi-tenant routing│  - Coordination patterns            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Pluggable Triggers** - Add new event sources without changing core
2. **Composable Nodes** - Chain processing steps in any order
3. **Swappable Agents** - Use any LLM provider with consistent interface
4. **Extensible Tools** - Add custom tools via MCP or built-in
5. **Flexible Memory** - Choose backend based on use case
6. **YAML-First** - Declarative configuration for everything

### Adding a New Component

Each component type has a trait/interface that new implementations must follow:

```rust
// Trigger trait - implement for new event sources
trait TriggerPlatform {
    async fn start(&self) -> Result<()>;
    async fn handle_event(&self, event: TriggerEvent) -> Result<()>;
}

// Node trait - implement for new processing steps
trait NodeExecutor {
    async fn execute(&self, ctx: &NodeContext) -> Result<NodeResult>;
}
```

---

## Current Release: v0.1.15

### Implemented Features

#### Core Engine
| Feature | Status | Notes |
|---------|--------|-------|
| Agent execution with multiple LLM providers | ✅ Complete | OpenAI, Anthropic, Google, Ollama, Groq |
| MCP (Model Context Protocol) server support | ✅ Complete | Client implementation |
| Memory backends | ✅ Complete | InMemory, File, SQLite |
| Built-in tools (Shell, HTTP, FileSystem) | ✅ Complete | |
| AgentFleet multi-agent coordination | ✅ Complete | |
| AgentFlow workflow orchestration | ✅ Complete | v1 schema with nodes/connections |

#### Trigger Types
| Trigger | Status | Priority | Notes |
|---------|--------|----------|-------|
| Slack | ✅ Complete | - | app_mention, message, slash_command |
| HTTP/Webhook | ✅ Complete | - | POST/GET with variable access |
| Schedule (Cron) | ✅ Complete | - | With timezone support |
| Manual (CLI) | ✅ Complete | - | `aofctl run` |
| Discord | ⚠️ Partial | P2 | message_create events only |
| Telegram | 🔄 Planned | P1 | [Issue #24](https://github.com/agenticdevops/aof/issues/24) |
| WhatsApp | 🔄 Planned | P1 | [Issue #23](https://github.com/agenticdevops/aof/issues/23) |
| GitHub | 🔄 Planned | P1 | [Issue #25](https://github.com/agenticdevops/aof/issues/25) |
| PagerDuty | 🔄 Planned | P3 | [Issue #26](https://github.com/agenticdevops/aof/issues/26) |
| Kafka | 🔄 Planned | P3 | Issue TBD |
| SQS | 🔄 Planned | P3 | Issue TBD |

#### Node Types (AgentFlow)
| Node | Status | Notes |
|------|--------|-------|
| Transform | ✅ Complete | Script execution with variable extraction |
| Agent | ✅ Complete | AI agent execution with context |
| Conditional | ✅ Complete | Route-based branching |
| Slack | ✅ Complete | Message sending, threads, reactions |
| Discord | ✅ Complete | Message sending |
| HTTP | ⚠️ Partial | Basic framework, needs full implementation |
| Wait | ✅ Complete | Duration-based delays |
| Parallel | ✅ Complete | Fan-out execution |
| Join | ✅ Complete | Merge strategies (all/any/majority) |
| Approval | ✅ Complete | Human approval gates |
| End | ✅ Complete | Terminal node |
| Loop | 🔄 Planned | Iteration over collections |

#### Platform Features
| Feature | Status | Notes |
|---------|--------|-------|
| Slack reactions approval | ✅ Complete | ✅/❌ approve/deny |
| Slack approval whitelist | ✅ Complete | `approval_allowed_users` |
| Bot self-approval prevention | ✅ Complete | Auto-detects bot_user_id |
| Conversation memory | ✅ Complete | Per-channel/thread isolation |
| Multi-tenant routing | ✅ Complete | FlowRouter with priorities |
| Config hot-reload | 🔄 Planned | [Issue #22] |

---

## Roadmap by Priority

### P0 - Critical (Current Sprint)
- [x] Slack approval workflow
- [x] Conversation memory
- [x] Multi-tenant routing
- [ ] Fix/organize flow examples

### P1 - High Priority (Individual Users)
- [ ] **WhatsApp trigger** - Interactive buttons for approval
- [ ] **Telegram trigger** - Inline keyboards for bots
- [ ] **GitHub trigger** - PR/Issue webhooks
- [ ] Tutorial documentation for individual users

### P2 - Medium Priority
- [ ] Discord full implementation
- [ ] Loop node for batch operations
- [ ] HTTP node full implementation
- [ ] State persistence (checkpointing)

### P3 - Lower Priority (Enterprise/SRE)
- [ ] PagerDuty trigger
- [ ] Kafka trigger
- [ ] SQS trigger
- [ ] AgentFleet integration in flows (v1alpha1 syntax)

---

## Schema Versions

### v1 (Current - Implemented)
The current production schema using `nodes:` and `connections:` arrays:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
spec:
  trigger:
    type: Slack
    config: { ... }
  nodes:
    - id: node-1
      type: Transform
      config: { ... }
  connections:
    - from: trigger
      to: node-1
```

### v1alpha1 (Planned - Future DSL)
A more declarative syntax with fleet integration (not yet implemented):

```yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFlow
spec:
  trigger:
    type: github
  fleet:
    name: code-review-team
  actions:
    - type: github_review
      config: { ... }
```

---

## Example Status

### Working Examples (v1 schema)
| Example | Triggers | Notes |
|---------|----------|-------|
| `slack-k8s-bot-flow.yaml` | Slack | Full workflow with approval |
| `multi-tenant/slack-prod-k8s-bot.yaml` | Slack | Channel filtering |
| `multi-tenant/slack-staging-k8s-bot.yaml` | Slack | Environment context |
| `multi-tenant/slack-dev-local-bot.yaml` | Slack | Local development |

### Planned Examples (v1alpha1 schema)
These examples demonstrate future syntax and require additional implementation:

| Example | Requires | Status |
|---------|----------|--------|
| `planned/incident-auto-remediation-flow.yaml` | PagerDuty, Fleet | Planned |
| `planned/pr-review-flow.yaml` | GitHub, Fleet | Planned |
| `planned/daily-standup-report-flow.yaml` | Cron, Fleet, Jira | Planned |
| `planned/slack-qa-bot-flow.yaml` | Inline agent spec | Planned |
| `planned/cost-optimization-flow.yaml` | Schedule, Fleet | Planned |
| `planned/deploy-notification-flow.yaml` | GitHub, Fleet | Planned |

---

## GitHub Issues

Track progress on GitHub: https://github.com/agenticdevops/aof/issues

| Issue | Title | Priority | Labels |
|-------|-------|----------|--------|
| [#22](https://github.com/agenticdevops/aof/issues/22) | Config hot-reload | P2 | enhancement |
| [#23](https://github.com/agenticdevops/aof/issues/23) | WhatsApp trigger support | P1 | enhancement |
| [#24](https://github.com/agenticdevops/aof/issues/24) | Telegram trigger support | P1 | enhancement |
| [#25](https://github.com/agenticdevops/aof/issues/25) | GitHub webhook trigger | P1 | enhancement |
| [#26](https://github.com/agenticdevops/aof/issues/26) | PagerDuty trigger | P3 | enhancement |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on submitting features and fixes.

When implementing a new feature:
1. Update this ROADMAP.md
2. Update CHANGELOG.md
3. Add/update relevant documentation in `docs/`
4. Add examples if applicable
