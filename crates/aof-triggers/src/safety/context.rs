//! Safety Context - Combines classification and policy for a request
//!
//! The SafetyContext brings together:
//! - Tool classification
//! - Platform policy
//! - User context
//! - Namespace/resource context

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::classifier::{ActionClass, ClassificationResult, ToolClassifier, ToolClassifications};
use super::policy::{PlatformPolicy, PolicyDecision, PolicyEngine};

/// Complete safety configuration from a Context YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// Kind
    pub kind: String,
    /// Metadata
    pub metadata: ContextMetadata,
    /// Specification
    pub spec: ContextSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub name: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSpec {
    /// Default namespace for operations
    #[serde(default)]
    pub namespace: String,
    /// Cluster name
    #[serde(default)]
    pub cluster: String,
    /// Default agent for this context
    #[serde(default)]
    pub default_agent: Option<String>,
    /// Platform-specific policies
    #[serde(default)]
    pub platform_policies: HashMap<String, PlatformPolicy>,
    /// Users allowed to approve operations
    #[serde(default)]
    pub approval_allowed_users: Vec<String>,
    /// Additional safety settings
    #[serde(default)]
    pub safety: SafetySettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetySettings {
    /// Namespaces requiring confirmation for operations
    #[serde(default)]
    pub require_confirmation_for_namespace: Vec<String>,
    /// Maximum resources affected per operation
    #[serde(default = "default_max_resources")]
    pub max_resources_per_operation: u32,
    /// Whether to audit all operations
    #[serde(default)]
    pub audit_all_operations: bool,
}

fn default_max_resources() -> u32 {
    10
}

/// Result of safety evaluation
#[derive(Debug, Clone)]
pub struct SafetyEvaluation {
    /// The classification result
    pub classification: ClassificationResult,
    /// The policy decision
    pub decision: PolicyDecision,
    /// Whether namespace confirmation is required
    pub requires_namespace_confirmation: bool,
    /// User-friendly message about the decision
    pub message: String,
}

impl SafetyEvaluation {
    /// Check if the operation is allowed
    pub fn is_allowed(&self) -> bool {
        self.decision.is_allowed() && !self.requires_namespace_confirmation
    }

    /// Check if approval is needed
    pub fn needs_approval(&self) -> bool {
        self.decision.requires_approval() || self.requires_namespace_confirmation
    }

    /// Check if the operation is blocked
    pub fn is_blocked(&self) -> bool {
        self.decision.is_blocked()
    }

    /// Get a formatted message for Telegram/Slack
    pub fn format_for_platform(&self, platform: &str) -> String {
        if self.is_allowed() {
            return String::new();
        }

        let icon = match &self.decision {
            PolicyDecision::Block { .. } => "🚫",
            PolicyDecision::RequireApproval { .. } => "⚠️",
            PolicyDecision::Allow => "✅",
        };

        match platform.to_lowercase().as_str() {
            "telegram" => format!("{} {}", icon, self.message),
            "slack" => format!("{} *{}*", icon, self.message),
            "whatsapp" => format!("{} {}", icon, self.message),
            _ => format!("{} {}", icon, self.message),
        }
    }
}

/// Safety context that evaluates commands against policies
pub struct SafetyContext {
    /// Tool classifier
    classifier: ToolClassifier,
    /// Policy engine
    policy_engine: PolicyEngine,
    /// Safety settings
    settings: SafetySettings,
    /// Context name
    name: String,
}

impl SafetyContext {
    /// Create a new safety context with defaults
    pub fn new(name: &str) -> Self {
        Self {
            classifier: ToolClassifier::new(),
            policy_engine: PolicyEngine::new(),
            settings: SafetySettings::default(),
            name: name.to_string(),
        }
    }

    /// Load from a SafetyConfig
    pub fn from_config(config: SafetyConfig) -> Self {
        let mut policy_engine = PolicyEngine::new();

        // Load platform policies
        for (platform, policy) in config.spec.platform_policies {
            policy_engine.set_policy(&platform, policy);
        }

        // Set approval users
        policy_engine.set_approval_users(config.spec.approval_allowed_users);

        Self {
            classifier: ToolClassifier::new(),
            policy_engine,
            settings: config.spec.safety,
            name: config.metadata.name,
        }
    }

    /// Load from YAML file
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: SafetyConfig = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(Self::from_config(config))
    }

    /// Load tool classifications
    pub fn load_classifications(&mut self, config: ToolClassifications) {
        self.classifier = ToolClassifier::from_config(config);
    }

    /// Load classifications from YAML file
    pub fn load_classifications_from_file(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read classifications file: {}", e))?;

        let config: ToolClassifications = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse classifications: {}", e))?;

        self.classifier = ToolClassifier::from_config(config);
        Ok(())
    }

    /// Set policy for a platform
    pub fn set_platform_policy(&mut self, platform: &str, policy: PlatformPolicy) {
        self.policy_engine.set_policy(platform, policy);
    }

    /// Get context name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Evaluate a command for a specific platform and user
    pub fn evaluate(
        &self,
        command: &str,
        platform: &str,
        user_id: &str,
        namespace: Option<&str>,
    ) -> SafetyEvaluation {
        // Classify the command
        let classification = self.classifier.classify(command);

        // Get policy decision
        let decision = self.policy_engine.evaluate_with_context(
            platform,
            classification.class,
            user_id,
            namespace,
        );

        // Check namespace confirmation requirement
        let requires_namespace_confirmation = namespace
            .map(|ns| self.settings.require_confirmation_for_namespace.contains(&ns.to_string()))
            .unwrap_or(false);

        // Build message
        let message = self.build_message(&classification, &decision, platform, namespace);

        SafetyEvaluation {
            classification,
            decision,
            requires_namespace_confirmation,
            message,
        }
    }

    /// Build a human-readable message for the evaluation
    fn build_message(
        &self,
        classification: &ClassificationResult,
        decision: &PolicyDecision,
        platform: &str,
        namespace: Option<&str>,
    ) -> String {
        match decision {
            PolicyDecision::Allow => {
                format!(
                    "Allowed: {} {} operation on {}",
                    classification.tool,
                    classification.class,
                    platform
                )
            }
            PolicyDecision::RequireApproval { reason, timeout_minutes } => {
                format!(
                    "{}\n\nApproval required within {} minutes.",
                    reason,
                    timeout_minutes
                )
            }
            PolicyDecision::Block { reason, suggestion } => {
                let mut msg = reason.clone();
                if let Some(sug) = suggestion {
                    msg.push_str("\n\n");
                    msg.push_str(sug);
                }
                if let Some(ns) = namespace {
                    msg.push_str(&format!("\n\nNamespace: {}", ns));
                }
                msg
            }
        }
    }

    /// Check if a user can approve operations
    pub fn can_user_approve(&self, user_id: &str) -> bool {
        self.policy_engine.can_user_approve(user_id)
    }

    /// Get the underlying classifier
    pub fn classifier(&self) -> &ToolClassifier {
        &self.classifier
    }

    /// Get the underlying policy engine
    pub fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }
}

impl Default for SafetyContext {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_context_default() {
        let ctx = SafetyContext::new("test");

        // Default: Telegram is read-only
        let eval = ctx.evaluate("kubectl get pods", "telegram", "user1", None);
        assert!(eval.is_allowed());

        let eval = ctx.evaluate("kubectl delete pod foo", "telegram", "user1", None);
        assert!(eval.is_blocked());

        // CLI is permissive
        let eval = ctx.evaluate("kubectl delete pod foo", "cli", "user1", None);
        assert!(eval.is_allowed());
    }

    #[test]
    fn test_safety_context_slack_approval() {
        let ctx = SafetyContext::new("test");

        // Slack requires approval for writes
        let eval = ctx.evaluate("kubectl apply -f deployment.yaml", "slack", "user1", None);
        assert!(eval.needs_approval());

        // Read is allowed
        let eval = ctx.evaluate("kubectl get pods", "slack", "user1", None);
        assert!(eval.is_allowed());
    }

    #[test]
    fn test_namespace_confirmation() {
        let config = SafetyConfig {
            api_version: "aof.dev/v1".to_string(),
            kind: "Context".to_string(),
            metadata: ContextMetadata {
                name: "prod".to_string(),
                labels: HashMap::new(),
            },
            spec: ContextSpec {
                namespace: "production".to_string(),
                cluster: "prod-cluster".to_string(),
                default_agent: None,
                platform_policies: HashMap::new(),
                approval_allowed_users: vec![],
                safety: SafetySettings {
                    require_confirmation_for_namespace: vec!["production".to_string()],
                    max_resources_per_operation: 10,
                    audit_all_operations: true,
                },
            },
        };

        let ctx = SafetyContext::from_config(config);

        // Operations in production namespace need confirmation
        let eval = ctx.evaluate("kubectl get pods", "cli", "user1", Some("production"));
        assert!(eval.requires_namespace_confirmation);

        // Other namespaces are fine
        let eval = ctx.evaluate("kubectl get pods", "cli", "user1", Some("default"));
        assert!(!eval.requires_namespace_confirmation);
    }

    #[test]
    fn test_evaluation_message_format() {
        let ctx = SafetyContext::new("test");

        let eval = ctx.evaluate("kubectl delete pod foo", "telegram", "user1", None);
        let msg = eval.format_for_platform("telegram");
        assert!(msg.contains("🚫"));

        let msg = eval.format_for_platform("slack");
        assert!(msg.contains("🚫"));
        assert!(msg.contains("*")); // Slack markdown
    }
}
