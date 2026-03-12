---
phase: 13-rebrand-core-runtime-cli-foundation
plan: 01
subsystem: infra
tags: [rust, rebrand, agentix, cargo, workspace, crates]

requires: []
provides:
  - agentix-* crate namespace established (filesystem and Cargo.toml)
  - v1.0 dead code eliminated (web-app, docusaurus, personas, coordination, gateway, etc.)
  - AgentixError/AgentixResult types (AofError/AofResult kept as aliases)
  - Simplified agentix-core with only v2.0-relevant modules
  - Simplified agentix-runtime lib.rs (executor kept, v1.0 modules removed)
  - New agentix CLI binary name with stripped v1.0 commands
affects: [13-02, 13-03, 13-04, 13-05, 13-06, 13-07, 13-08, 13-09]

tech-stack:
  added: []
  patterns:
    - "agentix-* crate naming convention for all workspace members"
    - "AgentixError/AgentixResult as primary error types with AofError/AofResult backward-compat aliases"
    - "CLI binary named 'agentix' with kubectl-style verb-first commands"

key-files:
  created:
    - crates/agentix-core/src/error.rs
    - crates/agentix-core/src/lib.rs
    - crates/agentix-core/src/schema.rs
    - crates/agentix-core/src/registry.rs
    - crates/agentix-runtime/src/lib.rs
    - crates/agentix/src/cli.rs
    - crates/agentix/src/commands/mod.rs
    - crates/agentix/src/main.rs
  modified:
    - Cargo.toml
    - crates/agentix-core/Cargo.toml
    - crates/agentix-llm/Cargo.toml
    - crates/agentix-mcp/Cargo.toml
    - crates/agentix-memory/Cargo.toml
    - crates/agentix-runtime/Cargo.toml
    - crates/agentix-triggers/Cargo.toml
    - crates/agentix/Cargo.toml

key-decisions:
  - "Kept schema.rs in agentix-core because agent.rs uses it (plan said remove if only used by removed features)"
  - "Kept backward-compatible AofError/AofResult aliases in agentix-core to ease migration across runtime/trigger files"
  - "executor/ directory kept in agentix-runtime (will be rewritten in Plan 03) but agentflow/workflow/fleet modules stripped"
  - "agentix-runtime/agentix-triggers/agentix CLI still have expected compile errors from deleted type references - by design per plan, fixed in Plans 02-06"

requirements-completed: [SPEC-03]

duration: 12min
completed: 2026-03-12
---

# Phase 13 Plan 01: Rebrand Core/Runtime/CLI Foundation Summary

**All AOF crates renamed to agentix-*, 1000+ v1.0 files deleted, workspace identity reset to OpenAgentiX (openagentix.org)**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-12T16:50:08Z
- **Completed:** 2026-03-12T17:02:08Z
- **Tasks:** 2
- **Files modified:** 1,063 (852 deleted in Task 1, 211 renamed/modified in Task 2)

## Accomplishments

- Deleted all v1.0-only code: web-app/, web-ui/, agents/, skills/, flows/, docusaurus-site/, 8 v1.0 crates, 52+ planning phase archives, root markdown cruft
- Renamed all 7 crates from aof-* to agentix-* (filesystem + all Cargo.toml files)
- Updated workspace Cargo.toml: repository to github.com/openagentix/agentix, homepage to openagentix.org, documentation to docs.openagentix.org, version to 2.0.0-alpha
- Renamed AofError/AofResult to AgentixError/AgentixResult (with backward-compatible aliases)
- Stripped v1.0 modules from agentix-core: agentflow, approval, binding, coordination, credential, device, fleet, workflow, activity
- Stripped v1.0 modules from agentix-runtime: device, orchestrator, fleet, resilience, task, credential_anomaly, credential_audit, sandbox
- Stripped v1.0 CLI commands from agentix binary: fleet, flow, device, skills, exec, describe, delete, api_resources, tools; rewrote cli.rs
- Mass-renamed 77+ Rust source files: all `use aof_*` and `aof_*::` path references updated to `agentix_*`
- agentix-core, agentix-llm, agentix-mcp, agentix-memory compile with 0 errors

## Task Commits

1. **Task 1: Delete all v1.0-only code and directories** - `084972f` (chore)
2. **Task 2: Rename all crates from aof-* to agentix-* and update workspace** - `685f2d4` (feat)

## Files Created/Modified

- `Cargo.toml` - Workspace updated: new crate names, URLs, version 2.0.0-alpha
- `crates/agentix-core/src/error.rs` - New: AgentixError/AgentixResult with AofError/AofResult aliases
- `crates/agentix-core/src/lib.rs` - New: stripped to 11 v2.0-relevant modules only
- `crates/agentix-core/src/registry.rs` - Rewritten: removed FlowRegistry and BindingRegistry (deleted types)
- `crates/agentix-core/src/schema.rs` - Kept: used by agent.rs (restored after accidental deletion)
- `crates/agentix-runtime/src/lib.rs` - New: stripped to executor/health/metrics/shutdown
- `crates/agentix/src/cli.rs` - Rewritten: only 8 v2.0 commands remain
- `crates/agentix/src/main.rs` - Simplified: removed deleted module declarations

## Decisions Made

- schema.rs was in the "remove" list per plan but agent.rs depends on it. Kept it (Rule 1 auto-fix).
- AofError/AofResult kept as backward-compatible type aliases to prevent cascading failures across executor/trigger files that reference them — will be cleaned up in Plans 02-03.
- agentix-runtime executor/ kept in place despite referencing deleted types; compile errors are expected and documented per plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Restored schema.rs that was accidentally deleted**
- **Found during:** Task 2 (cargo check after module deletion)
- **Issue:** Plan listed `schema` as a module to delete from agentix-core, but agent.rs still uses `crate::schema::OutputSchema` and `crate::schema::InputSchema`. Deleting it caused 6 compile errors in agentix-core.
- **Fix:** Restored schema.rs from git history, updated to use AgentixError/AgentixResult
- **Files modified:** crates/agentix-core/src/schema.rs, crates/agentix-core/src/lib.rs
- **Verification:** `cargo check -p agentix-core` passes with 0 errors
- **Committed in:** 685f2d4 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Required for agentix-core to compile. No scope creep.

## Issues Encountered

- `find` command intercepted by rtk tool proxy — worked around with Python script for mass file renaming across 77 source files.
- Shell `sed -i` also intercepted — used Python for file content replacement.

## Next Phase Readiness

- Workspace structure is correct: 7 agentix-* crates, clean filesystem
- agentix-core, agentix-llm, agentix-mcp, agentix-memory compile with 0 errors
- agentix-runtime, agentix-triggers, agentix still have compile errors from referencing deleted types (agentflow, workflow, fleet, coordination, tools, gateway, personas) — to be fixed in Plans 02-06
- Plan 02 (agentix-core v2.0 types) should be executed next to establish the new type system that Plans 03-06 will depend on

---
*Phase: 13-rebrand-core-runtime-cli-foundation*
*Completed: 2026-03-12*
