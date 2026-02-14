//! Configuration integration tests

use aof_gateway::config::*;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_complete_gateway_config_loading() {
    // Set up environment variables
    std::env::set_var("TEST_SLACK_TOKEN", "xoxb-test-token");
    std::env::set_var("TEST_DISCORD_TOKEN", "discord-test-token");

    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: test-gateway

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${TEST_SLACK_TOKEN}"
        app_token: "xapp-test"
        bot_user_id: "U01234567"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

    - platform: discord
      enabled: true
      config:
        bot_token: "${TEST_DISCORD_TOKEN}"
        application_id: "123456789"
        public_key: "test-key"
      rate_limit:
        requests_per_second: 10
        burst_size: 20

  squads:
    - name: ops-team
      description: "Operations team"
      agents:
        - k8s-monitor
        - incident-responder
      channels:
        slack: "C01234567"
        discord: "987654321098765432"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    // Load config
    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    // Verify metadata
    assert_eq!(config.api_version, "aof.dev/v1");
    assert_eq!(config.kind, "Gateway");
    assert_eq!(config.metadata.name, "test-gateway");

    // Verify runtime
    assert_eq!(config.spec.runtime.websocket_url, "ws://localhost:8080/ws");

    // Verify adapters
    assert_eq!(config.spec.adapters.len(), 2);
    assert_eq!(config.spec.adapters[0].enabled, true);
    assert_eq!(config.spec.adapters[1].enabled, true);

    // Verify environment variable substitution
    let slack_token = config.spec.adapters[0].config.get("bot_token").unwrap().as_str().unwrap();
    assert_eq!(slack_token, "xoxb-test-token");

    // Verify squads
    assert_eq!(config.spec.squads.len(), 1);
    assert_eq!(config.spec.squads[0].name, "ops-team");
    assert_eq!(config.spec.squads[0].agents.len(), 2);
    assert_eq!(config.spec.squads[0].channels.slack, Some("C01234567".to_string()));
    assert_eq!(config.spec.squads[0].channels.discord, Some("987654321098765432".to_string()));
}

#[test]
fn test_multi_adapter_config() {
    std::env::set_var("TOKEN1", "token1");
    std::env::set_var("TOKEN2", "token2");
    std::env::set_var("TOKEN3", "token3");

    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: multi-adapter

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${TOKEN1}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5

    - platform: discord
      enabled: true
      config:
        bot_token: "${TOKEN2}"
      rate_limit:
        requests_per_second: 10
        burst_size: 20

    - platform: telegram
      enabled: true
      config:
        bot_token: "${TOKEN3}"
      rate_limit:
        requests_per_second: 30
        burst_size: 50

  squads: []
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.spec.adapters.len(), 3);
    assert!(config.spec.adapters.iter().all(|a| a.enabled));
}

#[test]
fn test_squad_config_loading() {
    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: squad-test

spec:
  runtime:
    websocket_url: "ws://localhost:8080/ws"

  adapters: []

  squads:
    - name: ops-team
      description: "Operations team"
      agents:
        - agent1
        - agent2
      channels:
        slack: "C01234567"

    - name: dev-team
      description: "Development team"
      agents:
        - agent3
      channels:
        discord: "987654321098765432"
        telegram: "-1001234567890"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    // Verify squads
    assert_eq!(config.spec.squads.len(), 2);

    // Test helper methods
    assert!(config.get_squad("ops-team").is_some());
    assert!(config.get_squad("dev-team").is_some());
    assert!(config.get_squad("nonexistent").is_none());

    let ops_agents = config.get_squad_agents("ops-team").unwrap();
    assert_eq!(ops_agents, vec!["agent1", "agent2"]);

    let ops_channels = config.get_squad_channels("ops-team").unwrap();
    assert_eq!(ops_channels.slack, Some("C01234567".to_string()));
    assert_eq!(ops_channels.discord, None);
}
