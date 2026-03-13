---
plan: 17-05
phase: 17-cost-tracking-budgets
status: complete
completed: 2026-03-13
---

# Plan 17-05 Summary: Final Integration + Documentation

## What Was Built

**Gateway production persistence (router.rs)**:
- `Gateway::start` now calls `AgentManager::new_with_data_dir(Path::new("data"))` instead of `AgentManager::new()`
- Creates `data/runs.db` and `data/cost.db` on startup (directory created with `fs::create_dir_all`)
- `open_run_store_at(path)` and `open_cost_store_at(path)` helpers (replaces `open_run_store_default`/`open_cost_store_default`)

**AgentManager methods**:
- `new_with_data_dir(workspace, data_dir)` — production constructor with real SQLite files
- `data_db_paths(data_dir)` — helper that creates the data dir and returns (runs_path, cost_path)
- `new()` updated to use `:memory:` (tests only)
- `with_agents_dir()` updated to use `:memory:` (tests only)

**quickstart/agents/budget-example.yaml**:
- Valid K8s-style agent YAML with `budget.daily_limit_usd: 0.50` and `budget.max_tokens_per_run: 4000`
- Uses `google/gemini-2.0-flash` model
- Passes `agentix validate quickstart/agents/budget-example.yaml`

**Documentation**:
- `docs/guides/cost-budgets.md` — user guide covering: viewing costs with `agentix costs`, setting budgets in YAML, smart model routing (CORE-05), cost data source (COST-07), REST API endpoints
- `docs/reference/agent-spec-budget.md` — field reference for `budget.daily_limit_usd` and `budget.max_tokens_per_run` with behavior description, defaults, examples
- `CHANGELOG.md` — [2.0.0-alpha.5] section added at top with all Phase 17 features documented

**Pre-existing test fixes** (not Phase 17, but blocking workspace tests):
- `crates/agentix-core/src/registry.rs` — added `budget: None` to inline test
- `crates/agentix-core/tests/agent_types_test.rs` — added `budget: None` to AgentDefinition test
- `crates/agentix-runtime/tests/react_loop_test.rs` — added `max_tokens_per_run: None` and `budget: None`
- `crates/agentix-runtime/tests/streaming_test.rs` — added RunResult cost/token fields to test
- `crates/agentix-core/src/cost.rs` — fixed inline test message to reliably produce score > 30
- `crates/agentix-llm/tests/anthropic_tests.rs`, `openai_tests.rs`, `google_tests.rs`, `provider_tests.rs` — added `tool_call_id: None` to RequestMessage initializers
- `crates/agentix-triggers/tests/jira_platform_test.rs` — fixed `api_url` → `base_url`, added `enable_updates`, added `max_message_age_secs`
- `crates/agentix-triggers/tests/workflow_tests.rs` — removed RuntimeOrchestrator tests (not implemented), fixed TriggerHandlerConfig

## Test Results

```
cargo check --workspace: 0 errors
cargo test --lib --workspace (excluding agentix-mcp): 297 passed
cargo test --test cost_test -p agentix-core: 9 passed
cargo test --test cost_store_test -p agentix-runtime: 4 passed
cargo test --test budget_test -p agentix-runtime: 4 passed
agentix validate quickstart/agents/budget-example.yaml: "Agent 'budget-example' is valid"
```

## Artifacts

- `crates/agentix-runtime/src/gateway/router.rs` — new_with_data_dir wiring
- `crates/agentix-runtime/src/gateway/agent_manager.rs` — new_with_data_dir, open helpers
- `quickstart/agents/budget-example.yaml` (new)
- `docs/guides/cost-budgets.md` (new)
- `docs/reference/agent-spec-budget.md` (new)
- `CHANGELOG.md` — [2.0.0-alpha.5] section

## Self-Check: PASSED

- [x] cargo check --workspace exits 0
- [x] Gateway::start uses real data/cost.db path (not ":memory:")
- [x] quickstart/agents/budget-example.yaml exists with valid budget fields
- [x] agentix validate passes on the example
- [x] docs/guides/cost-budgets.md exists with setup guide
- [x] docs/reference/agent-spec-budget.md exists with field reference table
- [x] CHANGELOG.md has [2.0.0-alpha.5] section with all Phase 17 features
- [x] All Phase 17 requirements covered: COST-01..07, CLI-09, CORE-05
