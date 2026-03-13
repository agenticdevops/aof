//! Workspace configuration types for `agentix.yaml`.
//!
//! The workspace configuration file (`agentix.yaml`) is the entry point for an
//! OpenAgentiX workspace. It defines provider credentials, gateway settings,
//! agent discovery paths, and defaults inherited by all agents.
//!
//! See `docs/spec/workspace-config.md` for the full specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::AgentMode;
use crate::security::SecurityConfig;

// ---------------------------------------------------------------------------
// WorkspaceConfig — top-level workspace configuration
// ---------------------------------------------------------------------------

/// Top-level workspace configuration from `agentix.yaml`.
///
/// ```yaml
/// apiVersion: openagentix.dev/v1
/// kind: Workspace
/// metadata:
///   name: my-project
/// spec:
///   defaults:
///     model: anthropic/claude-sonnet-4-6
///   providers:
///     anthropic:
///       api_key: "${ANTHROPIC_API_KEY}"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Kubernetes-style API version header.
    #[serde(rename = "apiVersion", default)]
    pub api_version: String,
    /// Must be "Workspace".
    #[serde(default)]
    pub kind: String,
    /// Workspace metadata.
    pub metadata: WorkspaceMetadata,
    /// Workspace specification (providers, defaults, gateway, agents_dir).
    pub spec: WorkspaceSpec,
}

/// Workspace metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Workspace name (DNS-compatible, lowercase, hyphens).
    pub name: String,
    /// Optional key-value labels.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Optional annotations.
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

/// Workspace specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSpec {
    /// Default values inherited by all agents.
    #[serde(default)]
    pub defaults: WorkspaceDefaults,
    /// Provider credentials and endpoints.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    /// Gateway server settings.
    #[serde(default)]
    pub gateway: GatewayConfig,
    /// Root directory for agent discovery.
    #[serde(default = "default_agents_dir")]
    pub agents_dir: String,
    /// Telemetry and observability settings (Phase 18).
    #[serde(default)]
    pub telemetry: Option<TelemetryConfig>,
    /// Security settings (Phase 19).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
}

/// Telemetry configuration for workspace-level observability settings.
///
/// Controls trace collection, OpenTelemetry export, and Prometheus metrics.
///
/// ```yaml
/// spec:
///   telemetry:
///     enabled: true
///     otlp_endpoint: "http://localhost:4318/v1/traces"
///     service_name: "my-workspace"
///     export_metrics: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry collection is enabled (default: true).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// OTLP HTTP endpoint for span export (e.g., "http://localhost:4318/v1/traces").
    /// When set, spans are pushed to this endpoint after each agent run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp_endpoint: Option<String>,
    /// Service name reported in OTel resource attributes (default: "agentix").
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// Whether to export Prometheus metrics at /metrics (default: true).
    #[serde(default = "default_true")]
    pub export_metrics: bool,
}

fn default_true() -> bool {
    true
}

fn default_service_name() -> String {
    "agentix".to_string()
}

fn default_agents_dir() -> String {
    "./agents".to_string()
}

/// Default values inherited by all agents in the workspace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceDefaults {
    /// Default LLM model (`provider/model` format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Maximum ReAct loop iterations (1–100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,
    /// Wall-clock timeout as a duration string (e.g. "5m", "30s", "1h").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// Execution mode: autonomous, semi-autonomous, or manual.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<AgentMode>,
}

/// Provider credential and endpoint configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key (supports `${ENV_VAR}` expansion).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL override (optional, for compatible APIs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// Gateway server settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Interface to bind (default: "127.0.0.1").
    #[serde(default = "default_gateway_host")]
    pub host: String,
    /// TCP port (default: 7777).
    #[serde(default = "default_gateway_port")]
    pub port: u16,
}

fn default_gateway_host() -> String {
    "127.0.0.1".to_string()
}

fn default_gateway_port() -> u16 {
    7777
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: default_gateway_host(),
            port: default_gateway_port(),
        }
    }
}

// ---------------------------------------------------------------------------
// WorkspaceConfig methods
// ---------------------------------------------------------------------------

impl WorkspaceConfig {
    /// Parse a `agentix.yaml` YAML string into a `WorkspaceConfig`.
    ///
    /// Uses `serde_path_to_error` to provide precise field-level error messages.
    pub fn from_yaml(content: &str) -> crate::AgentixResult<Self> {
        let de = serde_yaml::Deserializer::from_str(content);
        serde_path_to_error::deserialize(de).map_err(|e| {
            crate::AgentixError::yaml_parse(e.path().to_string(), e.inner().to_string())
        })
    }

    /// Expand `${VAR_NAME}` references in all provider `api_key` and `base_url` fields.
    ///
    /// Returns an error if a referenced environment variable is not set.
    pub fn expand_env_vars(&mut self) -> crate::AgentixResult<()> {
        for (provider_name, provider) in self.spec.providers.iter_mut() {
            if let Some(api_key) = &provider.api_key {
                provider.api_key = Some(expand_env_var_str(api_key, provider_name, "api_key")?);
            }
            if let Some(base_url) = &provider.base_url {
                provider.base_url =
                    Some(expand_env_var_str(base_url, provider_name, "base_url")?);
            }
        }
        Ok(())
    }
}

/// Expand a single `${VAR_NAME}` reference in a string value.
///
/// Only handles the simple case: the entire value is `${VAR_NAME}`.
/// Partial expansion (e.g. `prefix_${VAR}`) is also supported.
fn expand_env_var_str(
    value: &str,
    provider: &str,
    field: &str,
) -> crate::AgentixResult<String> {
    if !value.contains("${") {
        return Ok(value.to_string());
    }

    let mut result = value.to_string();
    // Find all ${VAR_NAME} patterns and replace
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            let env_val = std::env::var(var_name).map_err(|_| {
                crate::AgentixError::ConfigNotFound(format!(
                    "spec.providers.{provider}.{field}: environment variable \"{var_name}\" is not set"
                ))
            })?;
            result = format!("{}{}{}", &result[..start], env_val, &result[start + end + 1..]);
        } else {
            break;
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Legacy config parsing (AGENTS.md / TOOLS.md — retained for backward compat)
// ---------------------------------------------------------------------------

/// Agent configuration from `AGENTS.md` (legacy workspace format).
///
/// This is the v1.0 personality-driven format. New workspaces should use the
/// GitAgent directory format (`agent.yaml` + `SOUL.md`) instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    pub personality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub skills: Vec<String>,
}

/// Tool configuration from `TOOLS.md` (legacy workspace format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Container for agents list in AGENTS.md
#[derive(Debug, Deserialize)]
struct AgentsFile {
    agents: Vec<AgentConfig>,
}

/// Container for tools list in TOOLS.md
#[derive(Debug, Deserialize)]
struct ToolsFile {
    tools: Vec<ToolConfig>,
}

use std::path::Path;

/// Parse `AGENTS.md` file and return list of agent configs (legacy format).
pub fn parse_agents_md<P: AsRef<Path>>(path: P) -> crate::AofResult<Vec<AgentConfig>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| {
        crate::AofError::config(format!(
            "Failed to read AGENTS.md at {}: {}",
            path.display(),
            e
        ))
    })?;

    let deserializer = serde_yaml::Deserializer::from_str(&content);
    let agents_file: AgentsFile = serde_path_to_error::deserialize(deserializer).map_err(|e| {
        crate::AofError::config(format!(
            "Failed to parse AGENTS.md at {}\nField: {}\nError: {}",
            path.display(),
            e.path(),
            e.inner()
        ))
    })?;

    Ok(agents_file.agents)
}

/// Parse `TOOLS.md` file and return list of tool configs (legacy format).
pub fn parse_tools_md<P: AsRef<Path>>(path: P) -> crate::AofResult<Vec<ToolConfig>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| {
        crate::AofError::config(format!(
            "Failed to read TOOLS.md at {}: {}",
            path.display(),
            e
        ))
    })?;

    let deserializer = serde_yaml::Deserializer::from_str(&content);
    let tools_file: ToolsFile = serde_path_to_error::deserialize(deserializer).map_err(|e| {
        crate::AofError::config(format!(
            "Failed to parse TOOLS.md at {}\nField: {}\nError: {}",
            path.display(),
            e.path(),
            e.inner()
        ))
    })?;

    Ok(tools_file.tools)
}

/// Calculate version hash from configuration files.
///
/// Returns SHA256 hash of concatenated `AGENTS.md` + `TOOLS.md` content.
pub fn version_hash<P: AsRef<Path>>(agents_path: P, tools_path: P) -> crate::AofResult<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    if let Ok(content) = std::fs::read(agents_path.as_ref()) {
        hasher.update(&content);
    }
    if let Ok(content) = std::fs::read(tools_path.as_ref()) {
        hasher.update(&content);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_workspace_config_minimal_parse() {
        let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
  providers:
    anthropic:
      api_key: "sk-test"
"#;
        let config = WorkspaceConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.metadata.name, "test-workspace");
        assert_eq!(
            config.spec.defaults.model.as_deref(),
            Some("anthropic/claude-sonnet-4-6")
        );
        assert_eq!(config.spec.gateway.host, "127.0.0.1");
        assert_eq!(config.spec.gateway.port, 7777);
        assert_eq!(config.spec.agents_dir, "./agents");
    }

    #[test]
    fn test_parse_agents_md_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let mut file = std::fs::File::create(&agents_file).unwrap();
        write!(
            file,
            r#"
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    personality: Methodical and detail-oriented
    avatar: 🤖
    skills:
      - kubectl
      - pod-debugging
"#
        )
        .unwrap();

        let agents = parse_agents_md(&agents_file).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "k8s-monitor");
    }

    #[test]
    fn test_gateway_config_defaults() {
        let config = GatewayConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7777);
    }
}
