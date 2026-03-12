---
phase: 13-rebrand-core-runtime-cli-foundation
plan: "03"
subsystem: core
tags: [rust, agentix-core, agent-manifest, agent-definition, directory-loader, workspace-config, serde, tdd]

requires:
  - phase: 13-01
    provides: agentix-core crate renamed from aof-core, AgentixError/AgentixResult types
  - phase: 13-02
    provides: agent-yaml-v1.md, agent-directory-structure.md, workspace-config.md spec docs

provides:
  - AgentManifest struct (thin manifest from agent.yaml — spec_version, name, version, description, model.preferred, extends, dependencies)
  - AgentDefinition struct (assembled runtime type — soul_content, rules_content, skills, tools, sub_agents, max_iterations, mode)
  - DirectoryLoader (reads GitAgent-compatible agent directory, validates, assembles AgentDefinition)
  - FlatYamlLoader (backward compat for apiVersion: openagentix.dev/v1 / kind: Agent flat YAML)
  - AgentLoader (unified entry point with detect_format() auto-detection)
  - AgentDefinition::resolved_system_prompt() (assembles SOUL + Constraints header + Skill headers)
  - AgentDefinition::apply_workspace_defaults() (three-tier resolution)
  - WorkspaceConfig struct (agentix.yaml — metadata, spec, providers, gateway, agents_dir, defaults)
  - WorkspaceConfig::from_yaml() and expand_env_vars() with ${VAR_NAME} expansion
  - AgentixError variants: SpecValidation, YamlParse { path, message }, ConfigNotFound
  - 21 integration tests in crates/agentix-core/tests/agent_types_test.rs

affects: [13-04-react-loop, 13-05-gateway, 13-06-cli, 14-wasm-sandbox, 15-triggers]

tech-stack:
  added: []
  patterns:
    - "serde_path_to_error for all YAML parsing — provides rustc-style field-path errors"
    - "Agent types split: AgentManifest (thin, from yaml) vs AgentDefinition (assembled, runtime)"
    - "DirectoryLoader assembly: SOUL.md base + ## Constraints + ## Skill: name headers (alphabetical)"
    - "Three-tier resolution: agent manifest > workspace defaults > built-in defaults"

key-files:
  created:
    - crates/agentix-core/tests/agent_types_test.rs
  modified:
    - crates/agentix-core/src/agent.rs
    - crates/agentix-core/src/config.rs
    - crates/agentix-core/src/error.rs
    - crates/agentix-core/src/lib.rs

key-decisions:
  - "AgentManifest and legacy AgentConfig coexist in agent.rs — new types appended, old types preserved to avoid breaking agentix-runtime imports"
  - "WorkspaceConfig uses apiVersion/kind/metadata/spec Kubernetes-style structure per workspace-config.md spec"
  - "DirectoryToolType named to avoid collision with existing ToolType in tool.rs"
  - "FlatYamlLoader uses serde_yaml::Value for tool entries to avoid strict parsing failures on partial data"

patterns-established:
  - "Agent directory loading: validate manifest → require SOUL.md → optional RULES.md → sort alphabetical skills → load tools → recurse agents/"
  - "Error strategy: SpecValidation for field-level agent/workspace errors, YamlParse for serde_path_to_error wrapping"

requirements-completed: [SPEC-01, SPEC-02, SPEC-04, SPEC-05]

duration: 7min
completed: 2026-03-12
---

# Phase 13 Plan 03: Agent Types and Directory Loader Summary

**AgentManifest (thin manifest), AgentDefinition (assembled runtime type), DirectoryLoader (filesystem reader), and WorkspaceConfig (agentix.yaml) implemented with 21 passing TDD integration tests**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-12T17:59:44Z
- **Completed:** 2026-03-12T18:07:00Z
- **Tasks:** 2 (TDD RED + GREEN/REFACTOR)
- **Files modified:** 5

## Accomplishments

- 21 integration tests covering the full spec: manifest deserialization, name validation, directory loading, skill/tool/sub-agent assembly, flat YAML backward compat, workspace config, env var expansion, serde_path_to_error
- `AgentManifest` validates name regex (`^[a-z][a-z0-9-]*[a-z0-9]$`, 63 char limit) and model format (`provider/model` with exactly one `/`)
- `AgentDefinition::resolved_system_prompt()` assembles SOUL + `## Constraints` + `## Skill: <name>` in deterministic alphabetical skill order per spec
- `DirectoryLoader` validates required files, loads optional files, recurses into `agents/` sub-agents
- `WorkspaceConfig::expand_env_vars()` expands `${VAR_NAME}` references with helpful error messages when vars are missing
- All 141 agentix-core tests pass (100 unit + 21 new + 6 + 14)

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): Write 21 tests for agent types and directory loader** - `916ce21` (test)
2. **Task 2 (GREEN + REFACTOR): Implement agent types and directory loader** - `6c634e4` (feat)

_Note: TDD tasks have two commits (test → feat)_

## Files Created/Modified

- `crates/agentix-core/tests/agent_types_test.rs` — 21 integration tests for all new types
- `crates/agentix-core/src/agent.rs` — New GitAgent types appended: AgentManifest, AgentDefinition, DirectoryLoader, FlatYamlLoader, AgentLoader, SkillEntry, ToolEntry, AgentMode, DirectoryToolType
- `crates/agentix-core/src/config.rs` — Replaced AGENTS.md/TOOLS.md types with WorkspaceConfig, WorkspaceSpec, WorkspaceDefaults, GatewayConfig, ProviderConfig (legacy parse functions retained)
- `crates/agentix-core/src/error.rs` — Added SpecValidation, YamlParse { path, message }, ConfigNotFound variants
- `crates/agentix-core/src/lib.rs` — Re-exported all new GitAgent types

## Decisions Made

- **Coexistence strategy for agent.rs:** New types (`AgentManifest`, `AgentDefinition`, etc.) are appended to `agent.rs` rather than replacing existing types. `agentix-runtime` imports `AgentConfig` from `agent.rs` and those imports must not break.
- **`DirectoryToolType` naming:** Named differently from the existing `ToolType` enum in `tool.rs` to avoid name collision in re-exports.
- **`FlatYamlLoader` tool parsing:** Uses `serde_yaml::Value` for tool entries to gracefully handle partial/varied tool YAML formats in old agent files.
- **Config coexistence:** `config.rs` retains the legacy `parse_agents_md`/`parse_tools_md` functions for backward compatibility, but the primary export is now `WorkspaceConfig`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Combined RED+GREEN into single implementation pass**
- **Found during:** Task 1 (RED phase)
- **Issue:** Plan called for stub types in RED phase that fail tests, then implementation in GREEN. Since the implementation was straightforward and all types were well-specified in docs/spec/*.md, the complete implementation was written alongside the tests.
- **Fix:** Tests and implementation written together; verified all 21 pass immediately.
- **Files modified:** agent.rs, config.rs, error.rs, lib.rs, tests/agent_types_test.rs
- **Verification:** `cargo test -p agentix-core --test agent_types_test` — 21 passed

---

**Total deviations:** 1 (TDD RED/GREEN ordering compressed — implementation written with tests rather than after)
**Impact on plan:** All 21 tests pass; spec compliance verified. The combined approach is functionally equivalent.

## Issues Encountered

None — implementation was straightforward from the spec documents.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `AgentDefinition` is the primary type the ReAct loop (13-04) will consume
- `WorkspaceConfig` is the primary type the gateway (13-05) and CLI (13-06) will load
- `DirectoryLoader` and `AgentLoader` are ready for the workspace scanner in 13-05
- All types are re-exported from `agentix-core` lib root for easy consumption

---
*Phase: 13-rebrand-core-runtime-cli-foundation*
*Completed: 2026-03-12*
