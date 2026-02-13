//! Configuration loading tests

use std::io::Write;
use tempfile::NamedTempFile;

use aof_gateway::config::load_gateway_config;

#[test]
fn test_valid_config_loads() {
    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: test-gateway
spec:
  runtime:
    websocket_url: ws://localhost:8080/ws
  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: test-token
      rate_limit:
        requests_per_second: 1
        burst_size: 5
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.api_version, "aof.dev/v1");
    assert_eq!(config.kind, "Gateway");
    assert_eq!(config.metadata.name, "test-gateway");
    assert_eq!(config.spec.adapters.len(), 1);
    assert_eq!(config.spec.adapters[0].rate_limit.requests_per_second, 1);
}

#[test]
fn test_env_var_substitution() {
    std::env::set_var("TEST_SLACK_TOKEN", "xoxb-secret-token");

    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: env-test-gateway
spec:
  runtime:
    websocket_url: ws://localhost:8080/ws
  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: ${TEST_SLACK_TOKEN}
      rate_limit:
        requests_per_second: 1
        burst_size: 5
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    // Verify env var was substituted
    assert_eq!(
        config.spec.adapters[0].config.get("bot_token").unwrap().as_str().unwrap(),
        "xoxb-secret-token"
    );

    std::env::remove_var("TEST_SLACK_TOKEN");
}

#[test]
fn test_invalid_api_version() {
    let yaml = r#"
apiVersion: v2
kind: Gateway
metadata:
  name: invalid-gateway
spec:
  runtime:
    websocket_url: ws://localhost:8080/ws
  adapters: []
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = load_gateway_config(file.path().to_str().unwrap());

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid apiVersion"));
}

#[test]
fn test_invalid_kind() {
    let yaml = r#"
apiVersion: aof.dev/v1
kind: NotAGateway
metadata:
  name: invalid-gateway
spec:
  runtime:
    websocket_url: ws://localhost:8080/ws
  adapters: []
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = load_gateway_config(file.path().to_str().unwrap());

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid kind"));
}

#[test]
fn test_disabled_adapter_loaded() {
    let yaml = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: disabled-test
spec:
  runtime:
    websocket_url: ws://localhost:8080/ws
  adapters:
    - platform: slack
      enabled: false
      config:
        bot_token: token
      rate_limit:
        requests_per_second: 1
        burst_size: 5
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_gateway_config(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.spec.adapters.len(), 1);
    assert!(!config.spec.adapters[0].enabled);
}
