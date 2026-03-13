use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelPlatformType, NotificationPayload,
    NotificationSeverity,
};
use agentix_runtime::channels::SlackChannelGateway;
use std::collections::HashMap;

fn slack_credentials() -> ChannelCredentials {
    ChannelCredentials::Slack {
        bot_token: "xoxb-test-token".into(),
        signing_secret: "test_signing_secret".into(),
        app_id: "A123TEST".into(),
    }
}

#[test]
fn test_slack_gateway_creation() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).expect("create gateway");
    assert_eq!(gw.platform_type(), ChannelPlatformType::Slack);
}

#[test]
fn test_parse_webhook_app_mention() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "type": "event_callback",
        "event": {
            "type": "app_mention",
            "channel": "C123",
            "user": "U456",
            "text": "<@BOT> help me",
            "ts": "1234567890.123456"
        },
        "team_id": "T789"
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_some());
    let msg = result.unwrap();
    assert_eq!(msg.platform, ChannelPlatformType::Slack);
    assert_eq!(msg.channel_id, "C123");
    assert_eq!(msg.user_id, "U456");
    assert!(msg.text.contains("help me"));
}

#[test]
fn test_parse_webhook_url_verification() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "type": "url_verification",
        "challenge": "abc123"
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_non_mention_message() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "type": "event_callback",
        "event": {
            "type": "message",
            "channel": "C123",
            "user": "U456",
            "text": "hello",
            "ts": "123"
        },
        "team_id": "T789"
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_with_thread() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "type": "event_callback",
        "event": {
            "type": "app_mention",
            "channel": "C123",
            "user": "U456",
            "text": "<@BOT> help",
            "ts": "1234567890.123456",
            "thread_ts": "1234567890.000000"
        },
        "team_id": "T789"
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    let msg = result.unwrap();
    assert_eq!(msg.thread_id.as_deref(), Some("1234567890.000000"));
}

#[test]
fn test_verify_signature_valid() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    let payload = b"test body";
    let timestamp = "1234567890";

    // Compute expected signature
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let base_string = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(payload));
    let mut mac = HmacSha256::new_from_slice(b"test_signing_secret").unwrap();
    mac.update(base_string.as_bytes());
    let result = mac.finalize();
    let expected_sig = format!("v0={}", hex::encode(result.into_bytes()));

    assert!(gw.verify_signature(payload, &expected_sig, timestamp));
}

#[test]
fn test_verify_signature_invalid() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    assert!(!gw.verify_signature(b"test body", "v0=deadbeef", "1234567890"));
}

#[test]
fn test_verify_signature_wrong_format() {
    let creds = slack_credentials();
    let gw = SlackChannelGateway::new(&creds).unwrap();

    assert!(!gw.verify_signature(b"test body", "invalid_format", "1234567890"));
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

    let blocks = SlackChannelGateway::format_notification_blocks(&payload);
    let blocks_str = serde_json::to_string(&blocks).unwrap();

    assert!(blocks_str.contains("#36a64f"));
    assert!(blocks_str.contains("Test Alert"));
}

#[test]
fn test_format_notification_critical() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Critical Alert".into(),
        body: "System is down".into(),
        severity: NotificationSeverity::Critical,
        run_id: None,
        metadata: Default::default(),
    };

    let blocks = SlackChannelGateway::format_notification_blocks(&payload);
    let blocks_str = serde_json::to_string(&blocks).unwrap();

    assert!(blocks_str.contains("#dc3545"));
    assert!(blocks_str.contains("Critical Alert"));
    assert!(blocks_str.contains("Critical"));
}

#[test]
fn test_format_notification_with_run_id() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Run Complete".into(),
        body: "Task finished".into(),
        severity: NotificationSeverity::Info,
        run_id: Some("run-123".into()),
        metadata: Default::default(),
    };

    let blocks = SlackChannelGateway::format_notification_blocks(&payload);
    let blocks_str = serde_json::to_string(&blocks).unwrap();

    assert!(blocks_str.contains("run-123"));
}
