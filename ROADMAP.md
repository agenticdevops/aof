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
│  │ Jira        │          │ Join         │      │ File       │  │
│  │ HTTP        │          │ Wait         │      │ ...        │  │
│  │ Schedule    │          │ Approval     │      └────────────┘  │
│  │ PagerDuty   │          │ Loop         │                       │
│  │ Opsgenie    │          │ ...          │                       │
│  └─────────────┘          └──────────────┘                       │
│                                                                  │
│  AGENTS                    MEMORY                TOOLS           │
│  ┌─────────────┐          ┌──────────────┐      ┌────────────┐  │
│  │ LLM Provider│          │ InMemory     │      │ Shell      │  │
│  │ Instructions│          │ File         │      │ HTTP       │  │
│  │ Context     │          │ SQLite       │      │ FileSystem │  │
│  │ Tools       │          │ Redis        │      │ MCP        │  │
│  └─────────────┘          └──────────────┘      │ kubectl    │  │
│                                                  │ Grafana    │  │
│                                                  │ Datadog    │  │
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

## Current Release: v0.3.2-beta

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
| Agent Library (30 pre-built agents) | ✅ Complete | K8s, Observability, Incident, CI/CD, Security, Cloud |

#### Trigger Types
| Trigger | Status | Notes |
|---------|--------|-------|
| Slack | ✅ Complete | app_mention, message, slash_command |
| HTTP/Webhook | ✅ Complete | POST/GET with variable access |
| Schedule (Cron) | ✅ Complete | With timezone support |
| Manual (CLI) | ✅ Complete | `aofctl run` |
| Telegram | ✅ Complete | Messages, inline keyboards |
| WhatsApp | ✅ Complete | Messages, interactive buttons |
| GitHub | ✅ Complete | PR, Issues, Push, Reviews |
| Jira | ✅ Complete | Issues, Comments, Automation webhooks |
| GitLab | ✅ Complete | MR, Issues, Push |
| Bitbucket | ✅ Complete | PR, Push |
| PagerDuty | ✅ Complete | Incidents, alerts |
| Opsgenie | ✅ Complete | Alerts, on-call |
| Discord | ⚠️ Partial | message_create events only |
| ServiceNow | 🔄 Planned | [Issue #48] |
| Kafka | 🔄 Planned | Event streaming |
| SQS | 🔄 Planned | AWS queue integration |

#### Tools
| Tool | Status | Notes |
|------|--------|-------|
| Shell | ✅ Complete | Command execution |
| HTTP | ✅ Complete | REST API calls |
| FileSystem | ✅ Complete | File operations |
| MCP | ✅ Complete | Model Context Protocol |
| Grafana | ✅ Complete | Dashboards, alerts |
| Datadog | ✅ Complete | Metrics, monitors |
| Prometheus | ✅ Complete | Metrics queries |
| Loki | ⚠️ Partial | Basic log queries |
| Jaeger | 🔄 Planned | [Issue #50] |
| Jenkins | 🔄 Planned | [Issue #55] |

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
| Built-in commands | ✅ Complete | /help, /agent, /fleet menus |
| Stale message filtering | ✅ Complete | Drops old queued messages |
| Config hot-reload | 🔄 Planned | [Issue #22] |

---

## Roadmap by Priority

### P0 - Current Focus (v0.3.3)
- [x] Slack approval workflow
- [x] Conversation memory
- [x] Multi-tenant routing
- [x] GitHub/Jira triggers
- [ ] **Structured I/O Schemas** - Standardize agent outputs ([#74], [#75], [#76])
- [ ] **MCP Server Catalog** - Document available integrations ([#71])

### P1 - Developer Experience
- [ ] Structured output schemas for agents
- [ ] MCP server catalog documentation
- [ ] More real-world flow examples
- [ ] Improved error messages (serde_path_to_error)

### P2 - Enterprise Features
- [ ] **Horizontal scaling** - Redis/NATS message queue ([#47])
- [ ] **Multi-org support** - Per-org credentials ([#46])
- [ ] **Config hot-reload** - No restart updates ([#22])
- [ ] **ServiceNow trigger** - Enterprise ITSM ([#48])

### P3 - Additional Integrations
- [ ] Kafka trigger
- [ ] SQS trigger
- [ ] Jaeger tool ([#50])
- [ ] Jenkins tool ([#55])
- [ ] Loki enhancement ([#49])
- [ ] Loop node for batch operations

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
| `flows/github/pr-review-flow.yaml` | GitHub | PR review automation |
| `flows/github/issue-triage-flow.yaml` | GitHub | Issue labeling |

### Planned Examples
| Example | Requires | Status |
|---------|----------|--------|
| `incident-auto-remediation-flow.yaml` | PagerDuty, Fleet | Planned |
| `daily-standup-report-flow.yaml` | Cron, Fleet, Jira | Planned |
| `cost-optimization-flow.yaml` | Schedule, Fleet | Planned |

---

## GitHub Issues

Track progress on GitHub: https://github.com/agenticdevops/aof/issues

### Open Issues
| Issue | Title | Priority |
|-------|-------|----------|
| [#22](https://github.com/agenticdevops/aof/issues/22) | Config hot-reload | P2 |
| [#46](https://github.com/agenticdevops/aof/issues/46) | Multi-org support | P1 |
| [#47](https://github.com/agenticdevops/aof/issues/47) | Horizontal scaling | P1 |
| [#48](https://github.com/agenticdevops/aof/issues/48) | ServiceNow trigger | P2 |
| [#49](https://github.com/agenticdevops/aof/issues/49) | Loki enhancement | P1 |
| [#50](https://github.com/agenticdevops/aof/issues/50) | Jaeger tool | P2 |
| [#55](https://github.com/agenticdevops/aof/issues/55) | Jenkins tool | P2 |
| [#71](https://github.com/agenticdevops/aof/issues/71) | MCP Server Catalog | P0 |
| [#74](https://github.com/agenticdevops/aof/issues/74) | Structured I/O | P0 |

### Recently Closed
| Issue | Title | Release |
|-------|-------|---------|
| [#78](https://github.com/agenticdevops/aof/issues/78) | Grafana tool | v0.3.0 |
| [#79](https://github.com/agenticdevops/aof/issues/79) | PagerDuty trigger | v0.3.0 |
| [#80](https://github.com/agenticdevops/aof/issues/80) | Datadog tool | v0.3.0 |
| [#81](https://github.com/agenticdevops/aof/issues/81) | Incident agents | v0.3.0 |
| [#82](https://github.com/agenticdevops/aof/issues/82) | Opsgenie trigger | v0.3.0 |
| [#98](https://github.com/agenticdevops/aof/issues/98) | Jira Automation | v0.3.3 |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on submitting features and fixes.

When implementing a new feature:
1. Update this ROADMAP.md
2. Update CHANGELOG.md
3. Add/update relevant documentation in `docs/`
4. Add examples if applicable
