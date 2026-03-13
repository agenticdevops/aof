---
plan: 17-03
phase: 17-cost-tracking-budgets
status: complete
completed: 2026-03-13
---

# Plan 17-03 Summary: Budget Enforcement + Smart Model Routing

## What Was Built

**BudgetConfig in agentix-core/src/cost.rs**:
- `BudgetConfig` struct: `daily_limit_usd: Option<f64>`, `max_tokens_per_run: Option<u64>`

**AgentConfig in agentix-core/src/agent.rs**:
- `budget: Option<BudgetConfig>` field added to AgentConfig
- Budget field added to: FlatAgentConfig (input), AgentSpec (K8s input), FlatAgentSpecBody (FlatYamlLoader), From conversion for both paths
- AgentDefinition: `budget: Option<BudgetConfig>` field added, propagated in DirectoryLoader and FlatYamlLoader

**ReActConfig in react_loop.rs**:
- `max_tokens_per_run: Option<u64>` added
- `from_definition()` reads from `def.budget.as_ref().and_then(|b| b.max_tokens_per_run)`

**ReAct loop token budget enforcement (COST-05)**:
- `total_input_tokens` and `total_output_tokens` accumulators in loop
- After each LLM call: `response.usage.input_tokens` and `output_tokens` accumulated
- After accumulation: check if `total_input + total_output > max_tokens_per_run` → stop with `BudgetStopReason::TokenLimitExceeded`

**AgentManager daily budget check (COST-04)**:
- Before spawning run task: check `definition.budget.daily_limit_usd`
- If set: call `cost_store.get_today_spend(agent_name)`
- If today_spend >= daily_limit_usd: return error (run blocked)

**BudgetStopReason, ModelTier, ModelComplexityScore** (in agentix-core/cost.rs, via Plan 17-01):
- All exported from agentix-core lib.rs
- `BudgetConfig` added and also exported

**agentix-triggers fix**:
- `budget: None` added to AgentConfig initializer in handler/mod.rs

**Tests** (`crates/agentix-runtime/tests/budget_test.rs`):
- 4 tests: complexity_score_simple (Flash ≤30), complexity_score_complex (>30), model_tier_routing (direct boundary checks + 8-keyword message), budget_stop_reason_serde (JSON round-trip for both variants)

## Test Results

```
cargo test --test budget_test -p agentix-runtime: 4 passed
cargo check --workspace: 0 errors
```

## Artifacts

- `crates/agentix-core/src/cost.rs` — BudgetConfig added
- `crates/agentix-core/src/agent.rs` — budget field in AgentConfig + AgentDefinition
- `crates/agentix-core/src/lib.rs` — BudgetConfig exported
- `crates/agentix-runtime/src/executor/react_loop.rs` — token accumulation + budget stop
- `crates/agentix-runtime/src/gateway/agent_manager.rs` — daily budget check before run
- `crates/agentix-triggers/src/handler/mod.rs` — budget: None fix
- `crates/agentix-runtime/tests/budget_test.rs`

## Self-Check: PASSED

- [x] cargo test --test budget_test -p agentix-runtime: 4 tests pass
- [x] cargo check --workspace exits 0
- [x] BudgetConfig in AgentConfig with daily_limit_usd and max_tokens_per_run
- [x] ReAct loop stops with TokenLimitExceeded when max_tokens_per_run exceeded
- [x] AgentManager checks daily budget before run start (COST-04)
- [x] ModelComplexityScore::score() returns 0-100, tier() maps to Flash/Standard/Pro
- [x] BudgetStopReason serializes with correct tagged enums
