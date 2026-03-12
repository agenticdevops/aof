---
phase: 15-triggers-scheduling
plan: 06
status: complete
completed_at: 2026-03-13
---

# Plan 15-06 Summary: Run Persistence + Notification Routing

## What was built

### RunRecord and RunStore (SQLite-backed)

Created `crates/agentix-runtime/src/gateway/run_store.rs` with:
- `RunRecord` struct — all run metadata: id, agent_name, trigger_source, trigger_id, started_at, ended_at, duration_ms, status, iterations, input_summary, output_summary, error
- `RunStore` — SQLite-backed store using rusqlite 0.31 bundled
  - `open(path)` — creates/opens DB, enables WAL mode, creates schema
  - `insert()` — INSERT OR IGNORE (idempotent)
  - `update_completed()` — calculates duration_ms from started_at
  - `update_failed()` — records error message
  - `list(agent_filter, limit)` — sorted by started_at DESC
  - `get(run_id)` — single record lookup

### AgentManager integration

Updated `agent_manager.rs`:
- Added `run_store: Arc<RunStore>` field (opens `./agentix-runs.db` on start)
- `run_agent_with_trigger()` inserts RunRecord before execution, spawns task to update status on completion/failure
- `dispatch_notifications()` reads agent YAML `notifications:` field and dispatches `webhook` (HTTP POST) and `log` types

### Global runs API endpoint

Added to `api.rs`:
- `GET /api/v1/runs` — lists recent runs across all agents
- `GET /api/v1/runs?agent=<name>` — filters by agent
- `GET /api/v1/runs?limit=N` — adjusts limit (default 20)
- Response includes `trigger_source` field in each run object

### Enhanced agentix runs CLI

Updated `crates/agentix/src/commands/runs.rs`:
- Added TRIGGER column to text output table showing trigger_source

Updated `crates/agentix/src/client.rs`:
- `list_runs()` now calls global `/api/v1/runs` endpoint (previously fetched per-agent)
- Supports `?agent=<name>` and `?limit=N` query params

### Documentation

Created `docs/concepts/run-persistence.md` covering:
- RunRecord fields table
- SQLite storage location and WAL mode
- CLI usage (agentix runs + filters)
- REST API endpoint
- Notification routing config schema
- Retention policy (manual pruning via SQLite)

## TDD cycle

- RED: 5 tests written in `crates/agentix-runtime/tests/run_persistence_test.rs` before implementation
- GREEN: RunStore implemented; all 5 tests pass
- No refactoring needed

## Verification results

- `cargo test --test run_persistence_test -p agentix-runtime` — 5/5 passed
- `cargo check` — 0 errors workspace-wide
- `GET /api/v1/runs` route present in api.rs
- `TRIGGER` column in agentix runs text output
- `docs/concepts/run-persistence.md` created with all sections

## Files modified

- `crates/agentix-runtime/src/gateway/run_store.rs` (new)
- `crates/agentix-runtime/src/gateway/mod.rs` (export RunRecord, RunStore)
- `crates/agentix-runtime/src/gateway/agent_manager.rs` (RunStore field + persistence)
- `crates/agentix-runtime/src/gateway/api.rs` (GET /api/v1/runs endpoint)
- `crates/agentix-runtime/tests/run_persistence_test.rs` (new — 5 tests)
- `crates/agentix/src/commands/runs.rs` (TRIGGER column)
- `crates/agentix/src/client.rs` (global runs endpoint)
- `Cargo.toml` (rusqlite workspace dep)
- `crates/agentix-runtime/Cargo.toml` (rusqlite dep)
- `docs/concepts/run-persistence.md` (new)
