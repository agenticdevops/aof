// AOF Core - Context resource type
//
// Context represents an execution environment boundary with:
// - Cluster configuration (kubeconfig, namespace)
// - Environment variables
// - Approval requirements
// - Audit configuration
// - Rate limits
//
// Contexts are injected at runtime via FlowBinding or CLI --context flag,
// following Kubernetes-style context injection patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context - Execution environment boundary
///
/// Example:
/// ```yaml
/// apiVersion: aof.dev/v1
/// kind: Context
/// metadata:
///   name: prod
/// spec:
///   kubeconfig: ${KUBECONFIG_PROD}
///   namespace: production
///   env:
///     CLUSTER_NAME: prod-us-east-1
///     LOG_LEVEL: info
///   approval:
///     required: true
///     allowed_users: [U015ADMIN, U016SRELEAD]
///     timeout_seconds: 300
///   audit:
///     enabled: true
///     sink: s3://company-audit/prod/
///   limits:
///     max_requests_per_minute: 100
///     max_tokens_per_day: 1000000
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// API version (e.g., "aof.dev/v1")
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind, always "Context"
    #[serde(default = "default_context_kind")]
    pub kind: String,

    /// Context metadata
    pub metadata: ContextMetadata,

    /// Context specification
    pub spec: ContextSpec,
}

fn default_api_version() -> String {
    "aof.dev/v1".to_string()
}

fn default_context_kind() -> String {
    "Context".to_string()
}

/// Context metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Context name (unique identifier)
    pub name: String,

    /// Namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Labels for categorization
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Annotations for additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

/// Context specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSpec {
    /// Kubeconfig file path (supports ${ENV_VAR} expansion)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubeconfig: Option<String>,

    /// Kubernetes namespace for operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Kubernetes cluster name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,

    /// Environment variables available to agents in this context
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Working directory for tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Approval configuration for this context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<ApprovalConfig>,

    /// Audit configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,

    /// Rate limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<LimitsConfig>,

    /// Secret references (for credentials)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<SecretRef>,

    /// Additional configuration
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Approval configuration for a context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApprovalConfig {
    /// Whether approval is required for destructive operations
    #[serde(default)]
    pub required: bool,

    /// List of users allowed to approve (platform-specific IDs)
    /// Supports formats: "U12345678", "slack:U12345678", "email:user@company.com"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_users: Vec<String>,

    /// Timeout for approval in seconds
    #[serde(default = "default_approval_timeout")]
    pub timeout_seconds: u32,

    /// Patterns that require approval (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub require_for: Vec<String>,

    /// Whether to allow self-approval
    #[serde(default)]
    pub allow_self_approval: bool,

    /// Minimum number of approvers required
    #[serde(default = "default_min_approvers")]
    pub min_approvers: u32,
}

fn default_approval_timeout() -> u32 {
    300 // 5 minutes
}

fn default_min_approvers() -> u32 {
    1
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            required: false,
            allowed_users: Vec::new(),
            timeout_seconds: default_approval_timeout(),
            require_for: Vec::new(),
            allow_self_approval: false,
            min_approvers: default_min_approvers(),
        }
    }
}

/// Audit configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Audit sink URL (s3://, file://, http://)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sink: Option<String>,

    /// Events to audit
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<AuditEvent>,

    /// Include full request/response in audit log
    #[serde(default)]
    pub include_payload: bool,

    /// Retention period (e.g., "30d", "1y")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention: Option<String>,
}

/// Types of events to audit
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEvent {
    /// Agent execution started
    AgentStart,
    /// Agent execution completed
    AgentComplete,
    /// Tool invocation
    ToolCall,
    /// Approval requested
    ApprovalRequested,
    /// Approval granted
    ApprovalGranted,
    /// Approval denied
    ApprovalDenied,
    /// Error occurred
    Error,
    /// All events
    All,
}

/// Rate limits configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LimitsConfig {
    /// Maximum requests per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_requests_per_minute: Option<u32>,

    /// Maximum tokens per day
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_day: Option<u64>,

    /// Maximum concurrent executions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent: Option<u32>,

    /// Maximum execution time per request (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_execution_time_seconds: Option<u32>,

    /// Cost limit per day (in credits/cents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_day: Option<f64>,
}

/// Secret reference for credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretRef {
    /// Secret name
    pub name: String,

    /// Secret key (if referencing a specific key in a secret)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Environment variable to set with secret value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,
}

impl Context {
    /// Get the context name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Validate the context configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check name
        if self.metadata.name.is_empty() {
            return Err("Context name is required".to_string());
        }

        // Validate approval config if present
        if let Some(ref approval) = self.spec.approval {
            if approval.required && approval.allowed_users.is_empty() {
                // Warning but not error - might be intentional (no one can approve)
            }
            if approval.min_approvers < 1 {
                return Err("min_approvers must be at least 1".to_string());
            }
        }

        // Validate limits
        if let Some(ref limits) = self.spec.limits {
            if let Some(max_concurrent) = limits.max_concurrent {
                if max_concurrent == 0 {
                    return Err("max_concurrent must be greater than 0".to_string());
                }
            }
        }

        Ok(())
    }

    /// Expand environment variables in configuration values
    pub fn expand_env_vars(&mut self) {
        // Expand kubeconfig
        if let Some(ref kubeconfig) = self.spec.kubeconfig {
            self.spec.kubeconfig = Some(expand_env_var(kubeconfig));
        }

        // Expand env values
        let expanded_env: HashMap<String, String> = self
            .spec
            .env
            .iter()
            .map(|(k, v)| (k.clone(), expand_env_var(v)))
            .collect();
        self.spec.env = expanded_env;

        // Expand working_dir
        if let Some(ref working_dir) = self.spec.working_dir {
            self.spec.working_dir = Some(expand_env_var(working_dir));
        }

        // Expand audit sink
        if let Some(ref mut audit) = self.spec.audit {
            if let Some(ref sink) = audit.sink {
                audit.sink = Some(expand_env_var(sink));
            }
        }
    }

    /// Get all environment variables (including from spec.env)
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.spec.env.clone();

        // Add context-specific vars
        env.insert("AOF_CONTEXT".to_string(), self.metadata.name.clone());

        if let Some(ref namespace) = self.spec.namespace {
            env.insert("AOF_NAMESPACE".to_string(), namespace.clone());
        }

        if let Some(ref cluster) = self.spec.cluster {
            env.insert("AOF_CLUSTER".to_string(), cluster.clone());
        }

        env
    }

    /// Check if approval is required for a given command
    pub fn requires_approval(&self, command: &str) -> bool {
        if let Some(ref approval) = self.spec.approval {
            if !approval.required {
                return false;
            }

            // If no patterns specified, require approval for everything
            if approval.require_for.is_empty() {
                return true;
            }

            // Check if command matches any pattern
            for pattern in &approval.require_for {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(command) {
                        return true;
                    }
                } else if command.contains(pattern) {
                    return true;
                }
            }

            false
        } else {
            false
        }
    }

    /// Check if a user is allowed to approve
    pub fn is_approver(&self, user_id: &str) -> bool {
        if let Some(ref approval) = self.spec.approval {
            if approval.allowed_users.is_empty() {
                return true; // Anyone can approve if no whitelist
            }

            // Check various formats - both directions
            approval.allowed_users.iter().any(|allowed| {
                // Direct match
                allowed == user_id
                    // User ID with prefix in whitelist
                    || allowed == &format!("slack:{}", user_id)
                    || allowed == &format!("telegram:{}", user_id)
                    || allowed == &format!("discord:{}", user_id)
                    // Whitelist entry has prefix, strip and compare
                    || allowed.strip_prefix("slack:").map_or(false, |id| id == user_id)
                    || allowed.strip_prefix("telegram:").map_or(false, |id| id == user_id)
                    || allowed.strip_prefix("discord:").map_or(false, |id| id == user_id)
            })
        } else {
            true // No approval config = anyone can approve
        }
    }
}

/// Expand ${VAR_NAME} patterns in a string
fn expand_env_var(value: &str) -> String {
    let mut result = value.to_string();
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();

    for cap in re.captures_iter(value) {
        let var_name = &cap[1];
        if let Ok(var_value) = std::env::var(var_name) {
            result = result.replace(&cap[0], &var_value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_context() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  labels:
    environment: production
spec:
  kubeconfig: ${KUBECONFIG_PROD}
  namespace: production
  env:
    CLUSTER_NAME: prod-us-east-1
    LOG_LEVEL: info
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ctx.metadata.name, "prod");
        assert_eq!(ctx.spec.namespace, Some("production".to_string()));
        assert_eq!(ctx.spec.env.get("CLUSTER_NAME"), Some(&"prod-us-east-1".to_string()));
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn test_parse_context_with_approval() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
  approval:
    required: true
    allowed_users:
      - U015ADMIN
      - slack:U016SRELEAD
    timeout_seconds: 300
    require_for:
      - kubectl delete
      - helm uninstall
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.spec.approval.is_some());
        let approval = ctx.spec.approval.as_ref().unwrap();
        assert!(approval.required);
        assert_eq!(approval.allowed_users.len(), 2);
        assert_eq!(approval.timeout_seconds, 300);
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn test_parse_context_with_audit() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  audit:
    enabled: true
    sink: s3://company-audit/prod/
    events:
      - agent_start
      - agent_complete
      - tool_call
    include_payload: false
    retention: "90d"
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.spec.audit.is_some());
        let audit = ctx.spec.audit.as_ref().unwrap();
        assert!(audit.enabled);
        assert_eq!(audit.sink, Some("s3://company-audit/prod/".to_string()));
        assert_eq!(audit.events.len(), 3);
    }

    #[test]
    fn test_parse_context_with_limits() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: staging
spec:
  limits:
    max_requests_per_minute: 100
    max_tokens_per_day: 1000000
    max_concurrent: 5
    max_execution_time_seconds: 300
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.spec.limits.is_some());
        let limits = ctx.spec.limits.as_ref().unwrap();
        assert_eq!(limits.max_requests_per_minute, Some(100));
        assert_eq!(limits.max_tokens_per_day, Some(1000000));
        assert_eq!(limits.max_concurrent, Some(5));
    }

    #[test]
    fn test_requires_approval() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  approval:
    required: true
    require_for:
      - "kubectl delete"
      - "helm uninstall"
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.requires_approval("kubectl delete pod nginx"));
        assert!(ctx.requires_approval("helm uninstall my-release"));
        assert!(!ctx.requires_approval("kubectl get pods"));
    }

    #[test]
    fn test_is_approver() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  approval:
    required: true
    allowed_users:
      - U015ADMIN
      - slack:U016SRELEAD
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.is_approver("U015ADMIN"));
        assert!(ctx.is_approver("U016SRELEAD"));
        assert!(!ctx.is_approver("U999RANDOM"));
    }

    #[test]
    fn test_get_env_vars() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
  cluster: prod-cluster
  env:
    CUSTOM_VAR: custom_value
"#;

        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        let env = ctx.get_env_vars();

        assert_eq!(env.get("AOF_CONTEXT"), Some(&"prod".to_string()));
        assert_eq!(env.get("AOF_NAMESPACE"), Some(&"production".to_string()));
        assert_eq!(env.get("AOF_CLUSTER"), Some(&"prod-cluster".to_string()));
        assert_eq!(env.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
    }

    #[test]
    fn test_validation_errors() {
        // Empty name
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: ""
spec: {}
"#;
        let ctx: Context = serde_yaml::from_str(yaml).unwrap();
        assert!(ctx.validate().is_err());

        // Invalid min_approvers
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: test
spec:
  approval:
    required: true
    min_approvers: 0
"#;
        let ctx2: Context = serde_yaml::from_str(yaml2).unwrap();
        assert!(ctx2.validate().is_err());
    }

    #[test]
    fn test_expand_env_var() {
        std::env::set_var("TEST_VAR", "test_value");
        let result = expand_env_var("prefix_${TEST_VAR}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");
        std::env::remove_var("TEST_VAR");
    }
}
