---
plan: 15-05
phase: 15-triggers-scheduling
status: complete
completed: 2026-03-13
---

# Summary: 15-05 AgentTrigger, CliTrigger, agentix run

## What Was Built

- `AgentTrigger` in `agentix-triggers/src/agent_trigger.rs` — `create_event(payload, caller)` produces TriggerEvent{source: Agent} with context.caller_agent; passive TriggerTrait implementation
- `CliTrigger` in `agentix-triggers/src/cli_trigger.rs` — `create_event(input)` produces TriggerEvent{source: Cli, payload: {input: "..."}}; no-op TriggerTrait start()
- `POST /api/v1/agents/:name/trigger` endpoint in `api.rs` — accepts TriggerBody{payload, caller}, creates TriggerEvent{source: Agent}, calls run_agent_with_trigger(), returns 202 with run_id
- `agentix run <agent> --input "..."` command — CLI trigger via `commands/run.rs`, dispatches to trigger endpoint
- `GatewayClient::trigger_agent()` HTTP client method
- `docs/guides/agent-to-agent-triggers.md` — coordinator pattern, fire-and-forget model
- `docs/guides/cli-triggers.md` — usage, examples, exit codes, script integration

## Self-Check: PASSED

- [x] `cargo test --test agent_trigger_test --test cli_trigger_test -p agentix-triggers` — 4 tests pass
- [x] `cargo check` — 0 errors
- [x] AgentTrigger.create_event() sets source=Agent and context.caller_agent
- [x] CliTrigger.create_event() sets source=Cli, payload.input, context.invocation=cli
- [x] POST /api/v1/agents/:name/trigger returns 202 with run_id
- [x] `agentix run` wired in main.rs; compiles
- [x] TDD cycle: RED → GREEN → clean

## Key Decisions

- CliTrigger.start() is a no-op — CLI triggers fire synchronously via HTTP, not via background task
- `agentix run` is fire-and-forget in v2.0 — returns immediately with run_id; user tracks via `agentix runs`
- The trigger_id for HTTP-triggered runs is auto-generated as `{agent}-agent-trigger`

## Commit

`fdf2dd7` feat(15-05): implement AgentTrigger, CliTrigger, agentix run command
