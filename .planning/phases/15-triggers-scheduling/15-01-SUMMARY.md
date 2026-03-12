---
plan: 15-01
phase: 15-triggers-scheduling
status: complete
completed: 2026-03-13
---

# Summary: 15-01 Core Trigger Abstraction

## What Was Built

Added the runtime trigger abstraction layer to `agentix-core`:

- `TriggerEvent` — envelope type delivered to every triggered agent run (source, payload, context, fired_at, trigger_id)
- `TriggerSource` — 9-variant enum (Cron, Webhook, GitHub, Jira, Slack, Discord, Telegram, Agent, Cli) with `Display` impl
- `TriggerTrait` — async_trait interface with `trigger_id()`, `source()`, `start(sender)`, `stop()` methods
- `TriggerRunRegistry` — runtime registry holding `Arc<dyn TriggerTrait>` instances keyed by trigger_id

All types exported from `agentix_core::lib`.

## Self-Check: PASSED

- [x] `cargo test --test trigger_event_test -p agentix-core` — 5 tests pass
- [x] `cargo clippy -p agentix-core -- -D warnings` — no issues
- [x] `cargo check` — 0 errors across whole workspace
- [x] TDD cycle: RED (compile errors from tests written before impl) → GREEN (5 tests pass) → REFACTOR (clippy clean)
- [x] `docs/concepts/triggers.md` exists with all 7 sections

## Key Decisions

- `TriggerRunRegistry` is separate from the YAML-loading `TriggerRegistry` in `registry.rs` — the new type manages live Arc<dyn TriggerTrait> instances; the existing type manages YAML config resources
- Added `tokio = { workspace = true, features = ["sync"] }` as a regular dep to agentix-core (previously dev-only) — required for `mpsc::Sender<(String, TriggerEvent)>` in TriggerTrait
- Fixed pre-existing clippy warnings in `context.rs` and `registry.rs` (unnecessary_map_or)

## Key Files Created

- `crates/agentix-core/src/trigger_event.rs` — full implementation
- `crates/agentix-core/tests/trigger_event_test.rs` — 5 TDD tests
- `docs/concepts/triggers.md` — 7-section concepts doc

## Commit

`e51f60c` feat(15-01): add TriggerEvent, TriggerSource, TriggerTrait, TriggerRunRegistry to agentix-core
