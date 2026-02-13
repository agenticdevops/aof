# Phase 2: Real Ops Capabilities - Context

**Gathered:** 2026-02-12
**Status:** Ready for planning
**Architecture Alignment:** OpenClaw hub-and-spoke, composable prompts, sandbox isolation

---

<domain>
## Phase Boundary

Agents can perform real DevOps work with full decision transparency and safe coordination.

What this includes:
- **K8s diagnostics** — Agents diagnose pod crashes, analyze logs, inspect metrics
- **Incident response** — Triage agent routes alerts to specialist agents (log analyzer, metric checker, K8s diagnostician)
- **Skills platform** — Agents discover and execute operational skills from filesystem (SKILL.md format, agentskills.io standard)
- **Decision logging** — All agent decisions logged to a shared "virtual office" (chat-like, searchable, visible to fleet)
- **Safe execution** — Destructive operations (restart, delete, scale) are serialized via resource locks (TTL-based)
- **Subagent spawning** — Parent agents can spawn specialist children with context pull model

What this does NOT include:
- Conversational configuration (Phase 6)
- Personas/character (Phase 5)
- UI/Mission Control (Phase 4)
- Messaging gateway integration (Phase 3)

</domain>

<decisions>
## Implementation Decisions

### Incident Response Flow

**Triage approach:** Hybrid (quick classification → targeted spawn)
- Alert fires → triage classifies severity (LLM-based routing)
- Spawn only specialists needed for that alert type
- Specialist agents pull context from shared store as needed

**Specialist coordination:**
- LLM-based routing: Triage uses LLM to understand alert and route to specialists
- Context pull: Specialists query shared context store (not pushed by triage)
- Enables independence: Each specialist drives its own investigation

**Escalation trigger:** Hybrid (AI recommends + human approves)
- Agents assess confidence levels, recommend escalation
- Low-severity escalations auto-approve
- Human-in-the-loop for critical escalations
- Escalation routes to: humans, other fleet agents, knowledge base

### Skills & Tool Discovery

**Skill format:** Standard agentskills.io + compatible with Claude, Codex formats
- Skills live as SKILL.md files in filesystem
- Single standard format (markdown-based)
- Version-controlled, transparent, portable
- Agents scan filesystem on startup; filesystem is the source of truth

**Skill updates:** Always latest
- Agents always use latest version of skills
- No pinning, no versioning per-agent
- Assumes skills are backward compatible or breaking changes communicated
- Simple approach, relies on skill author responsibility

**Skill gaps:** Confidence-driven escalation
- Agents learn from similar skills/examples
- If confident (>70%), attempt task using raw tools
- If not confident, create task for humans to build skill
- All attempts logged with confidence level and reasoning
- If still failing after human-built skill, escalate to human for approval

### Decision Transparency

**Shared virtual office model:**
- All decision logs go to central hub visible to fleet + humans
- Serves multiple purposes: audit trail + communication + context for other agents
- Chat-like format (Slack-style messages)
- Agents log in real-time as they make decisions

**Decision log content:**
- Agent name, action taken, reasoning, confidence level, timestamp
- Links to related decisions (if following up on earlier decision)
- Tags for searchability (agent, action type, resource, severity)

**Search capabilities:** Both semantic + structured
- Semantic: "What happened with pod crashes?" finds related decisions
- Structured: agent=ops-bot, action=restart, confidence>80%
- Agents can query to find patterns/context before acting

**Knowledge base:** Docusaurus-like portal
- Agents and humans write postmortems, learnings, detailed articles
- Searchable knowledge base for operational playbooks
- Builds over time as incidents occur

**Log routing:**
- Low confidence decisions → escalate to humans
- Known patterns with solutions → suggest to agents
- Unusual situations → notify relevant fleet members
- All decisions accessible to fleet for learning

### Resource Collision Prevention

**Scope:** Destructive operations only, per-resource
- Destructive = restart, delete, scale, terminate
- Read operations = get logs, get status, inspect metrics (can run in parallel)
- Lock is per-resource (Pod A can lock while Pod B operates freely)

**Lock mechanism:** Distributed lock with TTL
- Locks expire after 30 seconds (or configurable TTL)
- Agent must renew lock if operation takes longer
- Crash = lock auto-releases after TTL
- Simple, self-healing, no manual cleanup needed

**Lock conflict behavior:** Block and wait
- If Agent A locks resource, Agent B blocks and waits
- Agent B waits for lock to release (via TTL expiry)
- Simple and safe
- Serializes operations on same resource naturally

### Sandbox & Isolation

**Execution model:** Inherit OpenClaw's sandbox patterns
- Host-level access for trusted operations (main agent responsibilities)
- Sandbox isolation per session type or risk level
- Docker-based tool execution for untrusted tools
- File permissions restrict credential access

**Credential storage:** Restricted file permissions
- Agent credentials stored locally with file-level access control
- No credential sharing across agents unless explicit
- Follows principle of least privilege

</decisions>

<specifics>
## Specific Requirements

- **Virtual office implementation:** Chat-like interface in existing communication channel (Slack, Discord, or internal portal)
- **Skill format:** Strictly agentskills.io standard, tested against Claude/Codex compatibility
- **Decision logging frequency:** Log at every significant decision point (not every internal thought)
- **Resource lock timeout:** Default 30s, configurable per operation type
- **Fleet size support:** Minimum 5 concurrent agents, tested up to 20+

</specifics>

<deferred>
## Deferred Ideas

- **Scheduled skills** — Agents on timers, separate from incident response (Phase 7: Coordination)
- **Skill marketplace** — Publishing skills to central registry (considered but deferred; filesystem-only for Phase 2)
- **Advanced routing** — Rule engines or graph-based routing (LLM-based sufficient for now)
- **Transaction support** — Multi-resource atomic operations (out of scope; Phase 2 is single-resource)
- **Confidence calibration** — ML-based confidence threshold tuning (future: Phase 8+)

</deferred>

---

**Architecture:** Adopts OpenClaw hub-and-spoke model with composable prompts and sandbox isolation
**Dependencies:** Phase 1 (event infrastructure for decision logging)
**Parallelization:** Can run alongside Phase 3 (Messaging Gateway) — separate crates

*Phase: 02-real-ops-capabilities*
*Context gathered: 2026-02-12*
*Alignment: OpenClaw architecture patterns*
