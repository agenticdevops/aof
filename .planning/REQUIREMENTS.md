# Requirements: AOF - Humanized Agentic Ops Platform

**Defined:** 2026-02-11
**Core Value:** Agents that feel human — with personas, visible communication, and a Mission Control where you see your team of AI minions coordinating, reporting, and getting real work done.

## v1 Requirements

Requirements for v1 release. Each maps to roadmap phases.

### Agent Personas

- [ ] **PERS-01**: Each agent has a SOUL.md that defines personality, communication style, boundaries, and vibe
- [ ] **PERS-02**: Agents speak in character — personality comes through in every response and interaction
- [ ] **PERS-03**: Each agent has a visual identity — avatar/emoji, role title, and skill tags
- [ ] **PERS-04**: Agent persona persists across sessions and daemon restarts via memory
- [ ] **PERS-05**: Agents introduce themselves when joining a squad — "meet the team" experience

### Visible Communication

- [ ] **COMM-01**: Agents talk to each other in a shared squad chat stream visible to humans
- [ ] **COMM-02**: Cross-agent announce queue — agent A can message agent B with context
- [ ] **COMM-03**: Humans can join squad chat, interrupt agents, redirect work, or give new instructions
- [ ] **COMM-04**: One agent can create and assign tasks to another agent
- [ ] **COMM-05**: All agent communication is logged, persistent, and reviewable

### Mission Control (WASM Web UI)

- [ ] **MCUI-01**: Web dashboard with clean, beautiful UI — modern JS frontend (React/Svelte/SolidJS) backed by Rust WebSocket API
- [ ] **MCUI-02**: Agent cards with avatar, role, status (idle/working/waiting/blocked), personality summary, skills
- [ ] **MCUI-03**: Kanban task board — tasks flow through backlog → assigned → in-progress → review → done
- [ ] **MCUI-04**: Squad chat panel — real-time view of agent-to-agent and human-to-agent conversation
- [ ] **MCUI-05**: Live activity feed — real-time stream of agent actions (like GitHub activity feed)
- [ ] **MCUI-06**: Task detail view — description, context, assignee agent, comments, timeline
- [ ] **MCUI-07**: Squad overview — visual representation of all agents and their current state

### Conversational Interface

- [ ] **CONV-01**: User can talk to the system to create agents — "I need a K8s monitoring agent" creates one
- [ ] **CONV-02**: User can talk to build agent teams — "Build me an incident response squad" assembles a fleet
- [ ] **CONV-03**: User can talk to configure schedules — "Check my cluster every 30 min" sets up heartbeat
- [ ] **CONV-04**: User can talk to teach skills — "Learn how to debug our Postgres" creates a skill
- [ ] **CONV-05**: A main orchestrator agent routes user intent to the right specialist agents
- [ ] **CONV-06**: YAML/CLI exists as power-user layer — conversation generates config underneath

### Coordination Protocols

- [ ] **CORD-01**: Agents perform scheduled standups — report what they did, doing next, and blockers
- [ ] **CORD-02**: Agents proactively check in — periodic status reports without being asked
- [ ] **CORD-03**: Heartbeat system — proactive monitoring on configurable schedules
- [ ] **CORD-04**: Roundtable discussions — agents hold group conversations to solve problems together
- [ ] **CORD-05**: Human-in-the-loop — agents assign tasks to humans with context and comments

### Messaging Gateway

- [ ] **MSGG-01**: Single bot mode in Slack — one bot routes to different agents behind the scenes
- [ ] **MSGG-02**: Dedicated agent channels — agents can appear separately in squad channels
- [ ] **MSGG-03**: NAT-transparent — outbound WebSocket for Slack/Discord (no ngrok needed)
- [ ] **MSGG-04**: Agents respond in character with their persona in messaging platforms
- [ ] **MSGG-05**: Squad announcements — broadcast messages to all agents or specific teams

### Real Ops Capabilities

- [ ] **ROPS-01**: K8s diagnostics — pod debugging, log analysis, event inspection via agent tools
- [ ] **ROPS-02**: Incident response flow — triage agent coordinates specialist agents for investigation
- [ ] **ROPS-03**: Skills platform — codify tribal knowledge as executable SKILL.md files agents can use
- [ ] **ROPS-04**: Decision logging — agents log what they did AND why (reasoning, confidence, alternatives)
- [ ] **ROPS-05**: 10-20 bundled ops skills (kubectl, git, shell, HTTP, Prometheus queries, log search)

### OpenClaw-Inspired Engine Features

- [ ] **ENGN-01**: Queue management — lane-based serialization prevents agent collisions on shared resources
- [ ] **ENGN-02**: Cron + timezone scheduling — precise schedules ("daily 6am EST", "every 30min during business hours")
- [ ] **ENGN-03**: Browser automation — persistent session cookies, manual login once then agent reuses session
- [ ] **ENGN-04**: Subagent spawning — parent agent can spawn child agents for subtasks with announce queue

### SRE Capabilities

- [ ] **SREW-01**: Incident war rooms — dedicated channel auto-created when incident triggers, agents auto-assemble
- [ ] **SREW-02**: Automated triage — classify alert severity, route to correct specialist agents
- [ ] **SREW-03**: Root cause analysis — agents correlate logs, metrics, traces to identify probable cause
- [ ] **SREW-04**: Blameless postmortems — auto-generate incident timeline, contributing factors, action items after resolution

### Infrastructure

- [ ] **INFR-01**: Local Rust daemon — agents run on your machine, Mission Control and Slack connect to it
- [ ] **INFR-02**: WebSocket control plane — real-time event streaming from daemon to all clients
- [ ] **INFR-03**: Event-driven architecture — tokio broadcast channel as central event bus
- [ ] **INFR-04**: Session persistence — agent state, task queue, and memory survive daemon restarts
- [ ] **INFR-05**: Optional server deployment — same daemon can run on a server for always-on agents

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced Coordination

- **ADVR-01**: Incident response squad auto-formation — spawn specialist team from alert type
- **ADVR-02**: Cross-session deep context — agents remember decisions across weeks/months
- **ADVR-03**: Agent onboarding wizard — guided setup with personality, skills, permissions
- **ADVR-04**: Progressive trust model — agents earn autonomy based on track record

### Self-Learning & Knowledge

- **LRNG-01**: Knowledge base — agents build org-specific knowledge from incidents, postmortems, resolutions
- **LRNG-02**: Continuous learning — agents improve from past mistakes, track what worked vs didn't
- **LRNG-03**: Self-learning systems — ReasoningBank-style retrieve → judge → distill → consolidate pipeline

### Enterprise Features

- **ENTR-01**: Audit trail / compliance — immutable logs, SOC2/ISO export
- **ENTR-02**: Multi-cloud K8s intelligence — cluster topology, cost optimization, security posture
- **ENTR-03**: Real-time observability integration — Prometheus/OTel metrics for agents
- **ENTR-04**: Skills marketplace — publish, discover, install skills across teams

### Additional Messaging

- **AMSG-01**: Microsoft Teams integration
- **AMSG-02**: PagerDuty bidirectional integration
- **AMSG-03**: GitHub/Jira bot integration

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Multi-tenancy / MSP features | Enterprise product, not v1 open source |
| RBAC / SSO / audit trails | Enterprise product layer |
| Billing / usage tracking | Commercial feature, not v1 |
| Cloud-hosted SaaS offering | Self-hosted only for v1, reduces friction |
| Mobile app | Web + Slack/Discord are sufficient interfaces |
| Voice/video avatars | Gimmick for ops use case, adds cost/complexity |
| OAuth subscription support (Pro/Max) | Nice to have, not blocking |
| Blockchain/Web3 integration | Solution without a problem |
| Fully autonomous agents | Dangerous for production ops — always HITL for high-risk |
| Real-time token streaming for all agents | Creates UI noise, doesn't scale to 20+ agents |
| Public agent marketplace | Security nightmare, quality control impossible |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| **INFR-01** | Phase 1: Event Infrastructure | Pending |
| **INFR-02** | Phase 1: Event Infrastructure | Pending |
| **INFR-03** | Phase 1: Event Infrastructure | Pending |
| **INFR-04** | Phase 1: Event Infrastructure | Pending |
| **ROPS-01** | Phase 2: Real Ops Capabilities | Pending |
| **ROPS-02** | Phase 2: Real Ops Capabilities | Pending |
| **ROPS-03** | Phase 2: Real Ops Capabilities | Pending |
| **ROPS-04** | Phase 2: Real Ops Capabilities | Pending |
| **ROPS-05** | Phase 2: Real Ops Capabilities | Pending |
| **ENGN-01** | Phase 2: Real Ops Capabilities | Pending |
| **ENGN-02** | Phase 2: Real Ops Capabilities | Pending |
| **ENGN-03** | Phase 2: Real Ops Capabilities | Pending |
| **ENGN-04** | Phase 2: Real Ops Capabilities | Pending |
| **SREW-01** | Phase 2: Real Ops Capabilities | Pending |
| **SREW-02** | Phase 2: Real Ops Capabilities | Pending |
| **SREW-03** | Phase 2: Real Ops Capabilities | Pending |
| **SREW-04** | Phase 2: Real Ops Capabilities | Pending |
| **MSGG-01** | Phase 3: Messaging Gateway | Pending |
| **MSGG-02** | Phase 3: Messaging Gateway | Pending |
| **MSGG-03** | Phase 3: Messaging Gateway | Pending |
| **MSGG-05** | Phase 3: Messaging Gateway | Pending |
| **MCUI-01** | Phase 4: Mission Control UI | Pending |
| **MCUI-02** | Phase 4: Mission Control UI | Pending |
| **MCUI-03** | Phase 4: Mission Control UI | Pending |
| **MCUI-04** | Phase 4: Mission Control UI | Pending |
| **MCUI-05** | Phase 4: Mission Control UI | Pending |
| **MCUI-06** | Phase 4: Mission Control UI | Pending |
| **MCUI-07** | Phase 4: Mission Control UI | Pending |
| **COMM-05** | Phase 4: Mission Control UI | Pending |
| **PERS-01** | Phase 5: Agent Personas | Pending |
| **PERS-02** | Phase 5: Agent Personas | Pending |
| **PERS-03** | Phase 5: Agent Personas | Pending |
| **PERS-04** | Phase 5: Agent Personas | Pending |
| **PERS-05** | Phase 5: Agent Personas | Pending |
| **MSGG-04** | Phase 5: Agent Personas | Pending |
| **CONV-01** | Phase 6: Conversational Config | Pending |
| **CONV-02** | Phase 6: Conversational Config | Pending |
| **CONV-03** | Phase 6: Conversational Config | Pending |
| **CONV-04** | Phase 6: Conversational Config | Pending |
| **CONV-05** | Phase 6: Conversational Config | Pending |
| **CONV-06** | Phase 6: Conversational Config | Pending |
| **CORD-01** | Phase 7: Coordination Protocols | Pending |
| **CORD-02** | Phase 7: Coordination Protocols | Pending |
| **CORD-03** | Phase 7: Coordination Protocols | Pending |
| **CORD-04** | Phase 7: Coordination Protocols | Pending |
| **CORD-05** | Phase 7: Coordination Protocols | Pending |
| **COMM-01** | Phase 7: Coordination Protocols | Pending |
| **COMM-02** | Phase 7: Coordination Protocols | Pending |
| **COMM-03** | Phase 7: Coordination Protocols | Pending |
| **COMM-04** | Phase 7: Coordination Protocols | Pending |
| **INFR-05** | Phase 8: Production Readiness | Pending |

**Coverage:**
- v1 requirements: 48 total
- Mapped to phases: 48
- Unmapped: 0

**Coverage validation:** ✓ All requirements mapped (100% coverage)

---
*Requirements defined: 2026-02-11*
*Last updated: 2026-02-11 after roadmap creation*
