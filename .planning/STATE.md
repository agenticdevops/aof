# Project State: OpenAgentiX — Enterprise Agent Automation Platform

**Last Updated:** 2026-03-12
**Milestone:** v2.0 OpenAgentiX
**Status:** Executing Phase 13

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-12)

**Core value:** Let any technical organization automate operational tasks with AI agents — without writing Python, without managing infrastructure, without giving up control.
**Current focus:** Phase 13 — Rebrand + Core Runtime + CLI Foundation

---

## Current Position

Phase: 13 of 22 (Rebrand + Core Runtime + CLI Foundation)
Plan: 4 of 9 in current phase
Status: Executing
Last activity: 2026-03-12 — Completed 13-02: Agent directory structure spec, minimal agent.yaml rewrite, workspace-config.md update

Progress: [████░░░░░░] 44%

---

## Accumulated Context

### Key Decisions

- Pivoted from AOF (personality-driven) → OpenAgentiX (enterprise automation)
- CLI binary named `agentix` (confirmed)
- Svelte replaces React for command center; current branch React UI work discarded, build Phase 13 from main
- WASM sandbox introduced in Phase 14 (runtime), enforced as security policy in Phase 19
- Phase 22 (Command Center) depends on Phases 17, 18, 20 — backend must be solid first
- See PROJECT.md Key Decisions table for full list
- All crates renamed from aof-* to agentix-*, AofError/AofResult aliased to AgentixError/AgentixResult (13-01)
- schema.rs kept in agentix-core (agent.rs depends on it) — not deleted despite being in v1.0 list (13-01)
- Agent YAML uses `apiVersion: openagentix.dev/v1` / `kind: Agent` full Kubernetes style (13-02 old spec, now superseded)
- Unified `spec.tools` list with `type` discriminator (`cli`/`mcp`/`shell`) — no separate tool sections (13-02)
- `provider/model` explicit notation enforced in `spec.model` (exactly one `/` required) (13-02)
- `system_prompt` and `system_prompt_file` are mutually exclusive (13-02)
- Three-tier resolution order: agent YAML > workspace defaults > built-in defaults (13-02)
- Provider credentials are workspace-only (`agentix.yaml`) — cannot be set per-agent (13-02)
- Agent definitions are directories (GitAgent-compatible): agent.yaml = minimal manifest only (13-02)
- agent.yaml carries only spec_version, name, version, description, model.preferred, extends, dependencies (13-02)
- Behavior lives in SOUL.md + RULES.md + skills/ — NOT in agent.yaml (13-02)
- Runtime behavior fields (max_iterations, timeout, mode) moved to workspace defaults only (13-02)
- agents_dir scanning: directory with agent.yaml = GitAgent format; flat *.yaml = backward compat (13-02)
- Sub-agents (agents/ subdirectory) are lazy-loaded on first delegation from parent (13-02)

### Open Questions

1. **Repository rename:** github.com/agenticdevops/aof → github.com/openagentix/openagentix?

### Blockers

None active.

---

## Session Continuity

Last session: 2026-03-12
Stopped at: Completed 13-02-PLAN.md — Agent directory spec, minimal agent.yaml rewrite, workspace-config.md update
Resume file: None

---

*State tracking initialized: 2026-02-11*
*v1.0 milestone completed: 2026-02-22*
*v2.0 milestone started: 2026-03-12*
*Roadmap created: 2026-03-12*
