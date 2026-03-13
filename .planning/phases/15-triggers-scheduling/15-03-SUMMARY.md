---
plan: 15-03
phase: 15-triggers-scheduling
status: complete
completed: 2026-03-13
---

# Summary: 15-03 Webhook, GitHub, and Jira Triggers

## What Was Built

- `WebhookTrigger` in `agentix-triggers/src/webhook.rs` — `receive_payload()` with optional HMAC-SHA256 via `x-webhook-signature` header
- `GitHubTrigger` in `agentix-triggers/src/github_trigger.rs` — event-type filtering via `x-github-event` header, `sha256=` prefix signature via `x-hub-signature-256`
- `JiraTrigger` in `agentix-triggers/src/jira_trigger.rs` — event-type filtering via `body.webhookEvent`, HMAC via `x-hub-signature`
- `POST /webhooks/:trigger_id` gateway route — auto-detects source (webhook/github/jira from headers), dispatches TriggerEvent via trigger mpsc channel
- `AgentManager::dispatch_webhook_payload()` — looks up agent by trigger_id (format: `{agent_name}-{type}-{index}`), creates TriggerEvent, sends via channel
- `docs/concepts/webhook-triggers.md` — complete reference for all three trigger types

## Self-Check: PASSED

- [x] `cargo test --test webhook_test --test github_test --test jira_test --test cron_test -p agentix-triggers` — 15 tests pass
- [x] `cargo check` — 0 errors across whole workspace
- [x] WebhookTrigger: no-secret accepts all; secret validates HMAC-SHA256
- [x] GitHubTrigger: filters by event type; validates sha256= prefix signatures
- [x] JiraTrigger: filters by webhookEvent body field; validates HMAC signature
- [x] POST /webhooks/:trigger_id route returns 202 Accepted on success, 404 on unknown trigger_id
- [x] TDD cycle: RED → GREEN → clean

## Key Decisions

- HMAC is computed over `serde_json::to_vec(body)` (re-serialized JSON) since the HTTP handler parses the body into a `serde_json::Value` before passing to the trigger
- Tests compute HMAC over the same re-serialized bytes to maintain consistency
- Source auto-detection in the gateway route: `x-github-event` header → GitHub; `webhookEvent` body field → Jira; otherwise → Webhook
- The `dispatch_webhook_payload()` method on `AgentManager` (not on triggers) handles routing, since passive triggers in the registry don't hold the lookup logic

## Commit

`24b3b4d` feat(15-03): implement WebhookTrigger, GitHubTrigger, JiraTrigger + webhook gateway route
