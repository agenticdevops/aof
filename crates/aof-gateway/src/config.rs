//! Gateway configuration schema
//!
//! This module defines the YAML configuration schema for the messaging gateway.

use std::fs;

use serde::{Deserialize, Serialize};

use aof_core::AofError;
use crate::adapters::Platform;
use crate::rate_limiter::RateLimitConfig;

/// Gateway configuration (top-level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// API version (must be "aof.dev/v1")
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    /// Resource kind (must be "Gateway")
    pub kind: String,

    /// Metadata
    pub metadata: ConfigMetadata,

    /// Gateway specification
    pub spec: GatewaySpec,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Gateway name
    pub name: String,
}

/// Gateway specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySpec {
    /// Runtime configuration
    pub runtime: RuntimeConfig,

    /// Adapter configurations
    pub adapters: Vec<AdapterConfig>,

    /// Squad configurations
    #[serde(default)]
    pub squads: Vec<SquadConfig>,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// WebSocket URL to agent runtime
    pub websocket_url: String,

    /// Session ID (auto-generated if not set)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Platform type
    pub platform: Platform,

    /// Whether adapter is enabled
    pub enabled: bool,

    /// Platform-specific configuration (JSON blob)
    pub config: serde_json::Value,

    /// Rate limit configuration
    pub rate_limit: RateLimitConfig,
}

/// Squad configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadConfig {
    /// Squad name (unique identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Agent IDs in this squad
    pub agents: Vec<String>,

    /// Platform channel mappings
    pub channels: SquadChannels,
}

/// Squad channel mappings for each platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadChannels {
    /// Slack channel ID (C...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slack: Option<String>,

    /// Discord channel ID (numeric)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord: Option<String>,

    /// Telegram chat ID (numeric or -...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram: Option<String>,

    /// WhatsApp phone number (future)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whatsapp: Option<String>,
}

impl GatewayConfig {
    /// Get squad by name
    pub fn get_squad(&self, name: &str) -> Option<&SquadConfig> {
        self.spec.squads.iter().find(|s| s.name == name)
    }

    /// Get all agents in squad
    pub fn get_squad_agents(&self, squad_name: &str) -> Option<Vec<String>> {
        self.get_squad(squad_name).map(|s| s.agents.clone())
    }

    /// Get channels for squad
    pub fn get_squad_channels(&self, squad_name: &str) -> Option<&SquadChannels> {
        self.get_squad(squad_name).map(|s| &s.channels)
    }
}

/// Load gateway configuration from YAML file
pub fn load_gateway_config(path: &str) -> Result<GatewayConfig, AofError> {
    let content = fs::read_to_string(path)
        .map_err(|e| AofError::config(format!("Failed to read config file: {}", e)))?;

    let resolved = resolve_env_vars(&content)?;

    let deserializer = serde_yaml::Deserializer::from_str(&resolved);
    let config: GatewayConfig = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| AofError::config(format!("Config parse error at {}: {}", e.path(), e.inner())))?;

    validate_config(&config)?;

    Ok(config)
}

/// Load gateway configuration with .env file support (development)
pub fn load_config_with_dotenv(path: &str) -> Result<GatewayConfig, AofError> {
    // Load .env file if present
    dotenv::dotenv().ok();

    load_gateway_config(path)
}

/// Resolve environment variables in YAML content
fn resolve_env_vars(yaml: &str) -> Result<String, AofError> {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();
    let mut missing_vars = Vec::new();

    let result = re.replace_all(yaml, |caps: &regex::Captures| {
        let var_name = &caps[1];
        match std::env::var(var_name) {
            Ok(value) => value,
            Err(_) => {
                missing_vars.push(var_name.to_string());
                String::new()
            }
        }
    }).to_string();

    if !missing_vars.is_empty() {
        return Err(AofError::config(format!(
            "Missing required environment variables: {}",
            missing_vars.join(", ")
        )));
    }

    Ok(result)
}

/// Sanitize configuration for logging (mask sensitive tokens)
pub fn sanitize_config_for_logging(config: &GatewayConfig) -> GatewayConfig {
    let mut sanitized = config.clone();
    for adapter in &mut sanitized.spec.adapters {
        // Sanitize bot_token field
        if let Some(bot_token) = adapter.config.get("bot_token") {
            if let Some(token_str) = bot_token.as_str() {
                let masked = if token_str.len() >= 8 {
                    format!("{}...", &token_str[..8])
                } else {
                    "***".to_string()
                };
                adapter.config["bot_token"] = serde_json::json!(masked);
            }
        }

        // Sanitize app_token field
        if let Some(app_token) = adapter.config.get("app_token") {
            if let Some(token_str) = app_token.as_str() {
                let masked = if token_str.len() >= 8 {
                    format!("{}...", &token_str[..8])
                } else {
                    "***".to_string()
                };
                adapter.config["app_token"] = serde_json::json!(masked);
            }
        }
    }
    sanitized
}

/// Validate configuration
fn validate_config(config: &GatewayConfig) -> Result<(), AofError> {
    if config.api_version != "aof.dev/v1" {
        return Err(AofError::config(format!(
            "Invalid apiVersion: expected 'aof.dev/v1', got '{}'",
            config.api_version
        )));
    }

    if config.kind != "Gateway" {
        return Err(AofError::config(format!(
            "Invalid kind: expected 'Gateway', got '{}'",
            config.kind
        )));
    }

    // Validate squads
    validate_squads(config)?;

    Ok(())
}

/// Validate squad configurations
fn validate_squads(config: &GatewayConfig) -> Result<(), AofError> {
    let mut squad_names = std::collections::HashSet::new();

    for squad in &config.spec.squads {
        // Check for duplicate squad names
        if !squad_names.insert(&squad.name) {
            return Err(AofError::config(format!(
                "Duplicate squad name: '{}'",
                squad.name
            )));
        }

        // Check at least one channel configured
        let has_channel = squad.channels.slack.is_some()
            || squad.channels.discord.is_some()
            || squad.channels.telegram.is_some()
            || squad.channels.whatsapp.is_some();

        if !has_channel {
            return Err(AofError::config(format!(
                "Squad '{}' must have at least one channel configured",
                squad.name
            )));
        }

        // Validate channel IDs are non-empty
        if let Some(ref slack_id) = squad.channels.slack {
            if slack_id.trim().is_empty() {
                return Err(AofError::config(format!(
                    "Squad '{}': Slack channel ID cannot be empty",
                    squad.name
                )));
            }
        }

        if let Some(ref discord_id) = squad.channels.discord {
            if discord_id.trim().is_empty() {
                return Err(AofError::config(format!(
                    "Squad '{}': Discord channel ID cannot be empty",
                    squad.name
                )));
            }
        }

        if let Some(ref telegram_id) = squad.channels.telegram {
            if telegram_id.trim().is_empty() {
                return Err(AofError::config(format!(
                    "Squad '{}': Telegram chat ID cannot be empty",
                    squad.name
                )));
            }
        }

        // Warn about agents (don't fail - agents might not exist yet)
        if squad.agents.is_empty() {
            tracing::warn!(
                squad = %squad.name,
                "Squad has no agents configured"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_resolution() {
        std::env::set_var("TEST_TOKEN", "secret123");

        let yaml = r#"
token: ${TEST_TOKEN}
"#;

        let resolved = resolve_env_vars(yaml).unwrap();
        assert!(resolved.contains("secret123"));
    }

    #[test]
    fn test_missing_env_var_returns_error() {
        std::env::remove_var("NONEXISTENT_VAR");

        let yaml = r#"
token: ${NONEXISTENT_VAR}
"#;

        let result = resolve_env_vars(yaml);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Missing required environment variables"));
        assert!(error_message.contains("NONEXISTENT_VAR"));
    }

    #[test]
    fn test_sanitize_config() {
        let config = GatewayConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Gateway".to_string(),
            metadata: ConfigMetadata {
                name: "test".to_string(),
            },
            spec: GatewaySpec {
                runtime: RuntimeConfig {
                    websocket_url: "ws://localhost:8080".to_string(),
                    session_id: None,
                },
                adapters: vec![
                    AdapterConfig {
                        platform: Platform::Slack,
                        enabled: true,
                        config: serde_json::json!({
                            "bot_token": "xoxb-1234567890-abcdefghijklmnop",
                            "app_token": "test-app-token-placeholder"
                        }),
                        rate_limit: RateLimitConfig {
                            requests_per_second: 1,
                            burst_size: 5,
                        },
                    }
                ],
                squads: vec![],
            },
        };

        let sanitized = sanitize_config_for_logging(&config);

        // Check bot_token is masked
        let bot_token = sanitized.spec.adapters[0].config.get("bot_token").unwrap().as_str().unwrap();
        assert!(bot_token.starts_with("xoxb-123"));
        assert!(bot_token.ends_with("..."));
        assert!(!bot_token.contains("abcdefghijklmnop"));

        // Check app_token is masked
        let app_token = sanitized.spec.adapters[0].config.get("app_token").unwrap().as_str().unwrap();
        assert!(app_token.starts_with("xapp-1-A"));
        assert!(app_token.ends_with("..."));
    }

    #[test]
    fn test_load_config_with_dotenv() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create temporary config file
        let mut config_file = NamedTempFile::new().unwrap();
        std::env::set_var("TEST_BOT_TOKEN", "xoxb-test-token");

        let yaml_content = r#"
apiVersion: aof.dev/v1
kind: Gateway
metadata:
  name: test
spec:
  runtime:
    websocket_url: "ws://localhost:8080"
  adapters:
    - platform: slack
      enabled: true
      config:
        bot_token: "${TEST_BOT_TOKEN}"
      rate_limit:
        requests_per_second: 1
        burst_size: 5
  squads: []
"#;
        config_file.write_all(yaml_content.as_bytes()).unwrap();
        config_file.flush().unwrap();

        // Load config
        let config = load_config_with_dotenv(config_file.path().to_str().unwrap()).unwrap();

        // Verify token was resolved
        let bot_token = config.spec.adapters[0].config.get("bot_token").unwrap().as_str().unwrap();
        assert_eq!(bot_token, "xoxb-test-token");
    }

    #[test]
    fn test_validate_config() {
        let valid_config = GatewayConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Gateway".to_string(),
            metadata: ConfigMetadata {
                name: "test".to_string(),
            },
            spec: GatewaySpec {
                runtime: RuntimeConfig {
                    websocket_url: "ws://localhost:8080".to_string(),
                    session_id: None,
                },
                adapters: vec![],
                squads: vec![],
            },
        };

        assert!(validate_config(&valid_config).is_ok());

        let invalid_version = GatewayConfig {
            api_version: "v2".to_string(),
            ..valid_config.clone()
        };

        assert!(validate_config(&invalid_version).is_err());
    }

    #[test]
    fn test_squad_config_valid() {
        let config = GatewayConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Gateway".to_string(),
            metadata: ConfigMetadata {
                name: "test".to_string(),
            },
            spec: GatewaySpec {
                runtime: RuntimeConfig {
                    websocket_url: "ws://localhost:8080".to_string(),
                    session_id: None,
                },
                adapters: vec![],
                squads: vec![
                    SquadConfig {
                        name: "ops-team".to_string(),
                        description: "Operations team".to_string(),
                        agents: vec!["agent1".to_string(), "agent2".to_string()],
                        channels: SquadChannels {
                            slack: Some("C01234567".to_string()),
                            discord: Some("987654321098765432".to_string()),
                            telegram: None,
                            whatsapp: None,
                        },
                    }
                ],
            },
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_squad_duplicate_names() {
        let config = GatewayConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Gateway".to_string(),
            metadata: ConfigMetadata {
                name: "test".to_string(),
            },
            spec: GatewaySpec {
                runtime: RuntimeConfig {
                    websocket_url: "ws://localhost:8080".to_string(),
                    session_id: None,
                },
                adapters: vec![],
                squads: vec![
                    SquadConfig {
                        name: "ops-team".to_string(),
                        description: "First".to_string(),
                        agents: vec!["agent1".to_string()],
                        channels: SquadChannels {
                            slack: Some("C01234567".to_string()),
                            discord: None,
                            telegram: None,
                            whatsapp: None,
                        },
                    },
                    SquadConfig {
                        name: "ops-team".to_string(),
                        description: "Duplicate".to_string(),
                        agents: vec!["agent2".to_string()],
                        channels: SquadChannels {
                            slack: Some("C98765432".to_string()),
                            discord: None,
                            telegram: None,
                            whatsapp: None,
                        },
                    },
                ],
            },
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate squad name"));
    }

    #[test]
    fn test_squad_helper_methods() {
        let config = GatewayConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Gateway".to_string(),
            metadata: ConfigMetadata {
                name: "test".to_string(),
            },
            spec: GatewaySpec {
                runtime: RuntimeConfig {
                    websocket_url: "ws://localhost:8080".to_string(),
                    session_id: None,
                },
                adapters: vec![],
                squads: vec![
                    SquadConfig {
                        name: "ops-team".to_string(),
                        description: "Operations team".to_string(),
                        agents: vec!["agent1".to_string(), "agent2".to_string()],
                        channels: SquadChannels {
                            slack: Some("C01234567".to_string()),
                            discord: None,
                            telegram: None,
                            whatsapp: None,
                        },
                    }
                ],
            },
        };

        // Test get_squad
        assert!(config.get_squad("ops-team").is_some());
        assert!(config.get_squad("nonexistent").is_none());

        // Test get_squad_agents
        let agents = config.get_squad_agents("ops-team");
        assert!(agents.is_some());
        assert_eq!(agents.unwrap(), vec!["agent1", "agent2"]);

        // Test get_squad_channels
        let channels = config.get_squad_channels("ops-team");
        assert!(channels.is_some());
        assert_eq!(channels.unwrap().slack, Some("C01234567".to_string()));
    }
}
