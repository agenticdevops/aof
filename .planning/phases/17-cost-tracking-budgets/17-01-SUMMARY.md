---
plan: 17-01
phase: 17-cost-tracking-budgets
status: complete
completed: 2026-03-13
---

# Plan 17-01 Summary: Cost Types + Pricing Table + TDD

## What Was Built

**agentix-core cost module (`crates/agentix-core/src/cost.rs`)**:
- `CostRecord` — per-LLM-call record with id, agent_name, run_id, model, provider, input_tokens, output_tokens, cost_usd, actual_cost_usd (Option, COST-07 hook), recorded_at
- `CostSummary` — per-agent aggregate: total_runs, total_input_tokens, total_output_tokens, total_cost_usd, last_run_at
- `RunCostSummary` — per-run aggregate: run_id, agent_name, model, input_tokens, output_tokens, cost_usd, started_at
- `ModelPricing` — pricing table entry: provider, model, input_cost_per_1k, output_cost_per_1k
- `calculate_cost(input_tokens, output_tokens, &ModelPricing) -> f64` — estimates USD cost from token counts
- `default_model_pricing() -> HashMap<String, ModelPricing>` — 9 entries covering Anthropic, OpenAI, Google, Groq
- `BudgetStopReason` enum — DailyLimitExceeded + TokenLimitExceeded (tagged serde)
- `ModelComplexityScore` struct + `score(&str) -> u8` + `tier(u8) -> ModelTier` — CORE-05 routing algorithm
- `ModelTier` enum — Flash (0-30) / Standard (31-70) / Pro (71-100)

**agentix-core lib.rs**:
- `pub mod cost;` added
- All types re-exported: CostRecord, CostSummary, RunCostSummary, ModelPricing, calculate_cost, default_model_pricing, BudgetStopReason, ModelTier, ModelComplexityScore

**Tests** (`crates/agentix-core/tests/cost_test.rs`):
- 9 tests: CostRecord construction + serde, actual_cost_usd field, CostSummary, RunCostSummary, ModelPricing (3 providers), calculate_cost (2 models), zero-token case, default_model_pricing table

**Documentation** (`docs/concepts/cost-tracking.md`):
- Overview, data model (all 3 types with JSON examples), actual vs estimated cost explanation, pricing table, CLI preview, REST API endpoints

## Test Results

```
cargo test --test cost_test -p agentix-core: 9 passed
cargo check --workspace: 0 errors
```

## Artifacts

- `crates/agentix-core/src/cost.rs`
- `crates/agentix-core/src/lib.rs` — cost module + exports added
- `crates/agentix-core/tests/cost_test.rs`
- `docs/concepts/cost-tracking.md`

## Self-Check: PASSED

- [x] cargo test --test cost_test -p agentix-core: 9 tests pass
- [x] cargo check --workspace exits 0 (no errors)
- [x] CostRecord, CostSummary, RunCostSummary exported from agentix-core
- [x] calculate_cost produces correct USD for 1000 input + 500 output with claude-sonnet-4-6 (0.0105)
- [x] actual_cost_usd field on CostRecord (COST-07 hook)
- [x] default_model_pricing() returns 9 model entries
- [x] docs/concepts/cost-tracking.md exists with all 5 sections
