# OpenAgentiX — Enterprise Agent Automation Platform

## What This Is

An open-source (Apache 2.0) enterprise agent automation platform built in Rust. Organizations define expert agents in YAML — composing skills, tools, and MCP servers — then run them on cron, on events, or on demand. Single binary deployment, built-in cost tracking, multi-channel delivery (Slack, Telegram, Discord), and a Svelte command center for fleet management. Agents authenticate via API keys or existing LLM subscriptions (Claude, ChatGPT, Gemini) through OAuth.

## Core Value

Let any technical organization automate operational tasks with AI agents — without writing Python, without managing infrastructure, without giving up control. Define in YAML, run as a binary, track costs, keep humans in the loop.

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
- GitAgent-compatible agent directories with ReAct loop engine — v2.0
- Skills composition with 8 built-in packs + WASM sandbox isolation — v2.0
- Event-driven triggers (cron, webhook, GitHub, Jira, agent-to-agent) — v2.0
- Multi-agent coordination with vector memory and semantic recall — v2.0
- Per-call/run/agent cost tracking with budget enforcement — v2.0
- OpenTelemetry traces/metrics with CLI trace viewer — v2.0
- Security: SSRF protection, AES-256-GCM secrets, comprehensive audit trail — v2.0
- Human-in-the-loop approval workflows (3 autonomy modes) — v2.0
- Multi-channel gateway: Slack, Telegram, Discord bi-directional — v2.0
- Svelte Command Center: dashboard, agent builder, cost charts, trace viewer, approval queue — v2.0
- LLM subscription proxy: OAuth auth for Claude/ChatGPT/Gemini subscriptions — v2.0
- kubectl-style `agentix` CLI with 15+ commands — v2.0

### Active

(Defined in REQUIREMENTS.md for next milestone)

### Out of Scope

- Agent personas / SOUL.md / personality system — pivoted away from "fun team" model
- React Mission Control — replaced by Svelte command center
- Squad Chat UI — not enterprise-relevant
- Python SDK — Rust-native, no Python dependency is a differentiator
- Public agent marketplace — security concerns, community registry first
- A2A protocol — emerging standard, defer until stable
- Voice/talk mode — text-based for enterprise automation
- Mobile native app — web + messaging channels sufficient

## Context

**v1.0** shipped 2026-02-22: 12 phases, 55 plans, 164K LOC. Proved Rust + multi-channel architecture.
**v2.0** shipped 2026-03-23: 11 phases, 67 plans, 78K LOC (rewrite). Enterprise pivot — GitAgent agents, skills, triggers, cost tracking, telemetry, security, approvals, multi-channel gateway, Svelte dashboard, OAuth subscription proxy.

**Total:** 23 phases, 122 plans across 2 milestones.

**Tech stack:** Rust (agentix-core, agentix-llm, agentix-mcp, agentix-runtime, agentix-memory, agentix-triggers, agentix CLI), SvelteKit 5 (Command Center), SQLite (persistence).

**Market landscape (researched 2026-03-12):**
- Python-dominant (LangChain, LangGraph, CrewAI, Agno, AutoGen) — no Rust-native competitor
- 45% of developers who try LangChain never use it in production
- 75% of enterprise leaders cite security/compliance as top AI priority
- No framework does config-first + enterprise security + cost tracking + event-driven well

**Target users:** Platform engineering teams, SRE/DevOps (regulated industries), startup CTOs, enterprise architects, SI/MSP/GCC.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust single binary | No Python, no runtime deps, 180ms cold start, 40MB idle | ✓ Good |
| YAML-first agent definition | Enterprises want declarative, version-controllable config | ✓ Good |
| Svelte for command center | Lighter than React, better DX, user preference | ✓ Good |
| Skills composition (not monolithic agents) | Build expert agents from capability packs | ✓ Good |
| WASM sandbox for tools | Lightweight isolation without Docker overhead | ✓ Good |
| OTel native telemetry | Standard, exportable to Grafana/Datadog/any backend | ✓ Good |
| Smart model routing | Route simple->cheap, complex->expensive | ✓ Good |
| Cost tracking built-in | Not an add-on. Per-call, per-task, per-agent | ✓ Good |
| Pivot from personas to enterprise | Personality-driven agents don't solve enterprise problems | ✓ Good |
| AES-256-GCM for secrets | Already in workspace, proven crypto | ✓ Good |
| SQLite for all stores | Simple, embedded, zero-config persistence | ✓ Good |
| OAuth subscription proxy | Reduce cost for users with existing LLM subscriptions | ✓ Good |

## Constraints

- **Language**: Rust for core engine. Svelte for command center.
- **License**: Apache 2.0 — open source core, enterprise features later
- **Architecture**: Single binary, local-first. Server mode for teams.
- **CLI style**: kubectl-style (`agentix run agent`, `agentix apply -f agent.yaml`)
- **No Python dependency**: Users should never need pip/venv/conda
- **Backward compatibility**: Existing AOF agent YAML configs still work
- **Cross-platform**: macOS, Linux, Windows

---
*Last updated: 2026-03-23 after v2.0 milestone*
