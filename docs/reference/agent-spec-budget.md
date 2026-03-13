# Agent Spec: Budget Fields

The `budget` section of an agent spec controls spending limits enforced by the gateway.

## Field Reference

```yaml
spec:
  budget:
    daily_limit_usd: <float>         # Optional
    max_tokens_per_run: <integer>    # Optional
```

### `budget.daily_limit_usd`

| Property | Value |
|----------|-------|
| **Type** | `float` |
| **Default** | None (no daily limit) |
| **Minimum** | `0.0` |
| **Unit** | USD |

**Description**: Maximum amount this agent is allowed to spend in a single calendar day (UTC). The gateway checks the current day's recorded spend for this agent before starting each run. If `today_spend >= daily_limit_usd`, the run is blocked.

**Behavior when exceeded:**
- The run is rejected immediately — no LLM calls are made
- An error is returned: `Daily budget exceeded for '<name>': $X.XX spent, limit $X.XX`
- The run is not persisted to the run history

**When to use**: Protect against unexpected automation loops, high-frequency triggers, or accidental misconfiguration that could result in large bills.

---

### `budget.max_tokens_per_run`

| Property | Value |
|----------|-------|
| **Type** | `integer` (unsigned 64-bit) |
| **Default** | None (no token limit) |
| **Unit** | Tokens (input + output combined) |

**Description**: Maximum number of tokens (input + output summed across all LLM iterations) allowed in a single agent run. The ReAct loop tracks cumulative token usage after each LLM call. When the total exceeds this limit, the loop stops and reports the stop reason.

**Behavior when exceeded:**
- The current ReAct loop iteration completes
- The loop exits before the next LLM call
- `stopped_reason` in the run result is set to `TokenLimitExceeded { limit, used }`
- Partial output up to the stopping point is returned

**When to use**: Prevent runaway loops on complex tasks. A typical ReAct loop uses 200-1000 tokens per iteration; set this limit based on how many iterations are acceptable.

---

## Complete Example

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: budget-example
  description: "Agent with cost controls"
spec:
  model: google/gemini-2.0-flash
  system_prompt: |
    You are a helpful assistant. Answer concisely.

  budget:
    daily_limit_usd: 0.50
    max_tokens_per_run: 4000

  max_iterations: 5
```

## Checking Limits Programmatically

To verify the configured limits for a running agent:

```bash
agentix agents | grep budget-example
# Then inspect via REST
curl http://localhost:7777/api/v1/agents/budget-example | jq .budget
```

To see current daily spend:

```bash
agentix costs agent budget-example
```

## Notes

- Both fields are independent; you can set one without the other.
- `daily_limit_usd` resets at UTC midnight each day.
- `max_tokens_per_run` applies per individual run, not cumulatively.
- Budget enforcement uses `COALESCE(actual_cost_usd, cost_usd)` — actual provider-reported costs take precedence over estimates for the daily limit check.
- See `docs/guides/cost-budgets.md` for setup walkthrough and examples.
