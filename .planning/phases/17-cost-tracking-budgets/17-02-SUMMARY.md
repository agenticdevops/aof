---
plan: 17-02
phase: 17-cost-tracking-budgets
status: complete
completed: 2026-03-13
---

# Plan 17-02 Summary: CostStore SQLite Persistence + AgentManager Wiring

## What Was Built

**agentix-runtime cost_store.rs (`crates/agentix-runtime/src/cost_store.rs`)**:
- `CostStore::open(path)` — opens SQLite with WAL mode, creates cost_records table + 3 indexes
- `insert_record(&CostRecord)` — INSERT OR IGNORE
- `get_run_summary(run_id)` — aggregates input/output tokens + cost (COALESCE actual/estimated) for one run
- `get_agent_summary(agent_name)` — aggregates across all runs for one agent (COUNT DISTINCT run_id)
- `list_run_summaries(agent_name, limit)` — paginated per-run breakdown sorted newest first
- `list_all_summaries()` — one CostSummary per distinct agent, sorted by total_cost DESC
- `get_today_spend(agent_name)` — daily budget check: SUM(COALESCE(actual,estimated)) WHERE date = 'now'

**agentix-runtime lib.rs**:
- `pub mod cost_store;` + `pub use cost_store::CostStore;` re-export

**AgentManager wiring**:
- `cost_store: Arc<CostStore>` field added
- `open_cost_store_default()` helper — opens `./agentix-cost.db`, falls back to `:memory:`
- Both `AgentManager::new()` and `with_agents_dir()` initialize cost_store
- After each successful run, if tokens > 0: create CostRecord from RunResult token counts, look up pricing via `default_model_pricing()`, insert via `cost_store.insert_record()`

**react_loop.rs RunResult updates**:
- `total_input_tokens: u64` — accumulated across all LLM calls in run
- `total_output_tokens: u64` — accumulated across all LLM calls in run
- `total_cost_usd: f64` — computed by AgentManager using pricing table
- `stopped_reason: Option<BudgetStopReason>` — set when budget limit triggers stop

**Tests** (`crates/agentix-runtime/tests/cost_store_test.rs`):
- 4 tests: insert + get_run_summary (token sum check), get_agent_summary (total_runs=2), list_all_summaries (3 agents), actual_cost_usd precedence (COALESCE)

## Test Results

```
cargo test --test cost_store_test -p agentix-runtime: 4 passed
cargo check --workspace: 0 errors
```

## Artifacts

- `crates/agentix-runtime/src/cost_store.rs`
- `crates/agentix-runtime/src/lib.rs` — cost_store module + CostStore export
- `crates/agentix-runtime/src/gateway/agent_manager.rs` — cost_store field + recording
- `crates/agentix-runtime/src/executor/react_loop.rs` — RunResult token fields
- `crates/agentix-runtime/tests/cost_store_test.rs`

## Self-Check: PASSED

- [x] cargo test --test cost_store_test -p agentix-runtime: 4 tests pass
- [x] cargo check --workspace exits 0
- [x] CostStore with all 6 methods (open, insert_record, get_run_summary, get_agent_summary, list_run_summaries, list_all_summaries, get_today_spend)
- [x] AgentManager.cost_store field and recording after runs
- [x] COALESCE(actual_cost_usd, cost_usd) in all aggregation queries (COST-07)
- [x] RunResult has total_input_tokens, total_output_tokens, stopped_reason fields
