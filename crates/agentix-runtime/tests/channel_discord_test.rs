use agentix_core::channel::{
    ChannelCredentials, ChannelGateway, ChannelPlatformType, NotificationPayload,
    NotificationSeverity,
};
use agentix_runtime::channels::DiscordChannelGateway;
use std::collections::HashMap;

fn discord_credentials() -> ChannelCredentials {
    ChannelCredentials::Discord {
        bot_token: "discord-bot-token".into(),
        application_id: "BOT123".into(),
        public_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
    }
}

#[test]
fn test_discord_gateway_creation() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).expect("create gateway");
    assert_eq!(gw.platform_type(), ChannelPlatformType::Discord);
}

#[test]
fn test_discord_gateway_wrong_credentials() {
    let creds = ChannelCredentials::Slack {
        bot_token: "xoxb-test".into(),
        signing_secret: "secret".into(),
        app_id: "A123".into(),
    };
    let result = DiscordChannelGateway::new(&creds);
    assert!(result.is_err());
}

#[test]
fn test_parse_webhook_message_create_with_mention() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "msg123",
            "channel_id": "C999",
            "guild_id": "G111",
            "content": "<@BOT123> help me",
            "author": {
                "id": "U555",
                "username": "alice",
                "discriminator": "0001"
            },
            "timestamp": "2026-03-13T00:00:00+00:00"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_some());
    let msg = result.unwrap();
    assert_eq!(msg.platform, ChannelPlatformType::Discord);
    assert_eq!(msg.channel_id, "C999");
    assert_eq!(msg.user_id, "U555");
    assert_eq!(msg.user_name, "alice");
    assert!(msg.text.contains("help me"));
}

#[test]
fn test_parse_webhook_message_create_without_mention() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "msg123",
            "channel_id": "C999",
            "content": "just chatting",
            "author": {
                "id": "U555",
                "username": "alice"
            },
            "timestamp": "2026-03-13T00:00:00+00:00"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_mention_other_bot() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "msg123",
            "channel_id": "C999",
            "content": "<@OTHER456> do something",
            "author": {
                "id": "U555",
                "username": "alice"
            },
            "timestamp": "2026-03-13T00:00:00+00:00"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_ping_interaction() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "type": 1
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_parse_webhook_with_message_reference() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "msg123",
            "channel_id": "C999",
            "content": "<@BOT123> reply here",
            "author": {
                "id": "U555",
                "username": "alice"
            },
            "timestamp": "2026-03-13T00:00:00+00:00",
            "message_reference": {
                "message_id": "ref123"
            }
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    let msg = result.unwrap();
    assert_eq!(msg.thread_id.as_deref(), Some("ref123"));
}

#[test]
fn test_parse_webhook_bot_author_ignored() {
    let creds = discord_credentials();
    let gw = DiscordChannelGateway::new(&creds).unwrap();

    let payload = serde_json::json!({
        "t": "MESSAGE_CREATE",
        "d": {
            "id": "msg123",
            "channel_id": "C999",
            "content": "<@BOT123> self message",
            "author": {
                "id": "BOT123",
                "username": "mybot",
                "bot": true
            },
            "timestamp": "2026-03-13T00:00:00+00:00"
        }
    });

    let bytes = serde_json::to_vec(&payload).unwrap();
    let headers = HashMap::new();
    let result = gw.parse_webhook(&bytes, &headers).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_format_notification_info_embed() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Test Alert".into(),
        body: "Something happened".into(),
        severity: NotificationSeverity::Info,
        run_id: None,
        metadata: Default::default(),
    };

    let embed = DiscordChannelGateway::format_notification_embed(&payload);
    let embeds = embed.get("embeds").unwrap().as_array().unwrap();
    assert_eq!(embeds.len(), 1);

    let embed_obj = &embeds[0];
    assert_eq!(embed_obj.get("color").unwrap().as_i64().unwrap(), 0x36a64f);
    assert_eq!(embed_obj.get("title").unwrap().as_str().unwrap(), "Test Alert");
    assert_eq!(
        embed_obj.get("description").unwrap().as_str().unwrap(),
        "Something happened"
    );
}

#[test]
fn test_format_notification_critical_embed() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Critical".into(),
        body: "Down".into(),
        severity: NotificationSeverity::Critical,
        run_id: None,
        metadata: Default::default(),
    };

    let embed = DiscordChannelGateway::format_notification_embed(&payload);
    let embeds = embed.get("embeds").unwrap().as_array().unwrap();
    assert_eq!(embeds[0].get("color").unwrap().as_i64().unwrap(), 0xff0000);
}

#[test]
fn test_format_notification_with_run_id() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Run Complete".into(),
        body: "Finished".into(),
        severity: NotificationSeverity::Info,
        run_id: Some("run-xyz".into()),
        metadata: Default::default(),
    };

    let embed = DiscordChannelGateway::format_notification_embed(&payload);
    let embed_str = serde_json::to_string(&embed).unwrap();
    assert!(embed_str.contains("run-xyz"));
}

#[test]
fn test_format_notification_embed_structure() {
    let payload = NotificationPayload {
        agent_name: "test-agent".into(),
        title: "Test".into(),
        body: "Body".into(),
        severity: NotificationSeverity::Warning,
        run_id: None,
        metadata: Default::default(),
    };

    let embed = DiscordChannelGateway::format_notification_embed(&payload);

    // Verify structure
    assert!(embed.get("embeds").is_some());
    let embeds = embed.get("embeds").unwrap().as_array().unwrap();
    assert_eq!(embeds.len(), 1);

    let e = &embeds[0];
    assert!(e.get("title").is_some());
    assert!(e.get("description").is_some());
    assert!(e.get("color").is_some());
    assert!(e.get("footer").is_some());
    assert!(e.get("timestamp").is_some());
}
