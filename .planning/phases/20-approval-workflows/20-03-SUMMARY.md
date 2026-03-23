---
phase: 20-approval-workflows
plan: "03"
subsystem: agentix-runtime/executor
tags: [approval, react-loop, tdd, events, audit]
dependency_graph:
  requires: [20-01, 20-02]
  provides: [approval-gate-in-react-loop, approval-events]
  affects: [agentix-runtime/streaming, agentix-runtime/audit_store]
tech_stack:
  added: []
  patterns: [pause-poll-resume, broadcast-events, tdd-red-green]
key_files:
  created:
    - crates/agentix-runtime/tests/approval_gate_test.rs
  modified:
    - crates/agentix-runtime/src/executor/react_loop.rs
    - crates/agentix-runtime/src/streaming.rs
    - crates/agentix-runtime/src/audit_store.rs
decisions:
  - "ApprovalRequested/ApprovalResolved replace ApprovalWaiting/ApprovalDecided — richer variant fields carry ApprovalStatus enum for type-safe status propagation"
  - "ApprovalResolved.status is ApprovalStatus (not bool approved) — clients get full type-safe status with approver/reason/timestamp embedded"
  - "AuditStore::list_events() added as alias for get_agent_audit() for test-friendly API"
metrics:
  duration_secs: 453
  completed_date: "2026-03-23"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 3
  tests_added: 8
  tests_total_after: 228
---

# Phase 20 Plan 03: ReAct Loop Approval Gate Integration Summary

**One-liner:** Approval gate wired into ReAct loop — pause-poll-resume with ApprovalRequested/ApprovalResolved events, three-mode behavior (autonomous zero-overhead, semi-auto flagged tools, manual all tools), and audit trail logging.

## What Was Built

The ReAct loop now gates tool calls behind human approval when configured. Three-mode behavior:
- **Autonomous** — zero overhead, no store queries, no events emitted
- **Manual** — every tool call creates an `ApprovalRequest`, emits `ApprovalRequested`, polls store until decided
- **SemiAutonomous** — only flagged tools (by name or input pattern) trigger approval; unflagged tools run freely

The approval gate uses a `wait_for_approval` method that:
1. Creates an `ApprovalRequest` in `ApprovalStore` (SQLite-backed)
2. Emits `ReActEvent::ApprovalRequested` with full metadata
3. Polls store every 1 second for status change from Pending
4. On `Approved` → emits `ApprovalResolved{status: Approved{..}}`, logs audit, continues to tool execution
5. On `Denied` → emits `ApprovalResolved{status: Denied{..}}`, logs audit (Denied outcome), skips tool, continues loop
6. On `TimedOut` (deadline reached) → emits `ApprovalResolved{status: TimedOut{..}}`, skips tool, continues loop

## Event Variants

The `ReActEvent` enum now has richer approval variants replacing the old `ApprovalWaiting`/`ApprovalDecided`:

```rust
ApprovalRequested {
    request_id: String,
    run_id: String,
    agent_name: String,
    action_description: String,
    tool_name: Option<String>,
},
ApprovalResolved {
    request_id: String,
    status: agentix_core::ApprovalStatus,  // full enum: Approved/Denied/TimedOut
},
```

## Tests (8 added — all green)

| Test | Scenario | Verified |
|------|----------|---------|
| `autonomous_mode_no_approval_check` | Autonomous — zero requests created | No ApprovalRequest in store |
| `manual_mode_creates_approval_request` | Manual + pre-approve in bg | Tool executes, request Approved |
| `semi_autonomous_flagged_tool_pauses` | SemiAuto — kubectl flagged, git free | kubectl approved, no git request |
| `denied_action_skips_tool_call` | Manual + deny in bg | kubectl NOT called, loop continues |
| `timed_out_action_skips_tool_call` | Manual + 2s timeout, no action | kubectl NOT called after expiry |
| `approval_events_emitted` | Manual + approve | ApprovalRequested + ApprovalResolved emitted |
| `approval_logged_in_audit_trail` | Manual + approve | AuditStore has ApprovalDecision/Success |
| `denial_logged_in_audit_trail` | Manual + deny | AuditStore has ApprovalDecision/Denied |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Rename] Updated existing ApprovalWaiting/ApprovalDecided → ApprovalRequested/ApprovalResolved**
- **Found during:** Task 2 (implementation)
- **Issue:** The react_loop.rs already had `ApprovalWaiting`/`ApprovalDecided` variants from prior stub work; plan requires `ApprovalRequested`/`ApprovalResolved` with richer fields
- **Fix:** Renamed variants, added `run_id`, `agent_name`, `action_description` to `ApprovalRequested`; changed `approved: bool` to `status: ApprovalStatus` on `ApprovalResolved`
- **Files modified:** react_loop.rs, streaming.rs (SseEncoder, TextFormatter, JsonFormatter all updated)
- **Commit:** 5ab91c8

**2. [Rule 2 - Missing API] Added AuditStore::list_events() alias**
- **Found during:** Task 1 (RED tests used list_events, store only had get_agent_audit)
- **Fix:** Added `list_events` as a delegation wrapper for `get_agent_audit` in audit_store.rs
- **Files modified:** audit_store.rs
- **Commit:** 5ab91c8

## Self-Check: PASSED

All files exist. Both commits verified (aa3310a, 5ab91c8). 8 tests pass. 228 total tests pass.
