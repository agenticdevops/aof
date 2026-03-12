# OpenAgentiX — Enterprise Agent Automation Platform

## What This Is

An open-source (Apache 2.0) enterprise agent automation platform built in Rust. Organizations define expert agents in YAML — composing skills, tools, and MCP servers — then run them on cron, on events, or on demand. Single binary deployment, built-in cost tracking, multi-channel delivery (Slack, Teams, Telegram, Discord, WhatsApp), and a Svelte command center for fleet management.

## Core Value

Let any technical organization automate operational tasks with AI agents — without writing Python, without managing infrastructure, without giving up control. Define in YAML, run as a binary, track costs, keep humans in the loop.

## Current Milestone: v2.0 OpenAgentiX

**Goal:** Transform AOF into an enterprise agent automation platform with YAML-first agent composition, skills + tools + MCP, scheduled/event-driven execution, cost tracking, and a Svelte command center.

**Target features:**
- Rebrand to OpenAgentiX with new agent YAML spec
- Skills composition system (composable capability packs)
- Cron/event-driven agent triggers (not just chat)
- Per-agent cost tracking + budgets (inspired by ClawWork)
- OTel telemetry + agent traces
- WASM sandbox for untrusted tools (inspired by IronClaw)
- Approval workflows / human-in-the-loop
- Smart model routing (complexity scoring)
- Multi-channel gateway (Slack, Teams, Telegram, Discord, WhatsApp)
- Svelte command center (agent builder, run history, costs, scheduler)

## Requirements

### Validated

- Multi-provider LLM abstraction (Anthropic, OpenAI, Google, Groq, Ollama, Bedrock) — v1.0
- Agent execution engine with tool composition and streaming — v1.0
- Workflow execution (DAG-based step orchestration) — v1.0
- Memory backends (in-memory, file-based, optional Redis/Sled) — v1.0
- MCP client support (stdio, SSE, HTTP transports) — v1.0
- Built-in tool registry (kubectl, docker, git, shell, HTTP, file ops) — v1.0
- Trigger server with platform adapters (Telegram, Slack, Discord) — v1.0
- Skills system (SKILL.md loading, registry, requirements gating) — v1.0
- kubectl-style CLI (aofctl) — v1.0
- YAML-first agent/workflow/flow configuration — v1.0
- Session management with resume capability — v1.0
- WebSocket control plane — v1.0
- Production security (seccomp, capability dropping, credential audit) — v1.0

### Active

(Defined in REQUIREMENTS.md for v2.0)

### Out of Scope

- Agent personas / SOUL.md / personality system — pivoted away from "fun team" model
- React Mission Control — replaced by Svelte command center
- Squad Chat UI — not enterprise-relevant
- Kanban / workflow builder UI — defer to v2.1+
- Multi-tenancy / MSP isolation — v2.5 enterprise
- SSO / SAML / LDAP — v2.5 enterprise
- Mobile native app — web + messaging channels sufficient
- Voice/talk mode — text-based for v2.0
- Public agent marketplace — security concerns, community registry first
- A2A protocol — emerging standard, defer until stable

## Context

**Previous milestone:** v1.0 "Humanized Interfaces" shipped 2026-02-22 (12 phases, 55 plans, 164K LOC). Proved the Rust + multi-channel architecture works. The persona/personality direction is being pivoted away from.

**Market landscape (researched 2026-03-12):**
- Python-dominant (LangChain, LangGraph, CrewAI, Agno, AutoGen) — no Rust-native competitor
- 45% of developers who try LangChain never use it in production (abstraction bloat)
- 75% of enterprise leaders cite security/compliance as top AI priority
- Enterprises need: cost tracking, audit trails, RBAC, event-driven triggers, deployment flexibility
- MCP becoming standard (97M monthly SDK downloads). A2A emerging but nascent.
- No framework does config-first + enterprise security + cost tracking + event-driven well

**Competitive inspiration:**
- ClawWork: per-call/task/day cost tracking, quality thresholds, ROI metrics
- OpenFang: "Hands" (pre-built domain-complete agents), 16-layer security, workflow orchestration
- IronClaw: WASM sandbox, shared agentic loop, smart model routing, routines engine, cost guard
- ZeroClaw: trait-driven architecture, research phase, AgentBuilder pattern

**Target users:**
- Platform engineering teams (Netflix, Dropbox, Adobe scale)
- SRE/DevOps teams (regulated industries — Visa, Morgan Stanley)
- Startup CTOs (5-person team, need automation fast)
- Enterprise architects (standardize agents across 15+ teams)
- SI/MSP/GCC (manage agents for multiple clients)

**Example agents enterprises want to build:**
- DBA optimizer (analyze RDS config, recommend/apply optimizations)
- Cost anomaly detector (daily AWS cost analysis, alert on spikes)
- Compliance scanner (CIS benchmark checks, file Jira tickets)
- Security scanner (vulnerability detection, remediation tickets)
- Self-healing infrastructure (detect + remediate common failures)
- Deployment agent (canary rollouts with approval gates)

## Constraints

- **Language**: Rust for core engine. Svelte for command center.
- **License**: Apache 2.0 — open source core, enterprise features later
- **Architecture**: Single binary, local-first. Server mode for teams.
- **CLI style**: kubectl-style (`oax run agent`, `oax apply -f agent.yaml`)
- **No Python dependency**: Users should never need pip/venv/conda
- **Backward compatibility**: Existing AOF agent YAML configs should still work
- **Cross-platform**: macOS, Linux, Windows

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust single binary | No Python, no runtime deps, 180ms cold start, 40MB idle | — Pending |
| YAML-first agent definition | Enterprises want declarative, version-controllable config | — Pending |
| Svelte for command center | Lighter than React, better DX, user preference | — Pending |
| Skills composition (not monolithic agents) | Build expert agents from capability packs, not one agent with 100 tools | — Pending |
| WASM sandbox for tools | Lightweight isolation without Docker overhead, capability-based | — Pending |
| OTel native telemetry | Standard, exportable to Grafana/Datadog/any backend | — Pending |
| Smart model routing | Route simple→cheap models, complex→expensive. Cost optimization. | — Pending |
| Cost tracking built-in | Not an add-on. Per-call, per-task, per-agent. Enterprise requirement. | — Pending |
| Pivot from personas to enterprise | Personality-driven agents don't solve enterprise problems | ✓ Good |

---
*Last updated: 2026-03-12 after v2.0 milestone start*
