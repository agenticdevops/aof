use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelPlatformType, NotificationPayload,
    NotificationSeverity,
};
use agentix_runtime::channels::TelegramChannelGateway;
use std::collections::HashMap;

fn telegram_credentials() -> ChannelCredentials {
    ChannelCredentials::Telegram {
        bot_token: "123456:ABC-DEF".into(),
        webhook_secret: Some("mysecret".into()),
    }
}

fn telegram_credentials_no_secret() -> ChannelCredentials {
    ChannelCredentials::Telegram {
        bot_token: "123456:ABC-DEF".into(),
        webhook_secret: None,
    }
}

#[test]
fn test_telegram_gateway_creation() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).expect("create gateway");
    assert_eq!(gw.platform_type(), ChannelPlatformType::Telegram);
}

#[test]
fn test_telegram_gateway_creation_without_secret() {
    let creds = telegram_credentials_no_secret();
    let gw = TelegramChannelGateway::new(&creds);
    assert!(gw.is_ok());
}

#[test]
fn test_telegram_gateway_wrong_credentials() {
    let creds = ChannelCredentials::Slack {
        bot_token: "xoxb-test".into(),
        signing_secret: "secret".into(),
        app_id: "A123".into(),
    };
    let result = TelegramChannelGateway::new(&creds);
    assert!(result.is_err());
}

#[test]
fn test_parse_webhook_text_message() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "update_id": 123,
        "message": {
            "message_id": 456,
            "from": {
                "id": 789,
                "is_bot": false,
                "first_name": "Alice",
                "username": "alice"
            },
            "chat": {
                "id": -100123,
                "type": "group"
            },
            "date": 1234567890,
            "text": "hello agent"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_some());
    let msg = result.unwrap();
    assert_eq!(msg.platform, ChannelPlatformType::Telegram);
    assert_eq!(msg.channel_id, "-100123");
    assert_eq!(msg.user_id, "789");
    assert_eq!(msg.user_name, "alice");
    assert_eq!(msg.text, "hello agent");
}

#[test]
fn test_parse_webhook_no_message() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "update_id": 123,
        "edited_message": {
            "message_id": 456,
            "text": "edited"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_empty_text() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "update_id": 123,
        "message": {
            "message_id": 456,
            "from": { "id": 789, "is_bot": false, "first_name": "Alice" },
            "chat": { "id": -100123, "type": "group" },
            "date": 1234567890
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_reply() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "update_id": 123,
        "message": {
            "message_id": 456,
            "from": { "id": 789, "is_bot": false, "first_name": "Alice", "username": "alice" },
            "chat": { "id": -100123, "type": "group" },
            "date": 1234567890,
            "text": "reply text",
            "reply_to_message": { "message_id": 100 }
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    let msg = result.unwrap();
    assert_eq!(msg.thread_id.as_deref(), Some("100"));
}

#[test]
fn test_parse_webhook_private_chat() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "update_id": 123,
        "message": {
            "message_id": 456,
            "from": { "id": 789, "is_bot": false, "first_name": "Bob", "username": "bob" },
            "chat": { "id": 789, "type": "private" },
            "date": 1234567890,
            "text": "private message"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_verify_webhook_secret_valid() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let mut headers = HashMap::new();
    headers.insert(
        "x-telegram-bot-api-secret-token".to_string(),
        "mysecret".to_string(),
    );

    assert!(gw.verify_webhook_secret(&headers));
}

#[test]
fn test_verify_webhook_secret_invalid() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let mut headers = HashMap::new();
    headers.insert(
        "x-telegram-bot-api-secret-token".to_string(),
        "wrongsecret".to_string(),
    );

    assert!(!gw.verify_webhook_secret(&headers));
}

#[test]
fn test_verify_webhook_secret_missing_header() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let headers = HashMap::new();
    assert!(!gw.verify_webhook_secret(&headers));
}

#[test]
fn test_verify_webhook_secret_none_configured() {
    let creds = telegram_credentials_no_secret();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    let headers = HashMap::new();
    assert!(gw.verify_webhook_secret(&headers)); // No verification needed
}

#[test]
fn test_format_notification_info() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Test Alert".into(),
        body: "Something happened".into(),
        severity: NotificationSeverity::Info,
        run_id: None,
        metadata: Default::default(),
    };

    let text = TelegramChannelGateway::format_notification_text(&payload);
    assert!(text.contains("*Test Alert*"));
    assert!(text.contains("Something happened"));
    // Info emoji: ℹ️
    assert!(text.contains("\u{2139}"));
}

#[test]
fn test_format_notification_critical() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Critical".into(),
        body: "Down".into(),
        severity: NotificationSeverity::Critical,
        run_id: None,
        metadata: Default::default(),
    };

    let text = TelegramChannelGateway::format_notification_text(&payload);
    // Critical emoji: 🚨
    assert!(text.contains("\u{1f6a8}"));
}

#[test]
fn test_format_notification_with_run_id() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Done".into(),
        body: "Complete".into(),
        severity: NotificationSeverity::Info,
        run_id: Some("run-abc".into()),
        metadata: Default::default(),
    };

    let text = TelegramChannelGateway::format_notification_text(&payload);
    assert!(text.contains("run-abc"));
}

#[test]
fn test_api_base_url() {
    let creds = telegram_credentials();
    let gw = TelegramChannelGateway::new(&creds).unwrap();

    assert_eq!(
        gw.api_base_url(),
        "https://api.telegram.org/bot123456:ABC-DEF/"
    );
}
