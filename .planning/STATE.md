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
Plan: 2 of 9 in current phase
Status: Executing
Last activity: 2026-03-12 — Completed 13-02: Agent YAML v1 spec and workspace config spec (docs/spec/)

Progress: [██░░░░░░░░] 22%

---

## Accumulated Context

### Key Decisions

- Pivoted from AOF (personality-driven) → OpenAgentiX (enterprise automation)
- CLI binary named `agentix` (instructions say `agentix`; PROJECT.md says `oax` — instructions win, confirm with user)
- Svelte replaces React for command center; current branch React UI work discarded, build Phase 13 from main
- WASM sandbox introduced in Phase 14 (runtime), enforced as security policy in Phase 19
- Phase 22 (Command Center) depends on Phases 17, 18, 20 — backend must be solid first
- See PROJECT.md Key Decisions table for full list
- Agent YAML uses `apiVersion: openagentix.dev/v1` / `kind: Agent` full Kubernetes style (13-02)
- Unified `spec.tools` list with `type` discriminator (`cli`/`mcp`/`shell`) — no separate tool sections (13-02)
- `provider/model` explicit notation enforced in `spec.model` (exactly one `/` required) (13-02)
- `system_prompt` and `system_prompt_file` are mutually exclusive (13-02)
- Three-tier resolution order: agent YAML > workspace defaults > built-in defaults (13-02)
- Provider credentials are workspace-only (`agentix.yaml`) — cannot be set per-agent (13-02)

### Open Questions

1. **CLI binary name:** Confirmed as `agentix`.
2. **Crate renaming:** Rename `aof-*` crates to match new brand, or keep as internal implementation detail?
3. **Repository rename:** github.com/agenticdevops/aof → github.com/openagentix/openagentix?

### Blockers

- CLI binary name conflict between instructions (`agentix`) and PROJECT.md (`oax`) — resolve before Phase 13.
- Current branch (`sage-mind-yoqmvfyw`) has React UI work being discarded. Phase 13 must build from main.

---

## Session Continuity

Last session: 2026-03-12
Stopped at: Completed 13-02-PLAN.md — Agent YAML v1 spec + workspace config spec
Resume file: None

---

*State tracking initialized: 2026-02-11*
*v1.0 milestone completed: 2026-02-22*
*v2.0 milestone started: 2026-03-12*
*Roadmap created: 2026-03-12*
