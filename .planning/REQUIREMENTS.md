# Requirements: OpenAgentiX v2.0

**Defined:** 2026-03-12
**Core Value:** Let any technical organization automate operational tasks with AI agents — without writing Python, without managing infrastructure, without giving up control.

## v2.0 Requirements

### Core Runtime

- [x] **CORE-01**: Agent executes a ReAct loop (plan → act → observe → reflect) with configurable max iterations
- [x] **CORE-02**: Agent can call tools and receive structured results within the loop
- [x] **CORE-03**: Agent supports streaming responses (SSE) for real-time output
- [ ] **CORE-04**: Agent can use memory (recall context from previous runs)
- [ ] **CORE-05**: Smart model routing scores message complexity (0-100) and routes to appropriate model tier (flash/standard/pro)
- [ ] **CORE-06**: Agent supports research phase — proactive fact-gathering before response generation to reduce hallucinations

### Agent Specification

- [x] **SPEC-01**: User can define an agent in YAML with `apiVersion: openagentix.dev/v1`, `kind: Agent`, `metadata`, `spec`
- [x] **SPEC-02**: Agent spec supports `model`, `mode` (manual/semi-autonomous/autonomous), `skills`, `tools`, `mcp_servers`, `triggers`, `notifications`, `approval`, `budget`, `telemetry`
- [x] **SPEC-03**: Existing AOF agent YAML configs load with backward compatibility
- [x] **SPEC-04**: Agent spec supports `namespace` for team-scoped isolation
- [x] **SPEC-05**: Agent spec supports `version` for configuration versioning

### Skills Composition

- [ ] **SKILL-01**: User can compose an agent with multiple skills from a registry (e.g., `aws/rds-management`, `database/postgres-tuning`)
- [ ] **SKILL-02**: Skills are YAML/Markdown files with instructions, tool requirements, and domain expertise
- [ ] **SKILL-03**: Skills are selected deterministically by keyword/pattern matching (no LLM call needed)
- [ ] **SKILL-04**: User can create custom skills in their workspace
- [ ] **SKILL-05**: Built-in skill packs ship with the binary: aws, kubernetes, terraform, docker, git, database, security, observability

### Tools & Integrations

- [ ] **TOOL-01**: Agent can use CLI tools (kubectl, aws, psql, terraform, docker, git, shell)
- [ ] **TOOL-02**: Agent can connect to MCP servers (stdio, SSE, HTTP transports)
- [ ] **TOOL-03**: User can define custom tools as shell commands with structured input/output
- [ ] **TOOL-04**: WASM sandbox isolates untrusted tool execution with capability-based permissions
- [ ] **TOOL-05**: WASM tools declare required permissions (HTTP endpoints, secrets, resource limits) in a capabilities manifest

### Triggers

- [ ] **TRIG-01**: User can schedule an agent to run on a cron expression (e.g., `"0 9 * * 1"`)
- [ ] **TRIG-02**: Agent can be triggered by a webhook (HTTP POST with event payload)
- [ ] **TRIG-03**: Agent can be triggered by GitHub events (PR opened, push, tag) via webhook receiver
- [ ] **TRIG-04**: Agent can be triggered by Jira events (issue updated, comment created, agent mentioned)
- [ ] **TRIG-05**: Agent can be triggered by a message mention on Slack/Discord/Telegram (e.g., `@dba-optimizer`)
- [ ] **TRIG-06**: Agent can be triggered by another agent (coordinator delegates to specialist via agent trigger)
- [ ] **TRIG-07**: Agent can be triggered manually via CLI (`agentix run agent.yaml`)
- [ ] **TRIG-08**: Triggers are a pluggable trait — new trigger types can be added without modifying core
- [ ] **TRIG-09**: Each trigger delivers a `TriggerEvent` with source, payload, and context to the agent
- [ ] **TRIG-10**: Agent runs persist results and can be viewed later (`agentix runs`)
- [ ] **TRIG-11**: User can configure notification routing per trigger (where results are sent)

### Cost Tracking & Budgets

- [ ] **COST-01**: System tracks token usage (input/output) and cost per LLM call
- [ ] **COST-02**: System tracks cost per agent run (aggregated across all LLM calls in a run)
- [ ] **COST-03**: System tracks cost per agent (aggregated across all runs)
- [ ] **COST-04**: User can set a daily budget limit per agent in USD — agent stops when exceeded
- [ ] **COST-05**: User can set a max token limit per run — agent stops when exceeded
- [ ] **COST-06**: Cost data is queryable via CLI (`agentix costs`, `agentix costs agent <name>`)
- [ ] **COST-07**: System uses actual API cost from provider response when available (not just estimates)

### Telemetry & Observability

- [ ] **TELE-01**: Agent runs produce OpenTelemetry traces (spans for each loop iteration, tool call, LLM call)
- [ ] **TELE-02**: Agent runs produce OpenTelemetry metrics (run duration, token usage, tool call count, success/failure)
- [ ] **TELE-03**: Traces and metrics are exportable to any OTel collector (Grafana, Datadog, Jaeger)
- [ ] **TELE-04**: Agent run logs are structured JSON with correlation IDs
- [ ] **TELE-05**: User can view agent traces via CLI (`agentix logs <agent> --trace`)

### Security

- [ ] **SEC-01**: WASM sandbox prevents untrusted tools from accessing filesystem, network, or secrets without explicit capability grants
- [ ] **SEC-02**: Capability-based permissions — each tool declares what it needs, agent grants only what's allowed
- [ ] **SEC-03**: Secrets are encrypted at rest (AES-256-GCM) and never exposed in logs or traces
- [ ] **SEC-04**: Audit trail logs every agent action (tool call, LLM call, approval decision) with timestamps
- [ ] **SEC-05**: SSRF protection blocks tool access to private IPs, localhost, and cloud metadata endpoints

### Approval Workflows

- [ ] **APPR-01**: User can mark specific actions as requiring approval in the agent spec
- [ ] **APPR-02**: When approval is required, agent pauses and sends notification to configured approver
- [ ] **APPR-03**: Approver can approve/deny via CLI, messaging channel, or command center
- [ ] **APPR-04**: Approval decisions are logged in the audit trail
- [ ] **APPR-05**: Agent supports three modes: `manual` (all actions need approval), `semi-autonomous` (only flagged actions), `autonomous` (no approval needed)

### Agent Coordination

- [ ] **COORD-01**: Each agent has a bounded inbox (message queue) for receiving work from other agents
- [ ] **COORD-02**: A coordinator agent can delegate tasks to specialist agents via `type: agent` trigger
- [ ] **COORD-03**: Specialist agent returns results to the coordinator that triggered it
- [ ] **COORD-04**: Coordinator can wait for multiple specialist results before synthesizing a response
- [ ] **COORD-05**: Agent-to-agent delegation is logged in the audit trail with full context

### Memory & Knowledge

- [ ] **MEM-01**: Agent can persist vector embeddings of past run contexts for semantic recall
- [ ] **MEM-02**: Agent can recall relevant context from previous runs using vector similarity search
- [ ] **MEM-03**: Memory store supports SQLite backend (default) with pluggable trait for other backends
- [ ] **MEM-04**: Agent memory is scoped per agent (isolated, not shared) unless explicitly configured

### Gateway (Multi-Channel)

- [ ] **GW-01**: Agent can receive messages from Slack and respond in the same channel
- [ ] **GW-02**: Agent can receive messages from Telegram and respond in the same chat
- [ ] **GW-03**: Agent can receive messages from Discord and respond in the same channel
- [ ] **GW-04**: Agent can send notifications to configured channels (Slack, Telegram, Discord, email, webhook)
- [ ] **GW-05**: Gateway supports bi-directional communication — trigger agent AND receive results
- [ ] **GW-06**: Per-channel configuration (which agents respond on which channels)

### CLI (`agentix`)

- [x] **CLI-01**: `agentix run <file>` runs an agent from a YAML definition (one-shot)
- [x] **CLI-02**: `agentix start <file>` starts an agent as a background daemon/scheduled process
- [x] **CLI-03**: `agentix agents` lists all running/registered agents with status
- [x] **CLI-04**: `agentix runs` shows run history with timestamps, duration, cost, status
- [x] **CLI-05**: `agentix logs <agent>` shows agent output logs
- [x] **CLI-06**: `agentix stop <agent>` stops a running agent
- [x] **CLI-07**: `agentix apply -f <file>` creates or updates an agent definition
- [ ] **CLI-08**: `agentix skills list` shows available skill packs
- [ ] **CLI-09**: `agentix costs` shows cost summary across all agents
- [x] **CLI-10**: `agentix serve` starts the server (API + gateway + scheduler)
- [x] **CLI-11**: `agentix init` scaffolds a new agent YAML with interactive prompts
- [x] **CLI-12**: `agentix validate <file>` validates agent YAML without running

### Command Center (Svelte)

- [x] **CMD-01**: Web dashboard shows all registered agents with status (running/idle/error/scheduled)
- [x] **CMD-02**: Agent detail view shows configuration, run history, costs, and logs
- [x] **CMD-03**: Run history view shows all runs with filtering by agent, status, date range
- [ ] **CMD-04**: Cost dashboard shows per-agent and aggregate cost charts over time
- [ ] **CMD-05**: Agent builder provides a visual YAML editor with validation and skill browser
- [ ] **CMD-06**: Scheduler view shows cron schedules and upcoming runs
- [ ] **CMD-07**: Trace viewer shows agent execution traces (tool calls, LLM calls, decisions)
- [ ] **CMD-08**: Approval queue shows pending approval requests with approve/deny actions
- [x] **CMD-09**: Real-time updates via WebSocket (agent status changes, run completions, cost updates)

## v2.1 Requirements

### Agent Lifecycle

- **LIFE-01**: User can version agent configurations and roll back to previous versions
- **LIFE-02**: User can run canary deployments (new agent version alongside old)
- **LIFE-03**: User can A/B test agent configurations

### Extended Channels

- **CHAN-01**: WhatsApp integration
- **CHAN-02**: Microsoft Teams integration
- **CHAN-03**: Email integration (IMAP/SMTP)
- **CHAN-04**: Signal integration
- **CHAN-05**: Matrix integration

### Self-Learning Agents

- **LEARN-01**: Agent builds knowledge graph (entities + relations) from past runs
- **LEARN-02**: SONA adaptive learning — agent improves responses based on feedback and outcomes
- **LEARN-03**: GNN query refinement for better memory recall accuracy
- **LEARN-04**: Agent tracks outcome quality and adjusts strategies over time

### Workflow Orchestration

- **WORK-01**: Multi-agent pipelines (sequential, fan-out, collect, conditional)
- **WORK-02**: Visual workflow builder in command center

### Pre-Built Agent Packs ("Hands")

- **HAND-01**: DBA Optimizer (RDS/PostgreSQL analysis and tuning)
- **HAND-02**: Cost Anomaly Detector (AWS/GCP/Azure cost analysis)
- **HAND-03**: Compliance Scanner (CIS benchmark checking)
- **HAND-04**: Security Scanner (vulnerability detection)
- **HAND-05**: Deployment Agent (canary rollouts with approval gates)

## v2.5 Requirements (Enterprise)

### Kubernetes Native

- **K8S-01**: Helm chart for cluster deployment
- **K8S-02**: K8s operator that watches Agent CRDs
- **K8S-03**: Agent/AgentRun as Custom Resource Definitions

### Enterprise Auth

- **AUTH-01**: RBAC with roles (admin, operator, viewer) and namespace-scoped permissions
- **AUTH-02**: SSO via SAML/OIDC
- **AUTH-03**: API key management

### Multi-Tenancy

- **TENANT-01**: Namespace isolation (agents, secrets, costs separated per team)
- **TENANT-02**: Per-namespace cost allocation and budgets

## Out of Scope

| Feature | Reason |
|---------|--------|
| Agent personas / SOUL.md | Pivoted away — enterprise focus, not personality-driven |
| React web app | Replaced by Svelte command center |
| Python SDK | Rust-native, no Python dependency is a differentiator |
| Public agent marketplace | Security/quality concerns — community registry in v2.1 |
| A2A protocol | Emerging standard, not stable enough for v2.0 |
| Voice/talk mode | Text-based sufficient for enterprise automation |
| Mobile native app | Web + messaging channels sufficient |
| Real-time collaborative editing | Not needed for agent automation |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 13 | Complete |
| CORE-02 | Phase 13 | Complete |
| CORE-03 | Phase 13 | Complete |
| CORE-04 | Phase 16 | Pending |
| CORE-05 | Phase 17 | Pending |
| CORE-06 | Phase 16 | Pending |
| SPEC-01 | Phase 13 | Complete |
| SPEC-02 | Phase 13 | Complete |
| SPEC-03 | Phase 13 | Complete |
| SPEC-04 | Phase 13 | Complete |
| SPEC-05 | Phase 13 | Complete |
| SKILL-01 | Phase 14 | Pending |
| SKILL-02 | Phase 14 | Pending |
| SKILL-03 | Phase 14 | Pending |
| SKILL-04 | Phase 14 | Pending |
| SKILL-05 | Phase 14 | Pending |
| TOOL-01 | Phase 14 | Pending |
| TOOL-02 | Phase 14 | Pending |
| TOOL-03 | Phase 14 | Pending |
| TOOL-04 | Phase 14 | Pending |
| TOOL-05 | Phase 14 | Pending |
| TRIG-01 | Phase 15 | Pending |
| TRIG-02 | Phase 15 | Pending |
| TRIG-03 | Phase 15 | Pending |
| TRIG-04 | Phase 15 | Pending |
| TRIG-05 | Phase 15 | Pending |
| TRIG-06 | Phase 15 | Pending |
| TRIG-07 | Phase 15 | Pending |
| TRIG-08 | Phase 15 | Pending |
| TRIG-09 | Phase 15 | Pending |
| TRIG-10 | Phase 15 | Pending |
| TRIG-11 | Phase 15 | Pending |
| COST-01 | Phase 17 | Pending |
| COST-02 | Phase 17 | Pending |
| COST-03 | Phase 17 | Pending |
| COST-04 | Phase 17 | Pending |
| COST-05 | Phase 17 | Pending |
| COST-06 | Phase 17 | Pending |
| COST-07 | Phase 17 | Pending |
| TELE-01 | Phase 18 | Pending |
| TELE-02 | Phase 18 | Pending |
| TELE-03 | Phase 18 | Pending |
| TELE-04 | Phase 18 | Pending |
| TELE-05 | Phase 18 | Pending |
| SEC-01 | Phase 19 | Pending |
| SEC-02 | Phase 19 | Pending |
| SEC-03 | Phase 19 | Pending |
| SEC-04 | Phase 19 | Pending |
| SEC-05 | Phase 19 | Pending |
| APPR-01 | Phase 20 | Pending |
| APPR-02 | Phase 20 | Pending |
| APPR-03 | Phase 20 | Pending |
| APPR-04 | Phase 20 | Pending |
| APPR-05 | Phase 20 | Pending |
| COORD-01 | Phase 16 | Pending |
| COORD-02 | Phase 16 | Pending |
| COORD-03 | Phase 16 | Pending |
| COORD-04 | Phase 16 | Pending |
| COORD-05 | Phase 16 | Pending |
| MEM-01 | Phase 16 | Pending |
| MEM-02 | Phase 16 | Pending |
| MEM-03 | Phase 16 | Pending |
| MEM-04 | Phase 16 | Pending |
| GW-01 | Phase 21 | Pending |
| GW-02 | Phase 21 | Pending |
| GW-03 | Phase 21 | Pending |
| GW-04 | Phase 21 | Pending |
| GW-05 | Phase 21 | Pending |
| GW-06 | Phase 21 | Pending |
| CLI-01 | Phase 13 | Complete |
| CLI-02 | Phase 13 | Complete |
| CLI-03 | Phase 13 | Complete |
| CLI-04 | Phase 13 | Complete |
| CLI-05 | Phase 13 | Complete |
| CLI-06 | Phase 13 | Complete |
| CLI-07 | Phase 13 | Complete |
| CLI-08 | Phase 14 | Pending |
| CLI-09 | Phase 17 | Pending |
| CLI-10 | Phase 13 | Complete |
| CLI-11 | Phase 13 | Complete |
| CLI-12 | Phase 13 | Complete |
| CMD-01 | Phase 22 | Complete |
| CMD-02 | Phase 22 | Complete |
| CMD-03 | Phase 22 | Complete |
| CMD-04 | Phase 22 | Pending |
| CMD-05 | Phase 22 | Pending |
| CMD-06 | Phase 22 | Pending |
| CMD-07 | Phase 22 | Pending |
| CMD-08 | Phase 22 | Pending |
| CMD-09 | Phase 22 | Complete |

**Coverage:**
- v2.0 requirements: 90 total (note: original estimate was 81; actual count after enumeration is 90)
- Mapped to phases: 90
- Unmapped: 0

---
*Requirements defined: 2026-03-12*
*Last updated: 2026-03-12 — traceability populated after roadmap creation*
