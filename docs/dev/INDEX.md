# AOF Internal Developer Documentation Index

**Last Updated:** 2026-02-14
**Status:** Phase 6 Complete (5/5 plans) | Phase 7 Planning

This is the central index for AOF's internal development documentation. Use this to navigate architecture docs, implementation guides, and phase-specific details.

---

## 🏗️ Architecture & Design

### Core Architecture
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Crate structure, dependency diagram, core traits
  - Overview of 10+ crates (aof-core, aof-llm, aof-runtime, aof-memory, etc.)
  - Module responsibilities and trait definitions
  - Recommended reading: Start here for new contributors

### Design Documents
- **[AGENTFLOW_DESIGN.md](./AGENTFLOW_DESIGN.md)** - Multi-agent workflow execution
- **[decision-logging.md](./decision-logging.md)** - Decision audit trail system
- **[prompt-composition.md](./prompt-composition.md)** - System prompt construction
- **[resource-locking.md](./resource-locking.md)** - Resource management & concurrency
- **[sandbox-isolation.md](./sandbox-isolation.md)** - Sandbox security boundaries

---

## 📋 Phase Implementation Guides

### Phase 1: Event Infrastructure ✅
- **[event-infrastructure.md](./event-infrastructure.md)** - WebSocket, session persistence, event broadcast
  - Pub/sub messaging, daemon lifecycle, recovery mechanisms

### Phase 2: Real Ops Capabilities ✅
- **[skills-platform.md](./skills-platform.md)** - SKILL.md format, skill registry, requirements gating
- **[incident-response.md](./incident-response.md)** - Incident triage, decision logging, escalation

### Phase 3: Messaging Gateway ✅
- (Gateway docs in `/docs/internal/design/`)
- Slack, Discord adapters; NAT-transparent WebSocket

### Phase 4: Mission Control UI 🔄
- **[persona-system.md](./persona-system.md)** - AGENTS.md/SOUL.md, avatar system, personality model
- **[persona-loaders.md](./persona-loaders.md)** - Workspace file parsing, persona instantiation
- **[persona-ui-components.md](./persona-ui-components.md)** - AgentCard, AgentGrid React components

### Phase 5: Agent Personas ✅
- (Persona system fully implemented, see Phase 4 docs above)

### Phase 6: Conversational Configuration ✅
- **[PHASE-6-IMPLEMENTATION-SUMMARY.md](./PHASE-6-IMPLEMENTATION-SUMMARY.md)** - Complete phase overview
  - 5 plans delivered, 109/110 tests passing, 42 tasks
  - Architecture, intent taxonomy, specialist handlers, REST API, React UI
- **[conversational-architecture.md](./conversational-architecture.md)** - Full technical architecture
  - 3-tier architecture, session management, future enhancements
  - Intent flow diagrams, LRU cache design, security patterns
- **[conversation-api.md](./conversation-api.md)** - REST API reference & testing guide
  - 5 endpoints: session CRUD, message send, file confirm/cancel
  - Manual E2E testing steps, Redux integration
- **[squad-templates.md](./squad-templates.md)** - Squad template system
  - Embedded templates (incident-response, monitoring, deployment, cost-optimization)
  - Template structure, customization, adding new templates
- **[agent-generation-pipeline.md](./agent-generation-pipeline.md)** - Agent creation flow
  - Parameter extraction, prompt composition, file generation

### Phase 7: Coordination Protocols (⏰ Planning)
- [docs/internal/design/](../internal/design/) - Coordination specs (Byzantine, Raft, gossip)

### Phase 8: Production Readiness (⏰ Planning)
- Deployment, auth, monitoring (to be documented)

---

## 🛠️ Core Subsystems

### Agent & Skill Systems
- **[persona-system.md](./persona-system.md)** - Agent persona definition (SOUL.md)
- **[skills-platform.md](./skills-platform.md)** - Skill registry and execution
- **[prompt-composition.md](./prompt-composition.md)** - System prompt building

### Execution & Coordination
- **[AGENTFLOW_DESIGN.md](./AGENTFLOW_DESIGN.md)** - Workflow DAGs, agent composition
- **[incident-response.md](./incident-response.md)** - Triage and escalation flows
- **[resource-locking.md](./resource-locking.md)** - Concurrent resource access

### Networking & Events
- **[event-infrastructure.md](./event-infrastructure.md)** - WebSocket, session persistence
- **[sandbox-isolation.md](./sandbox-isolation.md)** - Execution sandboxes

### Reliability & Observability
- **[decision-logging.md](./decision-logging.md)** - Audit trails, decision rationale
- **[reliability-metrics.md](./reliability-metrics.md)** - SLA tracking, health checks

---

## 📚 External (User-Facing) Documentation

For end-user guides, see [`/docs/`](../README.md):
- Getting started, feature guides, API reference
- User-facing tutorials and examples
- Installation and deployment instructions

---

## 🔍 Navigation by Topic

### By Audience

**New Contributors:**
1. Read [ARCHITECTURE.md](./ARCHITECTURE.md) (15 min) — understand crate structure
2. Pick a phase: Read phase-specific doc (30 min) — see how things fit together
3. Read [CONTRIBUTING.md](./CONTRIBUTING.md) (10 min) — submission guidelines
4. Start with a small issue or refactor

**LLM/Agent Developers:**
1. [persona-system.md](./persona-system.md) — How agents are defined and loaded
2. [conversational-architecture.md](./conversational-architecture.md) — Intent classification & specialization
3. [agent-generation-pipeline.md](./agent-generation-pipeline.md) — End-to-end agent creation

**Ops/Incident Response Developers:**
1. [incident-response.md](./incident-response.md) — Incident triage, decision logging
2. [skills-platform.md](./skills-platform.md) — Skill definitions for runbooks
3. [decision-logging.md](./decision-logging.md) — Audit trails for compliance

**Infrastructure/DevOps Developers:**
1. [event-infrastructure.md](./event-infrastructure.md) — WebSocket, session persistence
2. [sandbox-isolation.md](./sandbox-isolation.md) — Execution security boundaries
3. [resource-locking.md](./resource-locking.md) — Resource contention management

---

## 📊 Implementation Status

### Completed Phases (6)
```
Phase 1: Event Infrastructure          ✅ Complete (3/3 plans)
Phase 2: Real Ops Capabilities         ✅ Complete (3/3 plans)
Phase 3: Messaging Gateway             ✅ Complete (3/3 plans)
Phase 4: Mission Control UI            ✅ Complete (4/4 plans) [Phase 5 integration pending]
Phase 5: Agent Personas                ✅ Complete (6/6 plans)
Phase 6: Conversational Configuration  ✅ Complete (5/5 plans)
```

### In Progress
```
Phase 7: Coordination Protocols         ⏰ Planning
Phase 8: Production Readiness           ⏰ Planning
```

### Documentation Metrics
- **Phase-specific docs:** 20+ files
- **Architecture docs:** 10+ files
- **API documentation:** REST endpoints, WebSocket events
- **Testing guides:** MockModel patterns, integration tests

---

## 🔗 Cross-References

### By Feature

**Agent Persona System**
- Definition: [persona-system.md](./persona-system.md)
- File Loading: [persona-loaders.md](./persona-loaders.md)
- UI Rendering: [persona-ui-components.md](./persona-ui-components.md)
- Implemented in: Phase 5

**Conversational Configuration**
- Architecture: [conversational-architecture.md](./conversational-architecture.md)
- API Reference: [conversation-api.md](./conversation-api.md)
- Squad Templates: [squad-templates.md](./squad-templates.md)
- Agent Pipeline: [agent-generation-pipeline.md](./agent-generation-pipeline.md)
- Implemented in: Phase 6

**Incident Response**
- Triage & Escalation: [incident-response.md](./incident-response.md)
- Decision Logging: [decision-logging.md](./decision-logging.md)
- Resource Locking: [resource-locking.md](./resource-locking.md)
- Implemented in: Phase 2

---

## 📖 Reading Paths

### Path 1: "I want to understand AOF architecture" (45 min)
1. [ARCHITECTURE.md](./ARCHITECTURE.md) (15 min)
2. [event-infrastructure.md](./event-infrastructure.md) (15 min)
3. [AGENTFLOW_DESIGN.md](./AGENTFLOW_DESIGN.md) (15 min)

### Path 2: "I want to build conversational features" (90 min)
1. [conversational-architecture.md](./conversational-architecture.md) (30 min)
2. [agent-generation-pipeline.md](./agent-generation-pipeline.md) (20 min)
3. [conversation-api.md](./conversation-api.md) (20 min)
4. [squad-templates.md](./squad-templates.md) (20 min)

### Path 3: "I want to add a new skill or tool" (60 min)
1. [skills-platform.md](./skills-platform.md) (20 min)
2. [incident-response.md](./incident-response.md) (20 min)
3. [decision-logging.md](./decision-logging.md) (20 min)

### Path 4: "I want to implement coordination protocols" (120 min)
1. [AGENTFLOW_DESIGN.md](./AGENTFLOW_DESIGN.md) (30 min)
2. [resource-locking.md](./resource-locking.md) (30 min)
3. [sandbox-isolation.md](./sandbox-isolation.md) (30 min)
4. [docs/internal/design/](../internal/design/) specs (30 min)

---

## 🎯 Quick Reference

### Key Crates
| Crate | Purpose | Docs |
|-------|---------|------|
| `aof-core` | Core traits, types, events | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| `aof-llm` | LLM provider abstraction | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| `aof-conversational` | Intent classification & specialists | [conversational-architecture.md](./conversational-architecture.md) |
| `aof-personas` | Agent persona system | [persona-system.md](./persona-system.md) |
| `aofctl` | CLI binary with REST API & WebSocket | [event-infrastructure.md](./event-infrastructure.md) |

### Key Traits
| Trait | Purpose | Location |
|-------|---------|----------|
| `Model` | LLM abstraction (provider-agnostic) | `aof-llm` |
| `Agent` | Agent execution | `aof-core` |
| `Tool` | Tool definition | `aof-core` |
| `Specialist` | Intent handler (Phase 6) | `aof-conversational` |

### Key Files
| File | Purpose | Path |
|------|---------|------|
| AGENTS.md | Agent registry | `workspace/AGENTS.md` |
| SOUL.md | Agent personality | `workspace/SOUL.md` |
| SKILL.md | Skill definition | `workspace/skills/SKILL.md` |
| triggers.yaml | Event triggers | `workspace/triggers.yaml` |

---

## 🔧 Contributing

Before submitting changes:
1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Update relevant docs in this folder
3. Add tests (follow MockModel pattern from Phase 6)
4. Link new docs to this INDEX.md

**Documentation expectations:**
- Architecture changes: Update [ARCHITECTURE.md](./ARCHITECTURE.md)
- Phase-specific changes: Update phase doc (e.g., [conversational-architecture.md](./conversational-architecture.md))
- New subsystems: Create subsystem doc and link from INDEX.md
- This is a living document — keep it updated as AOF evolves

---

## 📞 Questions?

- **Architecture questions:** See [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Phase-specific questions:** See phase docs listed above
- **API questions:** See [conversation-api.md](./conversation-api.md)
- **Testing questions:** See phase docs (look for "Test Coverage" sections)
- **Contributing questions:** See [CONTRIBUTING.md](./CONTRIBUTING.md)

---

**Last Updated:** 2026-02-14
**Milestone:** Reinvention (Humanized Agent Platform)
**Next Phase:** Phase 7 - Coordination Protocols
