# Roadmap: OpenAgentiX v2.0

## Milestones

- ✅ **v1.0 Humanized Interfaces** - Phases 1-12 (shipped 2026-02-22)
- 🚧 **v2.0 OpenAgentiX** - Phases 13-22 (in progress)

## Phases

<details>
<summary>✅ v1.0 Humanized Interfaces (Phases 1-12) - SHIPPED 2026-02-22</summary>

12 phases, 55 plans, 164K LOC. Archives at `.planning/milestones/v1.0-ROADMAP.md`.

</details>

### 🚧 v2.0 OpenAgentiX

**Milestone Goal:** Transform AOF into an enterprise agent automation platform — GitAgent-compatible agent directories, skills + tools + MCP, scheduled/event-driven execution, cost tracking, and a Svelte command center.

- [x] **Phase 13: Rebrand + Core Runtime + CLI Foundation** - `agentix` binary, GitAgent-compatible agent directories, ReAct loop, streaming, backward compatibility (completed 2026-03-12)
- [x] **Phase 14: Skills Composition + Tools + WASM Sandbox** - Skill packs, composable agents, WASM-isolated tool execution (completed 2026-03-13)
- [x] **Phase 15: Triggers + Scheduling** - Cron, webhook, GitHub, Jira, agent-to-agent, run persistence (completed 2026-03-13)
- [x] **Phase 16: Agent Coordination + Memory** - Multi-agent delegation, vector memory, semantic recall, research phase (completed 2026-03-13)
- [x] **Phase 17: Cost Tracking + Budgets** - Per-call/run/agent cost tracking, daily budget limits, smart model routing (completed 2026-03-13)
- [x] **Phase 18: Telemetry + Observability** - OTel traces + metrics, structured logs, exportable to Grafana/Datadog (completed 2026-03-13)
- [x] **Phase 19: Security** - WASM capability enforcement, secret encryption, audit trail, SSRF protection (completed 2026-03-13)
- [x] **Phase 20: Approval Workflows** - Human-in-the-loop pause/resume, approval queue, three autonomy modes (completed 2026-03-13)
- [x] **Phase 21: Gateway (Multi-Channel)** - Slack, Telegram, Discord bi-directional, per-channel agent routing (completed 2026-03-13)
- [x] **Phase 22: Command Center (Svelte)** - Web dashboard, agent builder, cost charts, trace viewer, approval queue (completed 2026-03-13)

## Phase Details

### Phase 13: Rebrand + Core Runtime + CLI Foundation
**Goal**: Users can define agents as GitAgent-compatible directories (agent.yaml + SOUL.md), run them via `agentix gateway start`, see streaming ReAct loop output, and manage everything with the `agentix` CLI. Existing AOF flat YAML agents still load.
**Depends on**: v1.0 Rust core (carried over)
**Requirements**: CORE-01, CORE-02, CORE-03, SPEC-01, SPEC-02, SPEC-03, SPEC-04, SPEC-05, CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, CLI-07, CLI-10, CLI-11, CLI-12
**Success Criteria** (what must be TRUE):
  1. `agentix init --name my-agent` creates a valid agent directory (agent.yaml + SOUL.md) and `agentix validate agents/my-agent` passes
  2. `agentix gateway start` loads agent directories, exposes REST API, and streams ReAct loop output via SSE
  3. An existing AOF flat YAML agent (apiVersion: openagentix.dev/v1) loads and executes without modification
  4. `agentix agents`, `agentix runs`, `agentix logs`, `agentix stop`, `agentix apply`, `agentix validate`, `agentix init`, `agentix onboard` all function
  5. `agentix validate <path>` works on both agent directories and flat YAML files, reports errors with field paths
**Plans**: 9 plans in 6 waves

Plans:
- [x] 13-01-PLAN.md — Rename crates aof-* to agentix-*, delete v1.0 dead code
- [ ] 13-02-PLAN.md — GitAgent-compatible spec documentation (agent-directory-structure.md, agent-yaml-v1.md, workspace-config.md)
- [ ] 13-03-PLAN.md — Agent types and directory loader in agentix-core (TDD: AgentManifest, AgentDefinition, DirectoryLoader)
- [ ] 13-04-PLAN.md — ReAct loop engine in agentix-runtime (TDD: plan-act-observe-reflect, tool dispatch)
- [ ] 13-05-PLAN.md — Streaming output (TDD: SSE encoder, text formatter, JSON formatter)
- [x] 13-06-PLAN.md — Gateway HTTP service (axum, agent directory loading, run management)
- [ ] 13-07-PLAN.md — CLI commands: gateway, agents, runs, logs, stop, apply, validate, version
- [ ] 13-08-PLAN.md — CLI commands: init (agent directory scaffolder), onboard (workspace setup)
- [ ] 13-09-PLAN.md — LLM provider wiring, full workspace compilation, documentation update

### Phase 14: Skills Composition + Tools + WASM Sandbox
**Goal**: Users can compose agents from skill packs and custom tools, with untrusted WASM tools running in an isolated sandbox that enforces declared capability permissions.
**Depends on**: Phase 13
**Requirements**: SKILL-01, SKILL-02, SKILL-03, SKILL-04, SKILL-05, TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05, CLI-08
**Success Criteria** (what must be TRUE):
  1. User adds a `skills/postgres-tuning/SKILL.md` to their agent directory and its instructions are applied without an LLM routing call
  2. `agentix skills list` shows all built-in skill packs (aws, kubernetes, terraform, docker, git, database, security, observability)
  3. User defines a custom skill in their agent's `skills/` directory and the agent picks it up at runtime
  4. Agent calls CLI tools (kubectl, aws, psql, terraform, docker, git, shell) and MCP servers within a run
  5. A WASM tool without a declared capability (e.g., network access) is blocked from making that call at runtime
**Plans**: TBD

### Phase 15: Triggers + Scheduling
**Goal**: Agents can be triggered by cron, webhooks, GitHub events, Jira events, channel mentions, other agents, and the CLI — with all runs persisted and viewable.
**Depends on**: Phase 13
**Requirements**: TRIG-01, TRIG-02, TRIG-03, TRIG-04, TRIG-05, TRIG-06, TRIG-07, TRIG-08, TRIG-09, TRIG-10, TRIG-11
**Success Criteria** (what must be TRUE):
  1. User sets `triggers: [{type: cron, expression: "0 9 * * 1"}]` and the agent fires automatically on schedule
  2. A GitHub webhook fires the agent when a PR is opened, and the agent receives the PR payload via `TriggerEvent`
  3. A Jira webhook fires the agent when an issue is updated or commented on
  4. Mentioning `@agent-name` in Slack/Discord/Telegram triggers the agent in that channel
  5. `agentix runs` shows all past runs with timestamps, duration, trigger source, and status
**Plans**: TBD

### Phase 16: Agent Coordination + Memory
**Goal**: A coordinator agent can delegate to specialist agents and synthesize their results, while agents persist and recall context from previous runs via vector memory.
**Depends on**: Phase 15
**Requirements**: CORE-04, CORE-06, COORD-01, COORD-02, COORD-03, COORD-04, COORD-05, MEM-01, MEM-02, MEM-03, MEM-04
**Success Criteria** (what must be TRUE):
  1. A coordinator agent delegates a task to a specialist agent via `type: agent` trigger and receives its result
  2. Coordinator waits for multiple specialist agents to complete before synthesizing a final response
  3. An agent recalls relevant context from a previous run using semantic similarity search
  4. Agent memory is stored in SQLite by default and is scoped per agent (isolated unless explicitly shared)
  5. Agent-to-agent delegation appears in the audit trail with full parent/child context
**Plans**: TBD

### Phase 17: Cost Tracking + Budgets
**Goal**: Every LLM call, agent run, and agent is tracked for token usage and cost — with budget enforcement that stops agents before overspending — and smart model routing minimizes costs automatically.
**Depends on**: Phase 13
**Requirements**: COST-01, COST-02, COST-03, COST-04, COST-05, COST-06, COST-07, CLI-09, CORE-05
**Success Criteria** (what must be TRUE):
  1. `agentix costs` shows per-agent and aggregate cost summary with input/output tokens and USD amounts
  2. `agentix costs agent <name>` shows cost history for a specific agent broken down by run
  3. An agent with `budget: daily_limit: $0.50` stops mid-run when that limit is exceeded and reports the reason
  4. A simple message routes to the flash model tier; a complex research task routes to the pro tier — automatically
  5. Cost figures use actual API cost from provider response when available, not estimates
**Plans**: TBD

### Phase 18: Telemetry + Observability
**Goal**: Every agent run produces OpenTelemetry traces and metrics exportable to any OTel backend, with structured JSON logs and CLI trace access.
**Depends on**: Phase 13
**Requirements**: TELE-01, TELE-02, TELE-03, TELE-04, TELE-05
**Success Criteria** (what must be TRUE):
  1. An agent run produces OTel spans for each ReAct loop iteration, tool call, and LLM call — visible in Jaeger or Grafana Tempo
  2. OTel metrics (run duration, token usage, tool call count, success/failure rate) export to a configured collector
  3. `agentix logs <agent> --trace` shows structured trace output in the terminal with correlation IDs
  4. All agent run logs are structured JSON with consistent correlation IDs linking logs to traces
**Plans**: 5 plans in 4 waves

Plans:
- [x] 18-01-PLAN.md — Telemetry core types (TraceContext, SpanRecord, SpanKind, SpanStatus, StructuredLogEntry, LogLevel) in agentix-core (TDD)
- [x] 18-02-PLAN.md — TraceCollector runtime type + ReAct loop instrumentation (TDD: spans for run, iteration, LLM call, tool call, research, memory recall)
- [x] 18-03-PLAN.md — TraceStore SQLite persistence + AgentManager wiring + REST API trace endpoints
- [x] 18-04-PLAN.md — OTel exporter (OTLP JSON) + Prometheus /metrics endpoint + TelemetryConfig + OTel integration guide
- [x] 18-05-PLAN.md — CLI `agentix logs --trace` waterfall viewer + quickstart examples + reference docs + CHANGELOG v2.0.0-alpha.6

### Phase 19: Security
**Goal**: Untrusted tools are sandbox-enforced by WASM capabilities, secrets are encrypted at rest and never logged, every agent action is audited, and SSRF attacks are blocked at the network layer.
**Depends on**: Phase 14
**Requirements**: SEC-01, SEC-02, SEC-03, SEC-04, SEC-05
**Success Criteria** (what must be TRUE):
  1. A WASM tool attempting filesystem or network access without an explicit capability grant is blocked and the attempt is logged
  2. Secrets defined in agent config are stored AES-256-GCM encrypted and do not appear in logs, traces, or error output
  3. Audit trail records every tool call, LLM call, and approval decision with actor, timestamp, and outcome
  4. An HTTP tool call targeting a private IP (RFC 1918), localhost, or cloud metadata endpoint (169.254.x.x) is rejected with a SSRF error
**Plans**: 5 plans in 4 waves

Plans:
- [ ] 19-01-PLAN.md — Security core types (SsrfGuard, SsrfViolation, SecurityConfig, CapabilityPolicy) in agentix-core (TDD)
- [ ] 19-02-PLAN.md — Secret encryption (AES-256-GCM SecretStore + SecretRedactor) in agentix-core (TDD)
- [ ] 19-03-PLAN.md — AuditStore SQLite persistence + AgentManager wiring + REST API audit endpoints (TDD)
- [ ] 19-04-PLAN.md — Security runtime wiring — SSRF guard, audit logging, secret redaction, WASM capability in ReAct loop + AgentManager
- [ ] 19-05-PLAN.md — CLI `agentix audit` command + docs + quickstart + CHANGELOG v2.0.0-alpha.7

### Phase 20: Approval Workflows
**Goal**: Agents can pause at flagged actions, notify a human approver, and resume or abort based on the approval decision — with all decisions logged.
**Depends on**: Phase 16
**Requirements**: APPR-01, APPR-02, APPR-03, APPR-04, APPR-05
**Success Criteria** (what must be TRUE):
  1. An agent in `semi-autonomous` mode pauses before a flagged action and sends a notification to the configured approver
  2. Approver runs `agentix approve <request-id>` to resume the agent
  3. Approver runs `agentix deny <request-id>` to abort the action; agent logs the denial and stops that action path
  4. An agent in `manual` mode requests approval for every action; in `autonomous` mode it requires no approvals
  5. All approval decisions appear in the audit trail with approver identity, timestamp, and decision
**Plans**: 5 plans in 4 waves

Plans:
- [x] 20-01-PLAN.md — Approval core types (ApprovalRequest, ApprovalStatus, ApprovalDecision, ApprovalPolicy) in agentix-core (TDD)
- [x] 20-02-PLAN.md — ApprovalStore SQLite persistence in agentix-runtime (TDD)
- [x] 20-03-PLAN.md — ReAct loop approval gate integration — pause/resume when approval required (TDD)
- [x] 20-04-PLAN.md — AgentManager wiring + REST API approval endpoints (GET/POST /api/v1/approvals)
- [x] 20-05-PLAN.md — CLI `agentix approve/deny/approvals` commands + quickstart + docs + CHANGELOG v2.0.0-alpha.8

### Phase 21: Gateway (Multi-Channel)
**Goal**: Agents can receive messages from and send responses to Slack, Telegram, and Discord — with per-channel routing configuration and bi-directional trigger + delivery.
**Depends on**: Phase 15
**Requirements**: GW-01, GW-02, GW-03, GW-04, GW-05, GW-06
**Success Criteria** (what must be TRUE):
  1. A message sent to a Slack channel reaches the configured agent, which processes it and replies in the same channel
  2. A message sent on Telegram triggers the agent and the response arrives in the same Telegram chat
  3. A message sent on Discord triggers the agent and the response arrives in the same Discord channel
  4. Agent sends a notification (run result, alert, approval request) to configured channels without a user-initiated trigger
  5. Per-channel config (`channels:`) routes different agents to different Slack channels or Telegram chats
**Plans**: 6 plans in 4 waves

Plans:
- [x] 21-01-PLAN.md — Channel gateway core types (ChannelConfig, ChannelRoute, ChannelGateway trait, NotificationTarget, NotificationPayload) in agentix-core (TDD)
- [x] 21-02-PLAN.md — Slack channel adapter (SlackChannelGateway implementing ChannelGateway trait) in agentix-runtime (TDD)
- [x] 21-03-PLAN.md — Telegram channel adapter (TelegramChannelGateway implementing ChannelGateway trait) in agentix-runtime (TDD)
- [x] 21-04-PLAN.md — Discord channel adapter (DiscordChannelGateway implementing ChannelGateway trait) in agentix-runtime (TDD)
- [x] 21-05-PLAN.md — ChannelGatewayManager + AgentManager wiring + REST API channel endpoints + workspace config
- [x] 21-06-PLAN.md — CLI `agentix channels/notify` commands + quickstart + docs + CHANGELOG v2.0.0-alpha.9

### Phase 22: Command Center (Svelte)
**Goal**: Users have a web dashboard (Svelte) to monitor all agents, view run history and costs, inspect execution traces, manage approvals, and build/edit agent definitions — with real-time WebSocket updates and interactive visualizations for traces, costs, and agent coordination.
**Depends on**: Phase 17, Phase 18, Phase 20
**Requirements**: CMD-01, CMD-02, CMD-03, CMD-04, CMD-05, CMD-06, CMD-07, CMD-08, CMD-09, VIZ-01, VIZ-02, VIZ-03
**Success Criteria** (what must be TRUE):
  1. Dashboard shows all registered agents with live status (running/idle/error/scheduled) updating in real time via WebSocket
  2. Agent detail view shows configuration, run history, per-run cost, and logs in one place
  3. Cost dashboard renders per-agent and aggregate cost charts filterable by date range
  4. Trace viewer shows a waterfall of spans (tool calls, LLM calls, loop iterations) for any selected run
  5. Approval queue shows pending approval requests; user clicks Approve or Deny and the agent resumes/aborts
  6. Agent run traces render as interactive waterfall diagrams with expandable span details
  7. Cost data renders as interactive charts (bar, line, pie) with date range filtering and drill-down
  8. Multi-agent coordination renders as interactive directed graphs showing delegation relationships
**Plans**: 8 plans in 4 waves

Plans:
- [ ] 22-01-PLAN.md — Fork mission-control scaffold, SvelteKit 5 app with sidebar, API types, REST client
- [ ] 22-02-PLAN.md — Gateway WebSocket endpoint for real-time event broadcasting (Rust)
- [ ] 22-03-PLAN.md — Dashboard landing, agent list with live status, agent detail, run history
- [ ] 22-04-PLAN.md — Cost dashboard with interactive bar, line, and doughnut charts
- [ ] 22-05-PLAN.md — Trace viewer with Jaeger-style waterfall and multi-agent coordination graph
- [ ] 22-06-PLAN.md — Approval queue with approve/deny actions and real-time updates
- [ ] 22-07-PLAN.md — Agent builder with form, SOUL.md editor, skill browser, test run
- [ ] 22-08-PLAN.md — Scheduler view, first-run wizard, settings, docs, CHANGELOG v2.0.0-alpha.10

## Progress

**Execution Order:** Phases execute in numeric order: 13 -> 14 -> 15 -> 16 -> 17 -> 18 -> 19 -> 20 -> 21 -> 22 -> 23

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 13. Rebrand + Core Runtime + CLI Foundation | 9/9 | Complete   | 2026-03-12 | - |
| 14. Skills Composition + Tools + WASM Sandbox | v2.0 | 0/TBD | Not started | - |
| 15. Triggers + Scheduling | v2.0 | Complete    | 2026-03-13 | - |
| 16. Agent Coordination + Memory | v2.0 | Complete    | 2026-03-13 | - |
| 17. Cost Tracking + Budgets | v2.0 | Complete    | 2026-03-13 | - |
| 18. Telemetry + Observability | 5/5 | Complete    | 2026-03-13 | - |
| 19. Security | 5/5 | Complete    | 2026-03-13 | - |
| 20. Approval Workflows | 1/5 | 4/5 | In Progress|  |
| 21. Gateway (Multi-Channel) | 6/6 | Complete    | 2026-03-13 | - |
| 22. Command Center (Svelte) | 8/8 | Complete   | 2026-03-13 | - |
| 23. LLM Subscription Proxy | 7/7 | Complete    | 2026-03-23 | - |

### Phase 23: LLM Subscription Proxy

**Goal:** Agents can use existing LLM subscriptions (Claude, ChatGPT, Gemini) via OAuth-based authentication instead of requiring separate API keys — reducing cost for users who already pay for subscriptions.
**Requirements**: SUB-01 (OAuth auth flows), SUB-02 (token persistence), SUB-03 (adaptive rate limiting), SUB-04 (provider factory routing), SUB-05 (Command Center UI)
**Depends on:** Phase 22
**Success Criteria** (what must be TRUE):
  1. User runs `agentix auth start openai` and authenticates via browser-based OAuth; token stored encrypted
  2. An agent configured with `mode: subscription` transparently uses the OAuth token instead of an API key
  3. Token refresh happens automatically with retry/backoff; expired tokens fail the run (no silent fallback)
  4. Settings page shows segmented control per provider (API Key | Subscription) with OAuth flow
  5. `agentix auth status` shows connection status for all three providers
**Plans**: 7 plans in 5 waves

Plans:
- [x] 23-01-PLAN.md — Auth core types, ProviderMode, encrypted profile store, OAuth common (agentix-core)
- [x] 23-02-PLAN.md — Per-provider OAuth flows (OpenAI, Gemini, Anthropic) + AuthService coordinator
- [x] 23-03-PLAN.md — Subscription provider adapters + ProviderFactory routing (agentix-llm)
- [x] 23-04-PLAN.md — Gateway OAuth API endpoints + AgentManager integration (agentix-runtime)
- [x] 23-05-PLAN.md — CLI `agentix auth start|status|disconnect` command
- [x] 23-06-PLAN.md — Command Center settings page subscription UI (segmented control + OAuth popup)
- [ ] 23-07-PLAN.md — Documentation, quickstart config, CHANGELOG v2.0.0-alpha.11
