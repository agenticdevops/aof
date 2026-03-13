use agentix_core::channel::{
    ChannelConfig, ChannelCredentials, ChannelDirection, ChannelGateway, ChannelMessage,
    ChannelPlatformType, ChannelRoute, NotificationPayload, NotificationSeverity,
    NotificationTarget,
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_channel_platform_type_variants() {
    let slack = ChannelPlatformType::Slack;
    let telegram = ChannelPlatformType::Telegram;
    let discord = ChannelPlatformType::Discord;

    assert_eq!(format!("{}", slack), "slack");
    assert_eq!(format!("{}", telegram), "telegram");
    assert_eq!(format!("{}", discord), "discord");
}

#[test]
fn test_channel_credentials_slack() {
    let creds = ChannelCredentials::Slack {
        bot_token: "xoxb-test-token".into(),
        signing_secret: "secret123".into(),
        app_id: "A12345".into(),
    };

    match &creds {
        ChannelCredentials::Slack {
            bot_token,
            signing_secret,
            app_id,
        } => {
            assert_eq!(bot_token, "xoxb-test-token");
            assert_eq!(signing_secret, "secret123");
            assert_eq!(app_id, "A12345");
        }
        _ => panic!("Expected Slack credentials"),
    }
}

#[test]
fn test_channel_credentials_telegram() {
    let creds = ChannelCredentials::Telegram {
        bot_token: "123456:ABC-DEF".into(),
        webhook_secret: Some("secret".into()),
    };

    match &creds {
        ChannelCredentials::Telegram {
            bot_token,
            webhook_secret,
        } => {
            assert_eq!(bot_token, "123456:ABC-DEF");
            assert_eq!(webhook_secret.as_deref(), Some("secret"));
        }
        _ => panic!("Expected Telegram credentials"),
    }
}

#[test]
fn test_channel_credentials_discord() {
    let creds = ChannelCredentials::Discord {
        bot_token: "discord-token".into(),
        application_id: "APP123".into(),
        public_key: "pubkey456".into(),
    };

    match &creds {
        ChannelCredentials::Discord {
            bot_token,
            application_id,
            public_key,
        } => {
            assert_eq!(bot_token, "discord-token");
            assert_eq!(application_id, "APP123");
            assert_eq!(public_key, "pubkey456");
        }
        _ => panic!("Expected Discord credentials"),
    }
}

#[test]
fn test_channel_route_matches_agent_inbound() {
    let route = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Inbound,
        description: None,
    };

    assert!(route.matches_agent("dba-optimizer"));
    assert!(!route.matches_agent("other-agent"));
}

#[test]
fn test_channel_route_matches_agent_bidirectional() {
    let route = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Bidirectional,
        description: None,
    };

    assert!(route.matches_agent("dba-optimizer"));
}

#[test]
fn test_channel_route_matches_agent_outbound_only() {
    let route = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Outbound,
        description: None,
    };

    // Outbound-only routes don't accept inbound messages
    assert!(!route.matches_agent("dba-optimizer"));
}

#[test]
fn test_channel_route_allows_outbound() {
    let route_out = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Outbound,
        description: None,
    };
    assert!(route_out.allows_outbound("dba-optimizer"));
    assert!(!route_out.allows_outbound("other-agent"));

    let route_in = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Inbound,
        description: None,
    };
    assert!(!route_in.allows_outbound("dba-optimizer"));
}

#[test]
fn test_channel_route_allows_outbound_bidirectional() {
    let route = ChannelRoute {
        channel_id: "C123".into(),
        agent_name: "dba-optimizer".into(),
        direction: ChannelDirection::Bidirectional,
        description: None,
    };

    assert!(route.allows_outbound("dba-optimizer"));
}

#[test]
fn test_channel_direction_default() {
    assert_eq!(ChannelDirection::default(), ChannelDirection::Bidirectional);
}

#[test]
fn test_channel_config_serde_yaml_roundtrip() {
    let config = ChannelConfig {
        platform: ChannelPlatformType::Slack,
        credentials: ChannelCredentials::Slack {
            bot_token: "xoxb-test".into(),
            signing_secret: "secret".into(),
            app_id: "A123".into(),
        },
        routes: vec![ChannelRoute {
            channel_id: "C_DEVOPS".into(),
            agent_name: "dba-optimizer".into(),
            direction: ChannelDirection::Bidirectional,
            description: Some("DevOps channel".into()),
        }],
        enabled: true,
    };

    let yaml = serde_yaml::to_string(&config).expect("serialize to YAML");
    let deserialized: ChannelConfig = serde_yaml::from_str(&yaml).expect("deserialize from YAML");

    assert_eq!(deserialized.platform, ChannelPlatformType::Slack);
    assert_eq!(deserialized.routes.len(), 1);
    assert_eq!(deserialized.routes[0].channel_id, "C_DEVOPS");
    assert_eq!(deserialized.routes[0].agent_name, "dba-optimizer");
    assert!(deserialized.enabled);
}

#[test]
fn test_channel_config_serde_json_roundtrip() {
    let config = ChannelConfig {
        platform: ChannelPlatformType::Telegram,
        credentials: ChannelCredentials::Telegram {
            bot_token: "123456:ABC".into(),
            webhook_secret: Some("secret".into()),
        },
        routes: vec![ChannelRoute {
            channel_id: "-100123".into(),
            agent_name: "monitor-agent".into(),
            direction: ChannelDirection::Inbound,
            description: None,
        }],
        enabled: true,
    };

    let json = serde_json::to_string(&config).expect("serialize to JSON");
    let deserialized: ChannelConfig = serde_json::from_str(&json).expect("deserialize from JSON");

    assert_eq!(deserialized.platform, ChannelPlatformType::Telegram);
    assert_eq!(deserialized.routes.len(), 1);
    assert_eq!(deserialized.routes[0].channel_id, "-100123");
    assert!(deserialized.enabled);
}

#[test]
fn test_notification_target_creation() {
    let target = NotificationTarget {
        platform: ChannelPlatformType::Slack,
        channel_id: "C12345".into(),
        thread_id: None,
    };

    assert_eq!(target.platform, ChannelPlatformType::Slack);
    assert_eq!(target.channel_id, "C12345");
    assert!(target.thread_id.is_none());
}

#[test]
fn test_notification_payload_serde_roundtrip() {
    let mut metadata = HashMap::new();
    metadata.insert("env".into(), json!("production"));

    let payload = NotificationPayload {
        agent_name: "cost-monitor".into(),
        title: "Budget Alert".into(),
        body: "Monthly spend exceeded threshold".into(),
        severity: NotificationSeverity::Warning,
        run_id: Some("run-abc-123".into()),
        metadata,
    };

    let json = serde_json::to_string(&payload).expect("serialize");
    let deserialized: NotificationPayload = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.agent_name, "cost-monitor");
    assert_eq!(deserialized.title, "Budget Alert");
    assert_eq!(deserialized.body, "Monthly spend exceeded threshold");
    assert_eq!(deserialized.severity, NotificationSeverity::Warning);
    assert_eq!(deserialized.run_id.as_deref(), Some("run-abc-123"));
    assert_eq!(deserialized.metadata.get("env"), Some(&json!("production")));
}

#[test]
fn test_notification_severity_display() {
    assert_eq!(format!("{}", NotificationSeverity::Info), "info");
    assert_eq!(format!("{}", NotificationSeverity::Warning), "warning");
    assert_eq!(format!("{}", NotificationSeverity::Error), "error");
    assert_eq!(format!("{}", NotificationSeverity::Critical), "critical");
}

#[test]
fn test_channel_message_creation() {
    let now = Utc::now();
    let msg = ChannelMessage {
        platform: ChannelPlatformType::Discord,
        channel_id: "C999".into(),
        user_id: "U555".into(),
        user_name: "alice".into(),
        text: "hello agent".into(),
        thread_id: Some("thread-123".into()),
        timestamp: now,
        raw_payload: json!({"test": true}),
    };

    assert_eq!(msg.platform, ChannelPlatformType::Discord);
    assert_eq!(msg.channel_id, "C999");
    assert_eq!(msg.user_id, "U555");
    assert_eq!(msg.user_name, "alice");
    assert_eq!(msg.text, "hello agent");
    assert_eq!(msg.thread_id.as_deref(), Some("thread-123"));
    assert_eq!(msg.timestamp, now);
}

#[test]
fn test_channel_config_disabled() {
    let config = ChannelConfig {
        platform: ChannelPlatformType::Slack,
        credentials: ChannelCredentials::Slack {
            bot_token: "xoxb-test".into(),
            signing_secret: "secret".into(),
            app_id: "A123".into(),
        },
        routes: vec![],
        enabled: false,
    };

    assert!(!config.enabled);
}

#[test]
fn test_multiple_routes_per_config() {
    let config = ChannelConfig {
        platform: ChannelPlatformType::Slack,
        credentials: ChannelCredentials::Slack {
            bot_token: "xoxb-test".into(),
            signing_secret: "secret".into(),
            app_id: "A123".into(),
        },
        routes: vec![
            ChannelRoute {
                channel_id: "C1".into(),
                agent_name: "agent-a".into(),
                direction: ChannelDirection::Bidirectional,
                description: None,
            },
            ChannelRoute {
                channel_id: "C2".into(),
                agent_name: "agent-b".into(),
                direction: ChannelDirection::Inbound,
                description: None,
            },
            ChannelRoute {
                channel_id: "C3".into(),
                agent_name: "agent-c".into(),
                direction: ChannelDirection::Outbound,
                description: Some("Alerts only".into()),
            },
        ],
        enabled: true,
    };

    assert_eq!(config.routes.len(), 3);
}
