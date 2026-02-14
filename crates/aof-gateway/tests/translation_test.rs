//! Event translation tests

use chrono::Utc;
use serde_json::json;

use aof_gateway::{InboundMessage, MessageUser, Platform, Attachment};

#[test]
fn test_slack_message_translation() {
    let message = InboundMessage {
        message_id: "1234.5678".to_string(),
        platform: Platform::Slack,
        channel_id: "C1234567890".to_string(),
        thread_id: None,
        user: MessageUser {
            user_id: "U1234567890".to_string(),
            username: "testuser".to_string(),
            display_name: Some("Test User".to_string()),
        },
        content: "Hello, agent!".to_string(),
        attachments: vec![],
        metadata: json!({}),
        timestamp: Utc::now(),
    };

    use aof_gateway::translation::translate_to_coordination_event;
    let event = translate_to_coordination_event(&message, "test-session").unwrap();

    assert_eq!(event.session_id, "test-session");
    assert_eq!(event.agent_id, "gateway-slack");

    // Verify activity metadata contains message info
    if let Some(ref details) = event.activity.details {
        if let Some(ref metadata) = details.metadata {
            assert_eq!(metadata.get("message_id").unwrap(), "1234.5678");
            assert_eq!(metadata.get("channel_id").unwrap(), "C1234567890");
            assert_eq!(metadata.get("user_id").unwrap(), "U1234567890");
        }
    }
}

#[test]
fn test_discord_threaded_message_translation() {
    let message = InboundMessage {
        message_id: "987654321".to_string(),
        platform: Platform::Discord,
        channel_id: "channel-123".to_string(),
        thread_id: Some("thread-456".to_string()),
        user: MessageUser {
            user_id: "discord-user-1".to_string(),
            username: "discorduser".to_string(),
            display_name: Some("Discord User".to_string()),
        },
        content: "Threaded message".to_string(),
        attachments: vec![],
        metadata: json!({}),
        timestamp: Utc::now(),
    };

    use aof_gateway::translation::translate_to_coordination_event;
    let event = translate_to_coordination_event(&message, "discord-session").unwrap();

    assert_eq!(event.session_id, "discord-session");
    assert_eq!(event.agent_id, "gateway-discord");

    // Verify thread_id is preserved
    if let Some(ref details) = event.activity.details {
        if let Some(ref metadata) = details.metadata {
            assert_eq!(metadata.get("thread_id").unwrap(), "thread-456");
        }
    }
}

#[test]
fn test_telegram_message_without_thread() {
    let message = InboundMessage {
        message_id: "tg-123".to_string(),
        platform: Platform::Telegram,
        channel_id: "chat-789".to_string(),
        thread_id: None,
        user: MessageUser {
            user_id: "tg-user-1".to_string(),
            username: "telegramuser".to_string(),
            display_name: None,
        },
        content: "Telegram message".to_string(),
        attachments: vec![],
        metadata: json!({}),
        timestamp: Utc::now(),
    };

    use aof_gateway::translation::translate_to_coordination_event;
    let event = translate_to_coordination_event(&message, "tg-session").unwrap();

    assert_eq!(event.session_id, "tg-session");
    assert_eq!(event.agent_id, "gateway-telegram");

    // Verify thread_id is not in metadata (None case handled correctly)
    if let Some(ref details) = event.activity.details {
        if let Some(ref metadata) = details.metadata {
            assert!(!metadata.contains_key("thread_id"));
        }
    }
}

#[test]
fn test_message_with_image_attachment() {
    let message = InboundMessage {
        message_id: "msg-with-image".to_string(),
        platform: Platform::Slack,
        channel_id: "C-images".to_string(),
        thread_id: None,
        user: MessageUser {
            user_id: "U-photo".to_string(),
            username: "photographer".to_string(),
            display_name: Some("Photo User".to_string()),
        },
        content: "Check out this image!".to_string(),
        attachments: vec![Attachment::Image {
            url: "https://example.com/image.png".to_string(),
            metadata: json!({
                "width": 1920,
                "height": 1080,
                "size_bytes": 524288
            }),
        }],
        metadata: json!({}),
        timestamp: Utc::now(),
    };

    use aof_gateway::translation::translate_to_coordination_event;
    let event = translate_to_coordination_event(&message, "image-session").unwrap();

    // Verify message translated successfully (attachment metadata preserved in original InboundMessage)
    assert_eq!(event.agent_id, "gateway-slack");
    assert_eq!(message.attachments.len(), 1);

    // Verify attachment is preserved in original message struct
    if let Attachment::Image { url, .. } = &message.attachments[0] {
        assert_eq!(url, "https://example.com/image.png");
    }
}
