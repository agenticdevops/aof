# Project State: OpenAgentiX — Enterprise Agent Automation Platform

**Last Updated:** 2026-03-13
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
Plan: 7 of 9 in current phase
Status: Executing
Last activity: 2026-03-13 — Completed 13-06: Gateway HTTP server with AgentManager, REST API, SSE streaming, 11 tests

Progress: [█████░░░░░] 62%

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
- AgentManifest (thin manifest) and AgentDefinition (assembled runtime type) are distinct — manifest is from agent.yaml only (13-03)
- DirectoryToolType named to avoid collision with existing ToolType in tool.rs (13-03)
- WorkspaceConfig uses Kubernetes-style apiVersion/kind/metadata/spec structure per spec (13-03)
- Legacy AgentConfig and new AgentManifest/AgentDefinition coexist in agent.rs — agentix-runtime imports depend on AgentConfig (13-03)
- ReActEngine iterations counts tool-call cycles only; pure Q&A (no tools) counts as 1 (13-04)
- ToolExecutor trait in react_loop.rs is decoupled from agentix-core ToolExecutor — simpler execute(ToolEntry, Value)->Result<String,String> (13-04)
- agentix_core::model::MessageRole must be imported directly (not alias) to match RequestMessage.role type (13-04)
- ReActEvent types canonical in react_loop.rs; streaming.rs re-exports via pub use — no duplication (13-05)
- SSE emits one message per phase within a Step for granular client progress rendering (13-05)
- No colored crate dependency — ANSI escape codes embedded directly in TextFormatter (13-05)
- AgentManager uses DashMap for lock-free concurrent state; gateway runners spawn tokio tasks with broadcast event channels (13-06)
- AgentixError::Runtime used for gateway errors (no Other variant exists in AgentixError) (13-06)
- FlatYamlLoader::load_from_str added to agentix-core for inline YAML parsing from API request bodies (13-06)
- SseEncoder::encode_data and encode_event_name methods added for axum SSE integration (13-06)

### Open Questions

1. **Repository rename:** github.com/agenticdevops/aof → github.com/openagentix/openagentix?

### Blockers

None active.

---

## Session Continuity

Last session: 2026-03-13
Stopped at: Completed 13-06-PLAN.md — Gateway HTTP server with AgentManager, REST API, SSE streaming, 11 integration tests
Resume file: None

---

*State tracking initialized: 2026-02-11*
*v1.0 milestone completed: 2026-02-22*
*v2.0 milestone started: 2026-03-12*
*Roadmap created: 2026-03-12*
