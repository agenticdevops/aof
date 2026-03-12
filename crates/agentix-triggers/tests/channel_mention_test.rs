// TDD RED phase — written before ChannelMentionTrigger implementation.

use agentix_core::TriggerSource;
use agentix_triggers::{ChannelMentionTrigger, ChannelPlatform};
use std::collections::HashMap;

#[test]
fn test_slack_mention_event() {
    let trigger = ChannelMentionTrigger::for_slack(
        "dba-optimizer",
        "dba-optimizer-slack-0",
        "xoxb-test-token",
        "slack-signing-secret",
    );

    // Slack Events API: app_mention event callback
    let payload = serde_json::json!({
        "type": "event_callback",
        "event": {
            "type": "app_mention",
            "user": "U123ABC",
            "text": "<@U999BOT> analyze slow queries",
            "channel": "C456DEF",
            "ts": "1741234567.000001"
        }
    });

    let headers = HashMap::new();
    let result = trigger.process_event(payload, &headers);
    assert!(result.is_ok(), "app_mention should succeed: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "app_mention should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::Slack));
    assert_eq!(event.payload["channel"], "C456DEF");
    assert_eq!(event.payload["user"], "U123ABC");
    let text = event.payload["text"].as_str().unwrap_or("");
    assert!(text.contains("analyze slow queries"), "text should contain mention content");
}

#[test]
fn test_discord_mention_event() {
    let trigger = ChannelMentionTrigger::for_discord(
        "dba-optimizer",
        "dba-optimizer-discord-0",
        "discord-bot-token",
        "1234567890",
    );

    // Discord MESSAGE_CREATE event with bot mention
    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "987654321",
            "channel_id": "111222333",
            "guild_id": "444555666",
            "content": "<@777888999> run health check",
            "author": {
                "id": "U100200300",
                "username": "devuser"
            }
        }
    });

    let headers = HashMap::new();
    let result = trigger.process_event(payload, &headers);
    assert!(result.is_ok(), "Discord mention should succeed: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "Discord mention should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::Discord));
    assert_eq!(event.payload["channel_id"], "111222333");
}

#[test]
fn test_telegram_mention_event() {
    let trigger = ChannelMentionTrigger::for_telegram(
        "dba-optimizer",
        "dba-optimizer-telegram-0",
        "bot123456:ABCdef",
    );

    // Telegram webhook update with text message
    let payload = serde_json::json!({
        "update_id": 123456789,
        "message": {
            "message_id": 1,
            "from": {
                "id": 987654,
                "username": "devuser"
            },
            "chat": {
                "id": 111222,
                "type": "group"
            },
            "text": "@dba_optimizer check replication lag",
            "date": 1741234567
        }
    });

    let headers = HashMap::new();
    let result = trigger.process_event(payload, &headers);
    assert!(result.is_ok(), "Telegram message should succeed: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "Telegram message should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::Telegram));
    assert_eq!(event.payload["chat_id"], 111222);
}

#[test]
fn test_non_mention_is_filtered() {
    let trigger = ChannelMentionTrigger::for_slack(
        "dba-optimizer",
        "dba-optimizer-slack-0",
        "xoxb-test-token",
        "slack-signing-secret",
    );

    // Regular message event — NOT an app_mention
    let payload = serde_json::json!({
        "type": "event_callback",
        "event": {
            "type": "message",
            "user": "U123ABC",
            "text": "hello world without any mention",
            "channel": "C456DEF"
        }
    });

    let headers = HashMap::new();
    let result = trigger.process_event(payload, &headers);
    assert!(result.is_ok());
    // Regular message should be filtered (not a mention) → None
    assert!(result.unwrap().is_none(), "non-mention message should be filtered");
}

#[test]
fn test_channel_mention_trigger_id() {
    let trigger = ChannelMentionTrigger::for_telegram(
        "my-agent",
        "my-agent-telegram-0",
        "bot123:TOKEN",
    );
    assert_eq!(trigger.trigger_id_str(), "my-agent-telegram-0");
}
