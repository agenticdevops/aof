# AOF - The Humanized Agentic Ops Platform

## What This Is

An open-source (Apache 2.0) platform that makes AI agents feel like team members, not scripts. Built on a Rust core, AOF gives DevOps/SRE engineers agent squads with real personalities, visible coordination, and a Mission Control dashboard — all while doing real ops work (K8s, monitoring, incident response). Think "OpenClaw for DevOps" but built for production infrastructure.

## Core Value

Agents that feel human — with personas, visible communication, and a Mission Control where you see your team of AI minions coordinating, reporting, and getting real work done.

## Requirements

### Validated

<!-- Shipped and confirmed valuable (existing AOF capabilities). -->

- Multi-provider LLM abstraction (Anthropic, OpenAI, Google, Groq, Ollama, Bedrock) — existing
- Agent execution engine with tool composition and streaming — existing
- Workflow execution (DAG-based step orchestration) — existing
- AgentFlow execution (multi-agent graph flows) — existing
- Memory backends (in-memory, file-based, optional Redis/Sled) — existing
- MCP client support (stdio, SSE, HTTP transports) — existing
- Built-in tool registry (kubectl, docker, git, shell, HTTP, file ops) — existing
- Trigger server with platform adapters (Telegram, Slack, Discord stubs) — existing
- Skills system (SKILL.md loading, registry, requirements gating) — existing
- Fleet coordination primitives (Raft, Byzantine consensus) — existing
- kubectl-style CLI (aofctl) — existing
- TUI interactive mode with streaming — existing
- Error knowledge base for learning from failures — existing
- Session management with resume capability — existing
- YAML-first agent/workflow/flow configuration — existing

### Active

<!-- The reinvention: humanized agentic ops platform. -->

**Agent Persona System (SOUL.md)**
- [ ] Each agent has a persistent personality defined in SOUL.md (identity, communication style, boundaries, vibe)
- [ ] Agents speak in character — their personality comes through in every interaction
- [ ] Avatar/icon system — each agent has a visual identity (emoji, pixel art, or custom image)
- [ ] Role titles and skill tags displayed on agent profile cards
- [ ] Agents maintain consistent personality across sessions via memory

**Visible Agent Communication**
- [ ] Squad chat — agents talk to each other in a shared chat stream visible to humans
- [ ] Announce queue — cross-agent communication protocol (agent A can message agent B)
- [ ] Humans can join squad chat, interrupt, redirect, or give new instructions
- [ ] Agent-to-agent task delegation — one agent can create tasks for another
- [ ] Communication logs are persistent and reviewable

**Mission Control (WASM Web UI)**
- [ ] WASM-based web dashboard compiled from Rust (pure Rust story, no JS framework)
- [ ] Agent cards — profile view with avatar, role, status, personality, skills, attention items
- [ ] Kanban task board — tasks flow through backlog/assigned/in-progress/review/done
- [ ] Squad chat panel — real-time view of agent-to-agent and human-to-agent conversation
- [ ] Live activity feed — real-time stream of what agents are doing (like GitHub activity)
- [ ] Task detail view — description, context, assignee (agent), comments, timeline, attachments
- [ ] Agent status indicators (idle, working, waiting for human, blocked)
- [ ] Squad overview — visual representation of all agents and their relationships

**Standups, Check-ins & Coordination**
- [ ] Agents perform scheduled standups — report what they did, what they're doing, blockers
- [ ] Check-in protocol — agents periodically report status without being asked
- [ ] Heartbeat system — proactive monitoring checks on schedules (every 30min, daily, etc.)
- [ ] Roundtable discussions — agents can hold group conversations to solve problems together
- [ ] Human-in-the-loop workflows — agents assign tasks to humans with context and comments

**Messaging Gateway (Slack/Discord)**
- [ ] Single bot mode — one bot in Slack, routes to different agents behind the scenes
- [ ] Dedicated agent channels — each agent appears separately in squad channels
- [ ] NAT-transparent — outbound WebSocket (no ngrok needed for Slack/Discord)
- [ ] Agents respond in character with their persona
- [ ] Squad announcements — broadcast to all agents or specific teams

**Conversational Configuration (The Interface IS Conversation)**
- [ ] Talk to the system to create agents — "I need a K8s monitoring agent" → agent with persona created
- [ ] Talk to build agent teams/fleets — "Build me an incident response squad" → team created with roles
- [ ] Talk to configure schedules — "Check my cluster every 30 minutes" → heartbeat configured
- [ ] Talk to add skills — "Learn how to debug our Postgres" → skill created from conversation
- [ ] YAML/CLI as power-user layer underneath — conversation generates config, not the other way around
- [ ] The main agent (orchestrator/router) understands intent and delegates to the right agents

**Real Ops Capabilities**
- [ ] K8s diagnostics — pod debugging, log analysis, event inspection, resource usage
- [ ] Incident response flow — triage agent coordinates specialist agents
- [ ] Monitoring integration — Prometheus queries, alert triage
- [ ] Skills platform — codify tribal knowledge as executable SKILL.md files
- [ ] Runbook execution — convert wiki/playbook procedures into agent skills

**Local-First Architecture**
- [ ] Local Rust daemon — agents run on your machine, Mission Control connects to it
- [ ] Optional server deployment — deploy daemon to server for always-on agents
- [ ] WebSocket control plane — Mission Control and Slack connect to daemon
- [ ] Session persistence — agent state survives daemon restarts

### Out of Scope

- Multi-tenancy / MSP features — enterprise product, not v1 open source
- RBAC / SSO / audit trails — enterprise product
- Billing / usage tracking — enterprise product
- Cloud-hosted SaaS offering — self-hosted only for v1
- Mobile app — web + Slack/Discord are the interfaces
- Voice/talk mode — text-based interactions for v1
- OAuth subscription support (Anthropic Pro/Max) — nice to have, not v1

## Context

**Why this exists:** OpenClaw proved that making AI agents feel human goes viral. Every agentic framework (LangGraph, CrewAI, Agno) feels like running scripts — even if technically powerful. The missing ingredient is the *human touch*: agents with personalities, visible coordination, and interfaces that make you feel like you're managing a team of intelligent minions. No one has built this for DevOps/SRE.

**What we're building on:** AOF has a solid Rust foundation — 13 crates covering LLM abstraction, agent execution, workflows, memory, tools, triggers, skills, and fleet coordination. The engine is proven. What's missing is the soul.

**Inspiration sources:**
- OpenClaw/Clawdbot: SOUL.md personas, agent-to-agent comms, skills platform, heartbeat system
- OpenClaw Mission Control: kanban tasks, agent cards, squad chat, live activity, task assignment
- Research in `/Users/gshah/work/opsflow-sh/plans/research/`: strategic analysis, feature extraction, architecture plans

**Existing codebase:** 13 Rust crates at v0.4.0-beta. Codebase map at `.planning/codebase/`. The Rust engine stays and evolves; the CLI/UX layer gets reinvented.

**Brand:** AOF (Agentic Ops Framework) remains the engine name. Product brand TBD — xops.bot is available as an option. Name decision deferred to post-prototype.

### Security: AOF's Enterprise Differentiation (vs OpenClaw)

**Phase 8 Delivery — Production Security Hardening:**

AOF is NOT just a humaner OpenClaw clone. It's **enterprise-grade agentic infrastructure** with security designed from the ground up:

**Defense-in-Depth Security Model (6 layers):**
1. **Sandbox Isolation:** Per-tool seccomp profiles blocking 23+ dangerous syscalls (ptrace, mount, bpf, etc.) — prevents kernel exploits
2. **Capability Dropping:** `--cap-drop=ALL` by default with per-tool allowlists — strips unnecessary permissions
3. **Credential Auditing:** CredentialAccessInterceptor logs every credential read with tamper-proof sequence numbers — track who accessed what
4. **Behavioral Anomaly Detection:** 4-component scoring system detects suspicious credential access patterns — catch insider threats
5. **Device Pairing & mTLS:** Private CA + device registry with approval workflow — only trusted devices can pair
6. **Production Observability:** SRE-grade metrics, health checks, graceful shutdown, incident runbooks — production-hardened

**Why this matters for enterprises:**
- **OpenClaw** executes user code with minimal isolation — fine for trusted OpenAI API calls, dangerous for production infrastructure access
- **AOF** runs untrusted agent code in hardened containers with comprehensive audit trails — enterprise can prove compliance
- **Selling point:** "Agents that feel human, but production-hardened for infrastructure access"

**Blog Series Planned (Q1 2026):**
1. "AOF vs OpenClaw: Why Human-Feeling Agents Need Enterprise Security"
2. "Seccomp Deep Dive: How AOF Prevents Sandbox Escape Attacks"
3. "Credential Auditing in Agentic Systems: The Missing Security Layer"
4. "From OpenClaw to OpenAgentiX: Generalizing AOF for Enterprise"

### Future Vision: OpenAgentiX Platform

**Phase 9-10 Generalization Path:**

AOF currently targets **DevOps/SRE** as initial market. Future vision is **OpenAgentiX** — a generalized agentic platform for any enterprise use case:

**Generalization Roadmap:**
- **v0.5 (AOF):** DevOps/SRE agents with K8s tools, incident response, monitoring
- **v1.0 (AOF + DevOps Enterprise):** Persona system, Mission Control, Slack/Discord, production hardening
- **v2.0 (OpenAgentiX):** Multi-domain agent framework — swap K8s tools for database, network, security, finance, HR tools
- **v2.5 (OpenAgentiX Enterprise):** Multi-tenancy, RBAC, SSO, audit trails, billing (separate commercial product)

**Key Insight:**
The security model (seccomp + credential auditing + behavioral anomaly detection) **is domain-agnostic**. It works for K8s agents, database agents, finance agents, any untrusted code executing against production systems.

**Market Positioning:**
- **OpenClaw** = Make agents feel human (great UX, no security)
- **AOF** = Make agents feel human + production-hardened (DevOps focused)
- **OpenAgentiX** = Make agents feel human + enterprise-secure (any domain, multi-tenancy, compliance)

## Constraints

- **Language**: Rust for core engine and WASM Mission Control (pure Rust story is a differentiator)
- **License**: Apache 2.0 — everything open source, enterprise features come later in separate products
- **Architecture**: Local-first — must work on a single machine, server deployment optional
- **Performance**: Rust performance is a selling point — agent communication and task coordination must be snappy
- **Frontend**: Mission Control built with builder.io (user's existing tool). Backend/daemon is Rust. Beautiful UX wins over language purity.
- **Backward compatibility**: Existing AOF YAML configs should still work (migration path, not hard break)
- **Cross-platform**: macOS, Linux, Windows (same as current AOF)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| builder.io for Mission Control | User's existing tool. Beautiful, polished UX. Rust backend + builder.io frontend. | — Pending |
| Local-first architecture | DevOps engineers want control, not another SaaS. Server mode is opt-in. | — Pending |
| Everything open source (v1) | Virality requires zero friction. Enterprise features are a separate product. | — Pending |
| Keep AOF as engine name | Established brand, crates already published. Product name TBD. | — Pending |
| Agents as "team members" not "tools" | This is THE differentiator. Every design decision serves the human feel. | — Pending |
| Slack/Discord dual mode | Single bot for quick access + dedicated agent channels for squad work | — Pending |
| Reinvention over evolution | Willing to restructure core if needed — the vision is more important than preserving current CLI patterns | — Pending |
| Conversation as primary interface | Users talk to the system, not write YAML. Config is generated from conversation. YAML is the power-user escape hatch. | — Pending |
| Simplicity over power | Dead simple first experience beats feature richness. If you need docs to start, you've lost. | — Pending |

---
*Last updated: 2026-02-11 after initialization*
