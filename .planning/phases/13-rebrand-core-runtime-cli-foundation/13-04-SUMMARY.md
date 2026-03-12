---
phase: 13-rebrand-core-runtime-cli-foundation
plan: 04
subsystem: runtime
tags: [react-loop, tdd, agentix-runtime, plan-act-observe-reflect, tool-dispatch]

# Dependency graph
requires:
  - phase: 13-rebrand-core-runtime-cli-foundation
    plan: 03
    provides: AgentDefinition, DirectoryLoader, ToolEntry, resolved_system_prompt()
provides:
  - ReActEngine: async run(AgentDefinition, input) -> RunResult
  - ToolExecutor trait: execute(ToolEntry, JSON) -> Result<String, String>
  - ReActEvent enum: Step(ReActStep) | Complete(RunResult) | Error(String)
  - RunResult: output, iterations, tool_calls, reached_max_iterations
  - ReActConfig: from_definition(AgentDefinition) -> config
  - ReActStep: plan, action, observation, reflection
affects:
  - 13-05-streaming
  - 13-06-cli
  - 13-07-gateway

# Tech tracking
tech-stack:
  added: [tokio::time::timeout, tokio::sync::broadcast]
  patterns:
    - TDD RED-GREEN-REFACTOR for async Rust loop engine
    - Tool failures as observations (never hard-fail)
    - AgentDefinition drives loop config (not raw config struct)
    - ToolExecutor trait decoupled from agentix-core ToolExecutor

key-files:
  created:
    - crates/agentix-runtime/src/executor/react_loop.rs
    - crates/agentix-runtime/tests/react_loop_test.rs
  modified:
    - crates/agentix-runtime/src/executor/mod.rs
    - crates/agentix-runtime/src/lib.rs

key-decisions:
  - "iterations counts tool-call iterations only; pure Q&A (no tools) counts as 1"
  - "ToolExecutor trait in react_loop.rs is separate from agentix-core ToolExecutor — simpler API for loop dispatch"
  - "MessageRole must use agentix_core::model::MessageRole (not re-exported alias) to match RequestMessage.role field type"
  - "Tool failures feed back as observations via format 'Tool X failed: error' — loop continues"
  - "ReAct instructions prepended to resolved_system_prompt() as suffix separator"

patterns-established:
  - "ReAct loop: LLM → tool call? → dispatch → observe → repeat until no tool call"
  - "Event emission via broadcast::Sender — silently dropped if no receivers"
  - "dispatch_tool returns String always — error text formatted as observation"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 10min
completed: 2026-03-12
---

# Phase 13 Plan 04: ReAct Loop Engine Summary

**ReActEngine executing plan-act-observe-reflect cycles via AgentDefinition.resolved_system_prompt(), with tool failure feed-back and broadcast event streaming**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-12T18:10:43Z
- **Completed:** 2026-03-12T18:20:49Z
- **Tasks:** 2 (RED + GREEN/REFACTOR)
- **Files modified:** 4

## Accomplishments

- Removed all v1.0 executor files (agent_executor, agentflow_executor, runtime, workflow_executor, locking, sandbox, risk_policy, incident_triage) and their 8 integration test files
- Implemented `ReActEngine::run()` driving plan-act-observe-reflect cycles with `AgentDefinition.resolved_system_prompt()` as system prompt
- Tool calls dispatched by name from `definition.tools`; failures return observation text, loop never hard-fails
- Events emitted via `broadcast::Sender<ReActEvent>` — Step per iteration, Complete on finish
- All 9 TDD tests pass: single/multi iterations, max_iterations, no-tool Q&A, tool failure feed-back, system prompt assembly, tool dispatch, event emission

## Task Commits

1. **Task 1 (RED): Write failing tests** - `8fcf3d7` (test)
2. **Task 2 (GREEN): Implement ReActEngine** - `d8734fb` (feat — combined with 13-05 streaming work by automated session)

## Files Created/Modified

- `crates/agentix-runtime/src/executor/react_loop.rs` — ReActEngine, ToolExecutor trait, ReActConfig, ReActStep, RunResult, ReActEvent
- `crates/agentix-runtime/tests/react_loop_test.rs` — 9 TDD tests with MockModel and MockToolExecutor
- `crates/agentix-runtime/src/executor/mod.rs` — exports react_loop only (v1.0 exports removed)
- `crates/agentix-runtime/src/lib.rs` — re-exports react_loop types (v1.0 executor exports removed)

## Decisions Made

- `iterations` counts tool-call iterations only; a pure Q&A response (no tools) counts as 1 — matches test semantics
- `ToolExecutor` trait in react_loop.rs uses simpler `execute(ToolEntry, Value) -> Result<String, String>` vs agentix-core's heavier trait
- `agentix_core::model::MessageRole` must be imported directly (not via `agentix_core::MessageRole`) to match `RequestMessage.role` field type — there are two distinct `MessageRole` types in agentix-core
- Tool failure observation format: `"Tool 'X' failed: error"` — predictable string the LLM can reason about

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- **MessageRole type ambiguity**: `agentix_core::MessageRole` (agent.rs) and `agentix_core::model::MessageRole` (model.rs) are distinct types despite same name. `RequestMessage.role` requires `model::MessageRole`. Fixed by using `agentix_core::model::MessageRole` directly.

## Next Phase Readiness

- ReActEngine is complete and tested — ready for gateway integration (13-07) and CLI wiring (13-06)
- ToolExecutor trait ready for concrete implementations (CLI executor, MCP executor)
- Event stream ready for SSE/WebSocket forwarding in API layer

## Self-Check: PASSED

- react_loop.rs: FOUND
- react_loop_test.rs: FOUND
- 13-04-SUMMARY.md: FOUND
- RED commit 8fcf3d7: FOUND
- GREEN commit d8734fb: FOUND
- 9/9 tests passing: VERIFIED

---
*Phase: 13-rebrand-core-runtime-cli-foundation*
*Completed: 2026-03-12*
