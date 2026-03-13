---
plan: 15-02
phase: 15-triggers-scheduling
status: complete
completed: 2026-03-13
---

# Summary: 15-02 CronTrigger Implementation

## What Was Built

- `CronTrigger` in `agentix-triggers/src/cron.rs` — validates 5-field cron expressions at construction, normalizes to 7-field format for the `cron` crate, spawns a tokio background task that fires `TriggerEvent` on schedule, graceful stop via oneshot channel
- `normalize_cron_expression()` helper for 5→7 field normalization
- `AgentTriggerConfig` and `AgentNotificationConfig` structs in `agentix-core/src/agent.rs` — `triggers:` and `notifications:` fields added to `AgentDefinition`, `AgentManifest`, `FlatAgentSpecBody`
- `AgentManager::run_agent_with_trigger()` — serializes TriggerEvent as JSON input and delegates to `start_run()`
- `TriggerRunRegistry` + trigger event dispatcher integrated into `AgentManager`
- `CronTriggerImpl` in `agentix-runtime` (avoids circular dep with agentix-triggers)
- `register_agent_triggers_sync()` and `start_all_triggers()` on `AgentManager`
- `docs/concepts/cron-triggers.md`

## Self-Check: PASSED

- [x] `cargo test --test cron_test -p agentix-triggers` — 4 tests pass
- [x] `cargo check` — 0 errors across whole workspace
- [x] CronTrigger::new() returns Err for invalid cron expressions
- [x] TriggerRunRegistry held by AgentManager, CronTriggers registered at load time
- [x] docs/concepts/cron-triggers.md exists with example YAML and 7 sections
- [x] TDD cycle: RED → GREEN → clean

## Key Decisions

- agentix-runtime cannot import agentix-triggers (circular dep), so `CronTriggerImpl` is duplicated in agent_manager.rs for gateway scheduling
- 5-field standard cron is supported via normalization to 7-field format the `cron` crate requires
- trigger registration is sync (at load time) but `start()` is async (at server boot via `start_all_triggers()`)

## Commit

`4d1f1c7` feat(15-02): implement CronTrigger, run_agent_with_trigger, gateway trigger wiring
