//! Slack platform tests

use aof_triggers::platforms::slack::{SlackPlatform, SlackConfig};
use aof_triggers::platforms::TriggerPlatform;
use std::collections::HashMap;

fn test_config() -> SlackConfig {
    SlackConfig {
        bot_token: "xoxb-test-token".to_string(),
        signing_secret: "test_signing_secret".to_string(),
        app_id: "A0123456789".to_string(),
        bot_user_id: "U0123456789".to_string(),
        bot_name: "aofbot".to_string(),
        allowed_workspaces: None,
        allowed_channels: None,
    }
}

#[tokio::test]
async fn test_slack_platform_creation() {
    let platform = SlackPlatform::new(test_config());
    assert!(platform.is_ok());
}

#[tokio::test]
async fn test_slack_parse_message_event() {
    let payload = r#"{
        "type": "event_callback",
        "team_id": "T0123456789",
        "event": {
            "type": "message",
            "user": "U1234567890",
            "channel": "C1234567890",
            "text": "/run agent test-bot hello",
            "ts": "1638360000.000100"
        }
    }"#;

    let platform = SlackPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let message = platform.parse_message(payload.as_bytes(), &headers).await.unwrap();

    assert_eq!(message.platform, "slack");
    assert_eq!(message.user.id, "U1234567890");
    assert_eq!(message.channel_id, "C1234567890");
    assert_eq!(message.text, "/run agent test-bot hello");
}

#[tokio::test]
async fn test_slack_parse_url_verification() {
    let payload = r#"{"type": "url_verification", "challenge": "3eZbrw1aBm2rZgRNFdxV2595E9CY3gmdALWMmHkvFXO7tYXAYM8P"}"#;
    let platform = SlackPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    // URL verification returns error with challenge
    assert!(result.is_err());
}

#[tokio::test]
async fn test_slack_parse_missing_event() {
    let payload = r#"{"type": "event_callback", "team_id": "T0123"}"#;
    let platform = SlackPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_slack_bot_name() {
    use aof_triggers::TriggerPlatform;

    let platform = SlackPlatform::new(test_config()).unwrap();
    assert_eq!(platform.bot_name(), "aofbot");
}

#[tokio::test]
async fn test_slack_platform_name() {
    use aof_triggers::TriggerPlatform;

    let platform = SlackPlatform::new(test_config()).unwrap();
    assert_eq!(platform.platform_name(), "slack");
}

#[tokio::test]
async fn test_slack_supports_threading() {
    use aof_triggers::TriggerPlatform;

    let platform = SlackPlatform::new(test_config()).unwrap();
    assert!(platform.supports_threading());
}

#[tokio::test]
async fn test_slack_supports_interactive() {
    use aof_triggers::TriggerPlatform;

    let platform = SlackPlatform::new(test_config()).unwrap();
    assert!(platform.supports_interactive());
}
