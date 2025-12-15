//! Telegram platform tests

use aof_triggers::platforms::telegram::{TelegramPlatform, TelegramConfig};
use aof_triggers::platforms::TriggerPlatform;
use std::collections::HashMap;

fn test_config() -> TelegramConfig {
    TelegramConfig {
        bot_token: "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string(),
        webhook_url: Some("https://example.com/webhook/telegram".to_string()),
        webhook_secret: Some("test_secret".to_string()),
        bot_name: "aofbot".to_string(),
        allowed_users: None,
        allowed_groups: None,
    }
}

#[tokio::test]
async fn test_telegram_platform_creation() {
    let platform = TelegramPlatform::new(test_config());
    assert!(platform.is_ok());
}

#[tokio::test]
async fn test_telegram_bot_name() {
    let platform = TelegramPlatform::new(test_config()).unwrap();
    assert_eq!(platform.bot_name(), "aofbot");
}

#[tokio::test]
async fn test_telegram_platform_name() {
    let platform = TelegramPlatform::new(test_config()).unwrap();
    assert_eq!(platform.platform_name(), "telegram");
}

#[tokio::test]
async fn test_telegram_supports_interactive() {
    let platform = TelegramPlatform::new(test_config()).unwrap();
    assert!(platform.supports_interactive());
}

#[tokio::test]
async fn test_telegram_parse_text_message() {
    let payload = r#"{
        "update_id": 123456789,
        "message": {
            "message_id": 1001,
            "from": {
                "id": 987654321,
                "is_bot": false,
                "first_name": "John",
                "last_name": "Doe",
                "username": "johndoe"
            },
            "chat": {
                "id": 987654321,
                "type": "private"
            },
            "date": 1638360000,
            "text": "/run agent test-bot hello"
        }
    }"#;

    let platform = TelegramPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let message = platform.parse_message(payload.as_bytes(), &headers).await.unwrap();

    assert_eq!(message.platform, "telegram");
    assert_eq!(message.user.id, "987654321");
    assert_eq!(message.channel_id, "987654321");
    assert_eq!(message.text, "/run agent test-bot hello");
}

#[tokio::test]
async fn test_telegram_parse_missing_message() {
    let payload = r#"{"update_id": 123456789}"#;
    let platform = TelegramPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_telegram_parse_invalid_json() {
    let payload = "invalid json";
    let platform = TelegramPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_telegram_parse_group_message() {
    let payload = r#"{
        "update_id": 123456789,
        "message": {
            "message_id": 1001,
            "from": {
                "id": 987654321,
                "is_bot": false,
                "first_name": "John",
                "username": "johndoe"
            },
            "chat": {
                "id": -1001234567890,
                "type": "supergroup",
                "title": "Test Group"
            },
            "date": 1638360000,
            "text": "/run agent test-bot"
        }
    }"#;

    let platform = TelegramPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let message = platform.parse_message(payload.as_bytes(), &headers).await.unwrap();

    assert_eq!(message.channel_id, "-1001234567890");
    assert_eq!(message.text, "/run agent test-bot");
}

#[tokio::test]
async fn test_telegram_signature_verification() {
    let platform = TelegramPlatform::new(test_config()).unwrap();

    // With correct secret
    let result = platform.verify_signature(b"test payload", "test_secret").await;
    assert!(result);

    // With incorrect secret
    let result = platform.verify_signature(b"test payload", "wrong_secret").await;
    assert!(!result);
}
