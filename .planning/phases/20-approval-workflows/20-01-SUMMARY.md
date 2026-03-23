---
phase: 20-approval-workflows
plan: 01
subsystem: agentix-core/approval
tags: [approval, tdd, core-types, human-in-the-loop]
dependency_graph:
  requires: []
  provides: [ApprovalRequest, ApprovalStatus, ApprovalDecision, ApprovalPolicy]
  affects: [agentix-runtime/approval_store, agentix-runtime/react_loop, agentix-runtime/api]
tech_stack:
  added: [chrono, uuid (already deps)]
  patterns: [tagged-enum-variants, single-decision-semantics, TDD-red-green]
key_files:
  created:
    - crates/agentix-core/src/approval.rs
    - crates/agentix-core/tests/approval_test.rs
    - docs/concepts/approval-workflows.md
  modified:
    - crates/agentix-core/src/lib.rs
    - crates/agentix-runtime/src/approval_store.rs
    - crates/agentix-runtime/src/executor/react_loop.rs
    - crates/agentix-runtime/src/gateway/api.rs
    - crates/agentix-runtime/tests/approval_store_test.rs
decisions:
  - ApprovalStatus uses tagged enum variants (Approved/Denied carry approver+timestamp, TimedOut carries expired_at) for type-level single-decision semantics
  - ApprovalPolicy uses AgentMode (from agent.rs) not a new AutonomyMode enum — avoids duplicate enum with same semantics
  - ApprovalRequest::new() takes (run_id, agent_name, action_description, tool_name, tool_input, timeout_secs) — UUID generated internally
  - SemiAutonomous checks flagged_tools first (exact match), then flagged_patterns (case-insensitive substring on JSON-serialized input)
  - is_expired() uses >= (not >) so timeout_secs=0 expires immediately without sleep
  - expire_stale() uses Unix epoch arithmetic in SQLite (strftime('%s',...)) to avoid RFC3339 nanosecond parsing issues
metrics:
  duration_minutes: 18
  tasks_completed: 3
  files_created: 3
  files_modified: 5
  tests_written: 18
  tests_passing: 18
  completed_date: "2026-03-23"
---

# Phase 20 Plan 01: Approval Core Types Summary

Approval workflow core types (TDD) — `ApprovalRequest`, `ApprovalStatus`, `ApprovalDecision`, `ApprovalPolicy` with rich tagged enum variants, single-decision semantics, and `AgentMode` integration.

## What Was Built

`agentix_core::approval` module implementing the foundational types for Phase 20 approval workflows:

- **`ApprovalStatus`** — tagged enum with data variants: `Pending`, `Approved { approver, decided_at }`, `Denied { approver, reason, decided_at }`, `TimedOut { expired_at }`
- **`ApprovalRequest`** — request struct with UUID generation, `approve()/deny()` methods enforcing single-decision semantics, `is_expired()` using `>=` comparison
- **`ApprovalDecision`** — standalone decision record for audit trail use (with `request_id`, `approved`, `approver`, `reason`, `decided_at`)
- **`ApprovalPolicy`** — policy using `AgentMode` with `flagged_tools`, `flagged_patterns` (case-insensitive substring), `default_timeout_secs`, `approvers`

## Verification

- `cargo test --test approval_test -p agentix-core`: **18/18 tests passing**
- `cargo check --workspace`: **0 errors** (warnings only, pre-existing)
- `docs/concepts/approval-workflows.md`: 7 sections created

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Existing approval.rs had different API — full rewrite required**
- **Found during:** Task 1 (test writing) — `approval.rs` existed as untracked file with `AutonomyMode`, flat `ApprovalStatus`, `apply_decision()` method
- **Issue:** Previous session had partially implemented approval with different type design — tests couldn't compile against it
- **Fix:** Rewrote `approval.rs` to match plan spec: tagged enum variants, `AgentMode` instead of `AutonomyMode`, `approve()/deny()` methods
- **Files modified:** `crates/agentix-core/src/approval.rs`, `crates/agentix-core/src/lib.rs`
- **Commit:** 1bc1516

**2. [Rule 3 - Blocking] approval_store.rs used old API — required migration**
- **Found during:** Task 2 (workspace check post-implementation)
- **Issue:** `approval_store.rs` imported `ApprovalAction`, used flat `ApprovalStatus` variants, old field names (`description`, `created_at`, `expires_at`)
- **Fix:** Rewrote `approval_store.rs` with new schema (action_description, requested_at, timeout_secs), SQLite epoch arithmetic for expire_stale
- **Files modified:** `crates/agentix-runtime/src/approval_store.rs`
- **Commit:** 1bc1516

**3. [Rule 3 - Blocking] react_loop.rs used old ApprovalRequest::new() signature and requires_approval() API**
- **Found during:** Task 2 (workspace check)
- **Issue:** Old `ApprovalRequest::new()` took 7 args in wrong order; `requires_approval()` took `(tool_name, tool_desc)` not `(tool_name, tool_input: &Value)`, `policy.timeout_seconds` not `policy.default_timeout_secs`
- **Fix:** Updated all call sites in react_loop.rs to new API
- **Files modified:** `crates/agentix-runtime/src/executor/react_loop.rs`
- **Commit:** 1bc1516

**4. [Rule 3 - Blocking] approval_store_test.rs used stale types (ApprovalAction, update_status, etc.)**
- **Found during:** Task 2 (runtime test run)
- **Issue:** Pre-existing test file for Plan 20-02 used old `ApprovalAction` enum, `update_status()` method, `get_pending_by_run()` not in new API
- **Fix:** Rewrote test file to use new `ApprovalStore::approve()/deny()` API matching new status variants
- **Files modified:** `crates/agentix-runtime/tests/approval_store_test.rs`
- **Commit:** 1bc1516

**5. [Rule 1 - Bug] is_expired() used > instead of >= causing 0-timeout test failure**
- **Found during:** Task 2 (GREEN test run, 1 failure)
- **Issue:** `Utc::now() > deadline` where `deadline = requested_at + 0s = requested_at` — identical instants not caught
- **Fix:** Changed to `>=`
- **Files modified:** `crates/agentix-core/src/approval.rs`
- **Commit:** 1bc1516

**6. [Rule 1 - Bug] SQLite expire_stale() had nanosecond RFC3339 parsing issue**
- **Found during:** Task 2 (approval_store_test run, 2 failures)
- **Issue:** `datetime(requested_at, '+N seconds')` failed silently when `requested_at` contained nanoseconds (`2026-03-23T04:50:00.123456789Z`)
- **Fix:** Switched to `CAST(strftime('%s', requested_at) AS INTEGER) + timeout_secs` epoch arithmetic
- **Files modified:** `crates/agentix-runtime/src/approval_store.rs`
- **Commit:** 1bc1516

## Self-Check: PASSED
