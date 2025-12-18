// AOF Core - FlowBinding resource type
//
// FlowBinding ties together a Trigger, Context, and Flow to create
// a complete routing configuration. This enables:
// - Decoupled architecture (define each resource once, compose with bindings)
// - Multi-tenant deployments (same flow, different contexts)
// - Flexible routing (multiple bindings per trigger)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FlowBinding - Ties trigger + context + flow together
///
/// Example:
/// ```yaml
/// apiVersion: aof.dev/v1
/// kind: FlowBinding
/// metadata:
///   name: prod-k8s-binding
/// spec:
///   trigger: slack-prod-channel    # Reference to Trigger
///   context: prod                   # Reference to Context
///   flow: k8s-ops-flow             # Reference to Flow
///   match:
///     patterns: ["kubectl", "k8s"]
///     priority: 100
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowBinding {
    /// API version (e.g., "aof.dev/v1")
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind, always "FlowBinding"
    #[serde(default = "default_binding_kind")]
    pub kind: String,

    /// Binding metadata
    pub metadata: FlowBindingMetadata,

    /// Binding specification
    pub spec: FlowBindingSpec,
}

fn default_api_version() -> String {
    "aof.dev/v1".to_string()
}

fn default_binding_kind() -> String {
    "FlowBinding".to_string()
}

/// FlowBinding metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowBindingMetadata {
    /// Binding name (unique identifier)
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

/// FlowBinding specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FlowBindingSpec {
    /// Reference to Trigger resource name
    pub trigger: String,

    /// Reference to Context resource name (optional - uses default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Reference to Flow resource name
    pub flow: String,

    /// Optional: Agent reference (for simple single-agent flows)
    /// If specified, creates an implicit single-step flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Optional: Fleet reference (for multi-agent coordination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet: Option<String>,

    /// Match configuration for this binding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#match: Option<BindingMatch>,

    /// Whether this binding is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Additional configuration overrides
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

/// Match configuration for routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BindingMatch {
    /// Message patterns to match (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,

    /// Priority for this binding (higher = more priority)
    #[serde(default)]
    pub priority: i32,

    /// Channel filter (override trigger's channels for this binding)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<String>,

    /// User filter (override trigger's users for this binding)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,

    /// Event filter (override trigger's events for this binding)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    /// Required: Message must contain all of these
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_keywords: Vec<String>,

    /// Excluded: Message must not contain any of these
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_keywords: Vec<String>,
}

impl FlowBinding {
    /// Get the binding name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the trigger reference
    pub fn trigger_ref(&self) -> &str {
        &self.spec.trigger
    }

    /// Get the context reference (if specified)
    pub fn context_ref(&self) -> Option<&str> {
        self.spec.context.as_deref()
    }

    /// Get the flow reference
    pub fn flow_ref(&self) -> &str {
        &self.spec.flow
    }

    /// Get agent reference (for simple bindings)
    pub fn agent_ref(&self) -> Option<&str> {
        self.spec.agent.as_deref()
    }

    /// Get fleet reference
    pub fn fleet_ref(&self) -> Option<&str> {
        self.spec.fleet.as_deref()
    }

    /// Validate the binding configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check name
        if self.metadata.name.is_empty() {
            return Err("FlowBinding name is required".to_string());
        }

        // Check trigger reference
        if self.spec.trigger.is_empty() {
            return Err("FlowBinding requires a trigger reference".to_string());
        }

        // Must have either flow, agent, or fleet
        if self.spec.flow.is_empty() && self.spec.agent.is_none() && self.spec.fleet.is_none() {
            return Err("FlowBinding requires flow, agent, or fleet reference".to_string());
        }

        Ok(())
    }

    /// Check if this binding matches the given message
    pub fn matches(&self, channel: Option<&str>, user: Option<&str>, text: Option<&str>) -> bool {
        if let Some(ref match_config) = self.spec.r#match {
            // Channel filter
            if !match_config.channels.is_empty() {
                if let Some(ch) = channel {
                    if !match_config.channels.iter().any(|c| c == ch) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // User filter
            if !match_config.users.is_empty() {
                if let Some(u) = user {
                    if !match_config.users.iter().any(|allowed| allowed == u) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Pattern filter
            if !match_config.patterns.is_empty() {
                if let Some(t) = text {
                    let matches_pattern = match_config.patterns.iter().any(|p| {
                        if let Ok(re) = regex::Regex::new(p) {
                            re.is_match(t)
                        } else {
                            t.to_lowercase().contains(&p.to_lowercase())
                        }
                    });
                    if !matches_pattern {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Required keywords
            if !match_config.required_keywords.is_empty() {
                if let Some(t) = text {
                    let text_lower = t.to_lowercase();
                    if !match_config.required_keywords.iter().all(|kw| text_lower.contains(&kw.to_lowercase())) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Excluded keywords
            if !match_config.excluded_keywords.is_empty() {
                if let Some(t) = text {
                    let text_lower = t.to_lowercase();
                    if match_config.excluded_keywords.iter().any(|kw| text_lower.contains(&kw.to_lowercase())) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Calculate a match score for routing priority
    /// Higher score = more specific match = higher priority
    pub fn match_score(&self, channel: Option<&str>, user: Option<&str>, text: Option<&str>) -> i32 {
        if !self.matches(channel, user, text) {
            return i32::MIN;
        }

        let mut score = 0i32;

        if let Some(ref match_config) = self.spec.r#match {
            // Explicit priority
            score += match_config.priority;

            // Channel specificity
            if !match_config.channels.is_empty() && channel.is_some() {
                score += 100;
            }

            // User specificity
            if !match_config.users.is_empty() && user.is_some() {
                score += 80;
            }

            // Pattern specificity
            if !match_config.patterns.is_empty() && text.is_some() {
                score += 60;
            }

            // Required keywords add specificity
            if !match_config.required_keywords.is_empty() {
                score += 40 * match_config.required_keywords.len() as i32;
            }
        }

        // Base score for having a binding at all
        score += 10;

        score
    }
}

/// Result of binding resolution
#[derive(Debug, Clone)]
pub struct ResolvedBinding {
    /// The binding that matched
    pub binding: FlowBinding,

    /// Match score
    pub score: i32,

    /// Trigger name
    pub trigger_name: String,

    /// Context name (if specified)
    pub context_name: Option<String>,

    /// Flow name
    pub flow_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flow_binding() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: prod-k8s-binding
  labels:
    environment: production
spec:
  trigger: slack-prod-channel
  context: prod
  flow: k8s-ops-flow
  match:
    patterns:
      - kubectl
      - k8s
    priority: 100
"#;

        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(binding.metadata.name, "prod-k8s-binding");
        assert_eq!(binding.spec.trigger, "slack-prod-channel");
        assert_eq!(binding.spec.context, Some("prod".to_string()));
        assert_eq!(binding.spec.flow, "k8s-ops-flow");
        assert!(binding.validate().is_ok());
    }

    #[test]
    fn test_parse_simple_binding() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: simple-binding
spec:
  trigger: telegram-oncall
  agent: incident-responder
  flow: incident-flow
"#;

        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(binding.spec.agent, Some("incident-responder".to_string()));
        assert!(binding.validate().is_ok());
    }

    #[test]
    fn test_parse_binding_with_match() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: k8s-only-binding
spec:
  trigger: slack-prod
  context: prod
  flow: k8s-ops-flow
  match:
    patterns:
      - "^kubectl"
      - "^k8s"
    channels:
      - production
    required_keywords:
      - pod
    excluded_keywords:
      - delete
    priority: 200
"#;

        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();
        let match_config = binding.spec.r#match.as_ref().unwrap();
        assert_eq!(match_config.patterns.len(), 2);
        assert_eq!(match_config.channels.len(), 1);
        assert_eq!(match_config.required_keywords.len(), 1);
        assert_eq!(match_config.excluded_keywords.len(), 1);
        assert_eq!(match_config.priority, 200);
    }

    #[test]
    fn test_validation_errors() {
        // Empty name
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: ""
spec:
  trigger: test
  flow: test
"#;
        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();
        assert!(binding.validate().is_err());

        // Missing trigger
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: test
spec:
  trigger: ""
  flow: test
"#;
        let binding2: FlowBinding = serde_yaml::from_str(yaml2).unwrap();
        assert!(binding2.validate().is_err());

        // Missing flow/agent/fleet
        let yaml3 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: test
spec:
  trigger: test
  flow: ""
"#;
        let binding3: FlowBinding = serde_yaml::from_str(yaml3).unwrap();
        assert!(binding3.validate().is_err());
    }

    #[test]
    fn test_matches() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: test
spec:
  trigger: test
  flow: test-flow
  match:
    patterns:
      - kubectl
    channels:
      - production
    required_keywords:
      - pod
    excluded_keywords:
      - delete
"#;

        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();

        // Matches
        assert!(binding.matches(Some("production"), None, Some("kubectl get pod")));

        // Wrong channel
        assert!(!binding.matches(Some("staging"), None, Some("kubectl get pod")));

        // Missing required keyword
        assert!(!binding.matches(Some("production"), None, Some("kubectl get deployment")));

        // Contains excluded keyword
        assert!(!binding.matches(Some("production"), None, Some("kubectl delete pod")));
    }

    #[test]
    fn test_match_score() {
        // Binding with high specificity
        let yaml1 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: specific
spec:
  trigger: test
  flow: test
  match:
    patterns: [kubectl]
    channels: [production]
    priority: 50
"#;

        // Binding with low specificity
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: catchall
spec:
  trigger: test
  flow: test
"#;

        let specific: FlowBinding = serde_yaml::from_str(yaml1).unwrap();
        let catchall: FlowBinding = serde_yaml::from_str(yaml2).unwrap();

        let score1 = specific.match_score(Some("production"), None, Some("kubectl get pods"));
        let score2 = catchall.match_score(Some("production"), None, Some("kubectl get pods"));

        // More specific binding should have higher score
        assert!(score1 > score2);
    }

    #[test]
    fn test_disabled_binding() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: disabled
spec:
  trigger: test
  flow: test
  enabled: false
"#;

        let binding: FlowBinding = serde_yaml::from_str(yaml).unwrap();
        assert!(!binding.spec.enabled);
    }
}
