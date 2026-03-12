---
phase: 13-rebrand-core-runtime-cli-foundation
plan: 02
subsystem: docs
tags: [spec, agent-directory, gitagent, documentation]

requires:
  - 13-01 (agentix-* crate namespace, clean workspace)
provides:
  - docs/spec/agent-directory-structure.md — primary GitAgent-compatible agent format reference
  - docs/spec/agent-yaml-v1.md — rewritten: minimal agent.yaml manifest (spec_version, name, model.preferred only)
  - docs/spec/workspace-config.md — updated with Agent Discovery section (directories + flat YAML)
affects: [13-03, 13-04, 13-05, 13-06, 13-07, 13-08, 13-09]

tech-stack:
  added: []
  patterns:
    - "Agent-as-directory: agent.yaml is minimal manifest, behavior in SOUL.md + RULES.md + skills/"
    - "System prompt assembly order: SOUL.md + RULES.md + skills/ + knowledge/ + hooks/bootstrap.md"
    - "agents/ subdirectory enables recursive multi-agent hierarchies (lazy loaded)"
    - "AgentManifestFormat enum: Directory (preferred) vs FlatYaml (backward compat)"
    - "spec_version field gates format evolution without breaking existing agents"

key-files:
  created:
    - docs/spec/agent-directory-structure.md
  modified:
    - docs/spec/agent-yaml-v1.md
    - docs/spec/workspace-config.md

key-decisions:
  - "agent.yaml carries only metadata + model preference — all behavior in markdown files (SOUL.md, RULES.md, skills/)"
  - "Runtime behavior fields (max_iterations, timeout, mode) live in workspace defaults, not per-agent agent.yaml"
  - "Both directory and flat YAML formats supported in agents_dir scan (backward compat preserved)"
  - "Sub-agents loaded lazily — only when parent first delegates to them"
  - "extends field enables base agent inheritance via git URL overlay"

requirements-completed: [SPEC-01, SPEC-02, SPEC-04, SPEC-05]

duration: 4min
completed: 2026-03-12
---

# Phase 13 Plan 02: Agent Directory Structure Specification Summary

**GitAgent-compatible directory-based agent format fully documented; agent.yaml rewritten to minimal manifest; workspace-config.md updated with directory-first agent discovery**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-12T17:53:18Z
- **Completed:** 2026-03-12T17:57:00Z
- **Tasks:** 2
- **Files modified:** 3 (1 created, 2 rewritten/updated)

## Accomplishments

- Created `docs/spec/agent-directory-structure.md` as the primary GitAgent format reference: full
  directory layout, required files, system prompt assembly algorithm (5-step, deterministic),
  skills/ composability, tools/ YAML schemas, agents/ recursive multi-agent, hooks/, knowledge/,
  memory/runtime/, validation rules, forward/backward compatibility, 3 concrete examples
- Rewrote `docs/spec/agent-yaml-v1.md` from monolithic YAML spec to minimal manifest spec:
  documents only 6 fields (spec_version, name, version, description, model.preferred, extends,
  dependencies) and explicitly documents what does NOT belong in agent.yaml
- Updated `docs/spec/workspace-config.md`: added Agent Discovery section documenting directory-first
  scanning with flat YAML backward compat; updated spec.defaults docs to explain these are now the
  sole location for runtime behavior fields

## Task Commits

1. **Task 1: Write the agent directory structure specification** - `e059549`
2. **Task 2: Rewrite agent-yaml-v1.md and update workspace-config.md** - `1989ee6`

## Files Created/Modified

- `docs/spec/agent-directory-structure.md` - Created: primary GitAgent format specification (484 lines)
- `docs/spec/agent-yaml-v1.md` - Rewritten: minimal manifest spec (was monolithic YAML, 633 → 306 lines)
- `docs/spec/workspace-config.md` - Updated: Agent Discovery section + updated defaults explanation

## Decisions Made

- `agent.yaml` is strictly minimal: only `spec_version`, `name`, `version`, `description`,
  `model.preferred`, `extends`, `dependencies`. Behavior fields removed.
- `max_iterations`, `timeout`, `mode` now live only in workspace `spec.defaults` (since they're
  no longer in `agent.yaml`, the workspace is the only place to configure them per-workspace).
- Both directory format and flat YAML format are auto-detected from `agents_dir` scanning — no
  explicit config needed.
- `sub-agents` under `agents/` are lazy-loaded — runtime doesn't load them at startup.

## Deviations from Plan

None — plan executed exactly as written.

---

## Self-Check

**Files exist:**
- docs/spec/agent-directory-structure.md: FOUND
- docs/spec/agent-yaml-v1.md: FOUND
- docs/spec/workspace-config.md: FOUND

**Commits exist:**
- e059549: FOUND
- 1989ee6: FOUND

## Self-Check: PASSED

---
*Phase: 13-rebrand-core-runtime-cli-foundation*
*Completed: 2026-03-12*
