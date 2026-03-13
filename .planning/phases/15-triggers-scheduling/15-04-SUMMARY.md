---
plan: 15-04
phase: 15-triggers-scheduling
status: complete
completed: 2026-03-13
---

# Summary: 15-04 ChannelMentionTrigger (Slack/Discord/Telegram)

## What Was Built

- `ChannelMentionTrigger` in `agentix-triggers/src/channel_mention.rs` — unified passive trigger with `ChannelPlatform` enum
  - Slack: fires on `app_mention` event type; filters `url_verification` and non-mention messages
  - Discord: fires on `MESSAGE_CREATE` events when content contains `<@` mention pattern
  - Telegram: fires on any incoming `message.text`
- `ChannelMentionTriggerImpl` in `agentix-runtime/src/gateway/agent_manager.rs` — mirrors the logic avoiding circular dep
- `register_agent_triggers_sync()` updated to handle `slack`/`discord`/`telegram` trigger types
- `docs/guides/slack-mention-trigger.md` — Slack app setup, Events API, YAML config, payload
- `docs/guides/discord-mention-trigger.md` — Discord bot setup, MESSAGE_CREATE, YAML config
- `docs/guides/telegram-mention-trigger.md` — BotFather setup, setWebhook, YAML config

## Self-Check: PASSED

- [x] `cargo test --test channel_mention_test -p agentix-triggers` — 5 tests pass
- [x] `cargo check` — 0 errors
- [x] Slack app_mention → TriggerEvent{source: Slack}
- [x] Discord MESSAGE_CREATE with `<@` → TriggerEvent{source: Discord}
- [x] Telegram message → TriggerEvent{source: Telegram}
- [x] Non-mention Slack messages filtered (Ok(None))
- [x] TDD cycle: RED → GREEN → clean

## Key Decisions

- Standalone lightweight JSON parsing rather than wrapping v1.0 `SlackPlatform`/`DiscordPlatform`/`TelegramPlatform` (which use the incompatible `TriggerPlatform` async trait)
- Telegram fires on all messages (not just @mentions) since bots naturally only receive relevant messages via Telegram's privacy mode
- Same `dispatch_webhook_payload()` gateway route handles all four trigger families (webhook/github/jira + slack/discord/telegram)

## Commit

`379e2bd` feat(15-04): implement ChannelMentionTrigger for Slack, Discord, Telegram
