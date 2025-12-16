//! Discord platform tests

use aof_triggers::platforms::discord::{DiscordPlatform, DiscordConfig};
use aof_triggers::platforms::TriggerPlatform;
use std::collections::HashMap;

fn test_config() -> DiscordConfig {
    DiscordConfig {
        bot_token: "MTIzNDU2Nzg5.TEST.token".to_string(),
        application_id: "123456789012345678".to_string(),
        // Valid Ed25519 public key (32 bytes in hex) - using a known valid test key
        // This is the Y coordinate of a valid point on the Ed25519 curve
        public_key: "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c".to_string(),
        guild_ids: None,
        allowed_roles: None,
    }
}

#[tokio::test]
async fn test_discord_platform_creation() {
    let platform = DiscordPlatform::from_discord_config(test_config());
    assert!(platform.is_ok());
}

#[tokio::test]
async fn test_discord_bot_name() {
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    // Discord platform returns "aof" as default bot name
    assert_eq!(platform.bot_name(), "aof");
}

#[tokio::test]
async fn test_discord_platform_name() {
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    assert_eq!(platform.platform_name(), "discord");
}

#[tokio::test]
async fn test_discord_supports_threading() {
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    assert!(platform.supports_threading());
}

#[tokio::test]
async fn test_discord_supports_interactive() {
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    assert!(platform.supports_interactive());
}

#[tokio::test]
async fn test_discord_parse_ping() {
    let payload = r#"{"type": 1, "id": "123", "application_id": "456"}"#;
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    // Ping should return error (handled separately)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discord_parse_invalid_json() {
    let payload = "invalid json";
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_discord_parse_slash_command() {
    let payload = r#"{
        "id": "1234567890123456789",
        "type": 2,
        "token": "A_UNIQUE_TOKEN",
        "application_id": "123456789012345678",
        "data": {"name": "agent", "options": [{"name": "action", "value": "run"}, {"name": "target", "value": "test-bot"}]},
        "channel_id": "3333333333333333333",
        "user": {
            "id": "4444444444444444444",
            "username": "testuser"
        }
    }"#;
    let platform = DiscordPlatform::from_discord_config(test_config()).unwrap();
    let headers = HashMap::new();
    let message = platform.parse_message(payload.as_bytes(), &headers).await.unwrap();

    assert_eq!(message.platform, "discord");
    assert_eq!(message.user.id, "4444444444444444444");
    assert_eq!(message.channel_id, "3333333333333333333");
    assert!(message.text.contains("agent"));
}
