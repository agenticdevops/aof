use agentix_core::channel::{
    ChannelConfig, ChannelCredentials, ChannelDirection, ChannelPlatformType, ChannelRoute,
};
use agentix_runtime::channels::ChannelGatewayManager;

fn slack_config(routes: Vec<ChannelRoute>) -> ChannelConfig {
    ChannelConfig {
        platform: ChannelPlatformType::Slack,
        credentials: ChannelCredentials::Slack {
            bot_token: "xoxb-test".into(),
            signing_secret: "secret".into(),
            app_id: "A123".into(),
        },
        routes,
        enabled: true,
    }
}

fn telegram_config(routes: Vec<ChannelRoute>) -> ChannelConfig {
    ChannelConfig {
        platform: ChannelPlatformType::Telegram,
        credentials: ChannelCredentials::Telegram {
            bot_token: "123456:ABC".into(),
            webhook_secret: None,
        },
        routes,
        enabled: true,
    }
}

fn discord_config(routes: Vec<ChannelRoute>) -> ChannelConfig {
    ChannelConfig {
        platform: ChannelPlatformType::Discord,
        credentials: ChannelCredentials::Discord {
            bot_token: "discord-token".into(),
            application_id: "APP123".into(),
            public_key: "pubkey".into(),
        },
        routes,
        enabled: true,
    }
}

fn make_route(channel_id: &str, agent_name: &str, direction: ChannelDirection) -> ChannelRoute {
    ChannelRoute {
        channel_id: channel_id.into(),
        agent_name: agent_name.into(),
        direction,
        description: None,
    }
}

#[test]
fn test_manager_creation() {
    let configs = vec![slack_config(vec![make_route(
        "C123",
        "agent-a",
        ChannelDirection::Bidirectional,
    )])];

    let mgr = ChannelGatewayManager::new(configs).expect("create manager");
    assert_eq!(mgr.get_all_routes().len(), 1);
}

#[test]
fn test_manager_multiple_platforms() {
    let configs = vec![
        slack_config(vec![make_route(
            "C1",
            "agent-a",
            ChannelDirection::Bidirectional,
        )]),
        telegram_config(vec![
            make_route("T1", "agent-b", ChannelDirection::Inbound),
            make_route("T2", "agent-c", ChannelDirection::Outbound),
        ]),
        discord_config(vec![make_route(
            "D1",
            "agent-d",
            ChannelDirection::Bidirectional,
        )]),
    ];

    let mgr = ChannelGatewayManager::new(configs).expect("create manager");
    assert_eq!(mgr.get_all_routes().len(), 4);
}

#[test]
fn test_route_inbound_match() {
    let configs = vec![slack_config(vec![make_route(
        "C123",
        "dba-optimizer",
        ChannelDirection::Bidirectional,
    )])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();
    let result = mgr.route_inbound(ChannelPlatformType::Slack, "C123");

    assert_eq!(result, Some("dba-optimizer".to_string()));
}

#[test]
fn test_route_inbound_no_match() {
    let configs = vec![slack_config(vec![make_route(
        "C123",
        "dba-optimizer",
        ChannelDirection::Bidirectional,
    )])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();
    let result = mgr.route_inbound(ChannelPlatformType::Slack, "C999");

    assert!(result.is_none());
}

#[test]
fn test_route_inbound_outbound_only() {
    let configs = vec![slack_config(vec![make_route(
        "C123",
        "dba-optimizer",
        ChannelDirection::Outbound,
    )])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();
    let result = mgr.route_inbound(ChannelPlatformType::Slack, "C123");

    assert!(result.is_none());
}

#[test]
fn test_route_inbound_wrong_platform() {
    let configs = vec![slack_config(vec![make_route(
        "C123",
        "dba-optimizer",
        ChannelDirection::Bidirectional,
    )])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();
    let result = mgr.route_inbound(ChannelPlatformType::Telegram, "C123");

    assert!(result.is_none());
}

#[test]
fn test_disabled_config_skipped() {
    let mut config = slack_config(vec![make_route(
        "C123",
        "agent-a",
        ChannelDirection::Bidirectional,
    )]);
    config.enabled = false;

    let mgr = ChannelGatewayManager::new(vec![config]).expect("create manager");
    assert!(mgr.get_all_routes().is_empty());
}

#[test]
fn test_multiple_routes_same_platform() {
    let configs = vec![slack_config(vec![
        make_route("C1", "agent-a", ChannelDirection::Bidirectional),
        make_route("C2", "agent-b", ChannelDirection::Inbound),
        make_route("C3", "agent-c", ChannelDirection::Outbound),
    ])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();

    assert_eq!(
        mgr.route_inbound(ChannelPlatformType::Slack, "C1"),
        Some("agent-a".to_string())
    );
    assert_eq!(
        mgr.route_inbound(ChannelPlatformType::Slack, "C2"),
        Some("agent-b".to_string())
    );
    assert_eq!(
        mgr.route_inbound(ChannelPlatformType::Slack, "C3"),
        None // Outbound-only
    );
}

#[test]
fn test_get_all_routes_info() {
    let configs = vec![slack_config(vec![ChannelRoute {
        channel_id: "C_TEST".into(),
        agent_name: "test-agent".into(),
        direction: ChannelDirection::Bidirectional,
        description: Some("Test channel".into()),
    }])];

    let mgr = ChannelGatewayManager::new(configs).unwrap();
    let routes = mgr.get_all_routes();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].platform, ChannelPlatformType::Slack);
    assert_eq!(routes[0].channel_id, "C_TEST");
    assert_eq!(routes[0].agent_name, "test-agent");
    assert_eq!(routes[0].direction, ChannelDirection::Bidirectional);
    assert_eq!(routes[0].description.as_deref(), Some("Test channel"));
}
