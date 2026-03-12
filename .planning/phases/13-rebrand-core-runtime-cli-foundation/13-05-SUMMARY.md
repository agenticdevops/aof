---
phase: 13-rebrand-core-runtime-cli-foundation
plan: "05"
subsystem: runtime
tags: [rust, agentix-runtime, streaming, sse, react-loop, tdd, ndjson, broadcast-channel]

requires:
  - phase: 13-03
    provides: AgentManifest, AgentDefinition, DirectoryLoader, WorkspaceConfig
  - phase: 13-04
    provides: react_loop.rs with ReActEvent/ReActStep/RunResult/ToolAction types

provides:
  - SseEncoder (SSE wire format encoder for HTTP/WebSocket clients)
  - TextFormatter (human-readable CLI output with [Plan]/[Act]/[Observe]/[Reflect] labels)
  - JsonFormatter (single-line NDJSON for --json flag and CI/CD pipelines)
  - EventSender/EventReceiver type aliases (tokio broadcast channel for real-time event fan-out)
  - Serialize/Deserialize/PartialEq derives on ReActEvent, ReActStep, RunResult, ToolAction

affects: [13-06-gateway, 13-07-cli]

tech-stack:
  added: []
  patterns:
    - "SSE format: one message per non-empty phase (plan/act/observe/reflect) for granular client streaming"
    - "TextFormatter renders without ANSI codes when use_colors=false (test-safe and TTY-agnostic)"
    - "JsonFormatter produces NDJSON: compact single-line JSON with event_type + RFC3339 timestamp"
    - "EventSender/EventReceiver are type aliases over tokio::sync::broadcast — no wrapper overhead"
    - "ReActEvent types are canonical in react_loop.rs; streaming.rs re-exports them via pub use"

key-files:
  created:
    - crates/agentix-runtime/src/streaming.rs
    - crates/agentix-runtime/tests/streaming_test.rs
  modified:
    - crates/agentix-runtime/src/lib.rs (added pub mod streaming + re-exports)
    - crates/agentix-runtime/src/executor/react_loop.rs (Serialize/Deserialize/PartialEq derives, fixed MessageRole import)

key-decisions:
  - "ReActEvent types stay canonical in react_loop.rs — streaming.rs imports via pub use (avoids duplication)"
  - "SSE emits one message per phase within a Step, not one message per Step — enables granular client progress rendering"
  - "No colored crate dependency — ANSI escape codes embedded directly (avoids workspace dependency churn)"
  - "Auto-fixed Rule 1 bug: MessageRole import in react_loop.rs was agentix_core::MessageRole (agent.rs type) but RequestMessage.role expects agentix_core::model::MessageRole — corrected to use model:: path"

requirements-completed: [CORE-03]

duration: 9min
completed: 2026-03-12
---

# Phase 13 Plan 05: Streaming Output for ReAct Loop Summary

**SSE encoder, text formatter, and NDJSON formatter implemented for ReAct loop event streaming, with broadcast channel type aliases and 14 passing TDD tests**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-12T18:10:47Z
- **Completed:** 2026-03-12T18:19:07Z
- **Tasks:** 2 (TDD RED + GREEN/REFACTOR)
- **Files modified:** 4

## Accomplishments

- `SseEncoder::encode` converts `ReActEvent` to valid SSE wire format: one `event: react_step\ndata: ...\n\n` message per non-empty phase per iteration; separate `event: complete` and `event: error` messages
- `TextFormatter::format` renders labelled sections `[Plan]` / `[Act]` / `[Observe]` / `[Reflect]` with ANSI colour codes when `use_colors: true`; no escape codes when `use_colors: false` (verified by test)
- `JsonFormatter::format` produces compact single-line JSON (NDJSON) with `event_type` and RFC 3339 `timestamp` fields
- `EventSender` / `EventReceiver` type aliases over `tokio::sync::broadcast::Sender/Receiver<ReActEvent>` — zero-cost abstraction for fan-out streaming from the ReAct loop to multiple consumers
- All 14 TDD tests pass: 6 SSE format tests, 5 text format tests, 2 JSON format tests, 1 channel ordering test
- Added `Serialize`, `Deserialize`, `PartialEq` derives to `ReActEvent`, `ReActStep`, `RunResult`, `ToolAction` in `react_loop.rs` — required for JSON serialization and test equality assertions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed duplicate MessageRole type collision in react_loop.rs**
- **Found during:** Task 2 (GREEN phase) compilation
- **Issue:** `react_loop.rs` imported `MessageRole` via `use agentix_core::MessageRole` which resolves to `agent.rs::MessageRole`, but `RequestMessage.role` has type `model::MessageRole` — causing type mismatch errors at lines 193, 275, 281
- **Fix:** Changed import to `use agentix_core::model::MessageRole` (the correct path for the model-level type)
- **Files modified:** `crates/agentix-runtime/src/executor/react_loop.rs`
- **Commit:** d8734fb

**2. [Rule 2 - Missing functionality] Added Serialize/Deserialize/PartialEq to ReAct types**
- **Found during:** Task 1 (RED phase) test design
- **Issue:** `ReActEvent`, `ReActStep`, `RunResult`, `ToolAction` only had `Debug, Clone` — `serde_json` encoding required for streaming formatters; `PartialEq` required for channel ordering test assertions
- **Fix:** Added `Serialize, Deserialize, PartialEq` derives to all four types
- **Files modified:** `crates/agentix-runtime/src/executor/react_loop.rs`
- **Commit:** 5372593

## Self-Check: PASSED

- `crates/agentix-runtime/src/streaming.rs` — FOUND
- `crates/agentix-runtime/tests/streaming_test.rs` — FOUND
- Commit `5372593` (RED) — FOUND
- Commit `d8734fb` (GREEN) — FOUND
