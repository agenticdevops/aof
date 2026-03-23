---
phase: 20-approval-workflows
plan: 04
subsystem: api
tags: [rust, axum, sqlite, approval-workflows, audit-trail, background-tasks]

# Dependency graph
requires:
  - phase: 20-01
    provides: ApprovalRequest, ApprovalStatus, ApprovalPolicy types
  - phase: 20-02
    provides: ApprovalStore SQLite persistence
  - phase: 20-03
    provides: ReAct loop approval gate (ApprovalRequested/ApprovalResolved events)

provides:
  - RunStatus::WaitingForApproval variant for run state tracking
  - Background 30-second expiration task in all AgentManager constructors
  - Per-run event watcher that transitions RunState.status on approval events
  - Audit trail (AuditEventType::ApprovalDecision) in approve/deny REST handlers

affects: [20-05, 22-command-center]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Per-run broadcast subscriber watcher task for real-time status transitions
    - tokio::time::interval for background periodic work with initial-tick skip

key-files:
  created: []
  modified:
    - crates/agentix-runtime/src/gateway/agent_manager.rs
    - crates/agentix-runtime/src/gateway/api.rs

key-decisions:
  - "Background expiry task skips first tick to avoid expiring requests immediately on gateway startup"
  - "Per-run watcher subscribes to broadcast channel rather than polling — avoids latency lag in status display"
  - "RunStatus serde renamed snake_case (was lowercase) to match waiting_for_approval serialization"
  - "Approve/deny audit logs actor as 'user:<approver>' for distinguishability from system events"

patterns-established:
  - "Approval status watcher: separate tokio::spawn subscribes to event_tx, updates RunState on ApprovalRequested/ApprovalResolved"
  - "Audit decision logging pattern: log to audit_store in REST handler immediately after store update"

requirements-completed: [APPR-02, APPR-03, APPR-04]

# Metrics
duration: 15min
completed: 2026-03-23
---

# Phase 20 Plan 04: AgentManager Wiring + REST API Approval Endpoints Summary

**RunStatus::WaitingForApproval added, background expiry task spawned every 30s, audit trail wired to approve/deny endpoints**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-23T05:20:00Z
- **Completed:** 2026-03-23T05:35:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `RunStatus::WaitingForApproval` variant so REST API and CLI can surface when a run is paused for human review
- Spawned background `tokio::time::interval` task in all three `AgentManager` constructors (`new`, `new_with_data_dir`, `with_agents_dir`) that calls `expire_stale()` every 30 seconds
- Per-run broadcast subscriber watcher transitions `RunState.status` to `WaitingForApproval` on `ApprovalRequested` events and back to `Running` on `ApprovalResolved` events
- `approve_request` and `deny_request` REST handlers now log `AuditEventType::ApprovalDecision` entries to the audit store, completing the APPR-04 audit trail requirement

## Task Commits

1. **Task 1: Add RunStatus::WaitingForApproval and approval expiry** - `7dd83b3` (feat)
2. **Task 2: Add AuditEntry logging to approve/deny endpoints** - `315e367` (feat)

## Files Created/Modified

- `crates/agentix-runtime/src/gateway/agent_manager.rs` — Added `RunStatus::WaitingForApproval`, `spawn_approval_expiry_task()`, and per-run approval event watcher
- `crates/agentix-runtime/src/gateway/api.rs` — Added audit logging to `approve_request` and `deny_request` handlers

## Decisions Made

- Background expiry task skips the first `interval.tick()` to avoid expiring requests that were just created during gateway startup. A 30-second window is required before the first sweep.
- Per-run status watcher uses `event_tx.subscribe()` (broadcast channel) rather than polling the approval store — this gives immediate status transitions when events fire.
- Changed `RunStatus` serde attribute from `lowercase` to `snake_case` to accommodate `waiting_for_approval` (cannot be represented in lowercase without underscore).
- Audit actor format is `user:<approver>` to distinguish from `system` events in the security audit query.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] RunStatus serde rename_all changed from lowercase to snake_case**
- **Found during:** Task 1 (adding WaitingForApproval variant)
- **Issue:** `#[serde(rename_all = "lowercase")]` cannot represent `waiting_for_approval` — it would serialize as `waitingforapproval`. The `snake_case` attribute is correct for multi-word variants.
- **Fix:** Changed `rename_all = "lowercase"` to `rename_all = "snake_case"` on `RunStatus` enum. Existing variants (running, completed, cancelled) are unaffected as they are single words.
- **Files modified:** crates/agentix-runtime/src/gateway/agent_manager.rs
- **Verification:** `cargo check --workspace` exits 0
- **Committed in:** 7dd83b3 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in serde attribute that would break new variant serialization)
**Impact on plan:** Necessary correctness fix. No scope creep.

## Issues Encountered

Most of Plan 20-04 was already implemented in prior commits (the REST handlers, routes, and ApprovalStore field all existed from plan preparation). The remaining work was the three items listed above.

## Next Phase Readiness

- `RunStatus::WaitingForApproval` is available for CLI `agentix run status` display (Plan 20-05)
- Audit trail complete: approve/deny decisions appear in `GET /api/v1/agents/:name/audit`
- Background expiry ensures stale requests don't accumulate indefinitely
- REST API fully functional: 4 endpoints (list, get, approve, deny) with 404/409 semantics

---
*Phase: 20-approval-workflows*
*Completed: 2026-03-23*
