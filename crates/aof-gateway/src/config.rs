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

/// Load gateway configuration from YAML file
pub fn load_gateway_config(path: &str) -> Result<GatewayConfig, AofError> {
    let content = fs::read_to_string(path)
        .map_err(|e| AofError::config(format!("Failed to read config file: {}", e)))?;

    let resolved = resolve_env_vars(&content);

    let deserializer = serde_yaml::Deserializer::from_str(&resolved);
    let config: GatewayConfig = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| AofError::config(format!("Config parse error at {}: {}", e.path(), e.inner())))?;

    validate_config(&config)?;

    Ok(config)
}

/// Resolve environment variables in YAML content
fn resolve_env_vars(yaml: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();
    re.replace_all(yaml, |caps: &regex::Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_else(|_| {
            tracing::warn!("Environment variable {} not set, using empty string", var_name);
            String::new()
        })
    }).to_string()
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
other: ${NONEXISTENT}
"#;

        let resolved = resolve_env_vars(yaml);
        assert!(resolved.contains("secret123"));
        assert!(resolved.contains("other: "));
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
            },
        };

        assert!(validate_config(&valid_config).is_ok());

        let invalid_version = GatewayConfig {
            api_version: "v2".to_string(),
            ..valid_config.clone()
        };

        assert!(validate_config(&invalid_version).is_err());
    }
}
