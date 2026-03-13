---
plan: 17-04
phase: 17-cost-tracking-budgets
status: complete
completed: 2026-03-13
---

# Plan 17-04 Summary: Cost REST API + CLI Interface

## What Was Built

**REST endpoints in agentix-runtime/src/gateway/api.rs**:
- `GET /api/v1/costs` — `list_all_costs` handler: returns JSON array of all agent cost summaries (sorted by total_cost_usd DESC via CostStore SQL ORDER BY)
- `GET /api/v1/costs/agents/:name` — `get_agent_costs` handler: returns single CostSummary or 404
- `GET /api/v1/costs/agents/:name/runs` — `get_agent_run_costs` handler: returns paginated Vec<RunCostSummary> with `?limit=N` (default 20)
- `CostRunsQuery` struct added for query param extraction
- Routes registered in `create_router()`

**AgentManager in agentix-runtime/src/gateway/agent_manager.rs**:
- `get_all_cost_summaries()` — delegates to cost_store.list_all_summaries()
- `get_agent_cost_summary(name)` — delegates to cost_store.get_agent_summary()
- `get_agent_run_cost_summaries(name, limit)` — delegates to cost_store.list_run_summaries()

**GatewayClient in crates/agentix/src/client.rs**:
- `list_all_costs()` — GET /api/v1/costs
- `get_agent_cost_summary(agent)` — GET /api/v1/costs/agents/:name
- `list_agent_run_costs(agent, limit)` — GET /api/v1/costs/agents/:name/runs?limit=N

**CLI in crates/agentix/src/cli.rs**:
- `Commands::Costs { action: Option<CostsAction>, limit: usize }` added
- `CostsAction::Agent { name: String, limit: usize }` enum

**CLI in crates/agentix/src/commands/costs.rs** (CREATED):
- `costs_all(ctx)` — prints summary table: AGENT | RUNS | INPUT TOKENS | OUTPUT TOKENS | COST (USD) | LAST RUN, plus total row
- `costs_agent(ctx, agent, limit)` — prints summary block + per-run table: RUN ID | MODEL | INPUT TOK | OUTPUT TOK | COST (USD)
- Both functions support `--output json` for machine-readable output

**Wired in main.rs**:
- `Commands::Costs { action, limit }` match arm dispatches to costs_all or costs_agent

**Documentation**:
- `docs/reference/cli-costs.md` — full CLI reference with output examples, flags, JSON output, cost sources, REST API equivalents, and budget configuration cross-reference

## Test Results

```
cargo check --workspace: 0 errors
```

## Artifacts

- `crates/agentix-runtime/src/gateway/api.rs` — 3 cost route handlers + CostRunsQuery struct
- `crates/agentix-runtime/src/gateway/agent_manager.rs` — 3 cost query methods
- `crates/agentix/src/client.rs` — 3 cost client methods
- `crates/agentix/src/cli.rs` — Costs command + CostsAction enum
- `crates/agentix/src/commands/costs.rs` (new)
- `crates/agentix/src/commands/mod.rs` — pub mod costs
- `crates/agentix/src/main.rs` — Costs dispatch arm
- `docs/reference/cli-costs.md` (new)

## Self-Check: PASSED

- [x] cargo check --workspace exits 0
- [x] GET /api/v1/costs returns JSON array of CostSummary
- [x] GET /api/v1/costs/agents/:name returns single CostSummary or 404
- [x] GET /api/v1/costs/agents/:name/runs returns paginated RunCostSummary array
- [x] `agentix costs` subcommand exists in cli.rs
- [x] `agentix costs agent <name>` subcommand exists
- [x] GatewayClient has list_all_costs, get_agent_cost_summary, list_agent_run_costs methods
- [x] docs/reference/cli-costs.md exists
