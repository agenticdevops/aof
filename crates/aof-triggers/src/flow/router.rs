//! FlowRouter - Routes incoming trigger events to matching AgentFlows
//!
//! The FlowRouter handles:
//! - Matching events to flows based on platform, channel, user, patterns
//! - Priority-based routing when multiple flows match
//! - Default flow fallback

use std::sync::Arc;

use aof_core::{AgentFlow, TriggerType};
use regex::Regex;
use tracing::{debug, info, warn};

use super::registry::FlowRegistry;
use crate::TriggerMessage;

/// Match result from routing
#[derive(Debug, Clone)]
pub struct FlowMatch {
    /// The matched flow
    pub flow: Arc<AgentFlow>,

    /// Match score (higher = better match)
    pub score: u32,

    /// Why this flow matched
    pub reason: MatchReason,
}

/// Reason for flow match
#[derive(Debug, Clone)]
pub enum MatchReason {
    /// Matched by channel filter
    Channel(String),

    /// Matched by user filter
    User(String),

    /// Matched by message pattern
    Pattern(String),

    /// Matched by platform only (no specific filters)
    PlatformDefault,

    /// Matched as explicit default flow
    ExplicitDefault,
}

/// FlowRouter routes incoming events to AgentFlows
pub struct FlowRouter {
    /// Flow registry
    registry: Arc<FlowRegistry>,

    /// Default flow name (fallback when no match)
    default_flow: Option<String>,
}

impl FlowRouter {
    /// Create a new router with a registry
    pub fn new(registry: Arc<FlowRegistry>) -> Self {
        Self {
            registry,
            default_flow: None,
        }
    }

    /// Create router with a default flow
    pub fn with_default(registry: Arc<FlowRegistry>, default_flow: impl Into<String>) -> Self {
        Self {
            registry,
            default_flow: Some(default_flow.into()),
        }
    }

    /// Set the default flow
    pub fn set_default(&mut self, flow_name: impl Into<String>) {
        self.default_flow = Some(flow_name.into());
    }

    /// Route a trigger message to matching flows
    pub fn route(&self, platform: &str, message: &TriggerMessage) -> Vec<FlowMatch> {
        let mut matches = Vec::new();

        // Get all flows for this platform
        let platform_flows = self.registry.by_platform(platform);

        for flow in platform_flows {
            if let Some(flow_match) = self.check_flow_match(&flow, message) {
                matches.push(flow_match);
            }
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.cmp(&a.score));

        // If no matches and we have a default, add it
        if matches.is_empty() {
            if let Some(ref default_name) = self.default_flow {
                if let Some(default_flow) = self.registry.get(default_name) {
                    matches.push(FlowMatch {
                        flow: default_flow,
                        score: 0,
                        reason: MatchReason::ExplicitDefault,
                    });
                }
            }
        }

        matches
    }

    /// Get the best matching flow for a message
    pub fn route_best(&self, platform: &str, message: &TriggerMessage) -> Option<FlowMatch> {
        self.route(platform, message).into_iter().next()
    }

    /// Check if a flow matches a message
    fn check_flow_match(&self, flow: &Arc<AgentFlow>, message: &TriggerMessage) -> Option<FlowMatch> {
        let trigger = &flow.spec.trigger;
        let config = &trigger.config;

        let mut score = 0u32;
        let mut matched_reason: Option<MatchReason> = None;

        // Check channel filter
        if !config.channels.is_empty() {
            let channel_match = config.channels.iter().any(|ch| {
                // Support exact match or wildcard
                if ch == "*" {
                    true
                } else if ch.starts_with('#') {
                    // Slack channel name format
                    message.channel_id == ch[1..] ||
                    message.metadata.get("channel_name")
                        .and_then(|v| v.as_str())
                        .map(|name| name == &ch[1..])
                        .unwrap_or(false)
                } else {
                    message.channel_id == *ch
                }
            });

            if !channel_match {
                return None; // Channel filter doesn't match
            }

            score += 100;
            matched_reason = Some(MatchReason::Channel(message.channel_id.clone()));
        }

        // Check user filter
        if !config.users.is_empty() {
            let user_match = config.users.iter().any(|u| {
                if u == "*" {
                    true
                } else {
                    message.user_id == *u || message.user_name == *u
                }
            });

            if !user_match {
                return None; // User filter doesn't match
            }

            score += 50;
            if matched_reason.is_none() {
                matched_reason = Some(MatchReason::User(message.user_id.clone()));
            }
        }

        // Check message patterns
        if !config.patterns.is_empty() {
            let pattern_match = config.patterns.iter().find_map(|pattern| {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if re.is_match(&message.text) {
                            Some(pattern.clone())
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        warn!("Invalid pattern '{}' in flow {}: {}", pattern, flow.metadata.name, e);
                        None
                    }
                }
            });

            if let Some(matched_pattern) = pattern_match {
                score += 200; // Pattern matches get highest priority
                matched_reason = Some(MatchReason::Pattern(matched_pattern));
            } else if !config.patterns.is_empty() {
                return None; // Pattern filter doesn't match
            }
        }

        // If no specific filters, this is a platform-level match
        if config.channels.is_empty() && config.users.is_empty() && config.patterns.is_empty() {
            score = 1; // Low priority for platform-only match
            matched_reason = Some(MatchReason::PlatformDefault);
        }

        matched_reason.map(|reason| FlowMatch {
            flow: flow.clone(),
            score,
            reason,
        })
    }

    /// Get the registry
    pub fn registry(&self) -> &Arc<FlowRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    fn create_test_message(channel: &str, user: &str, text: &str) -> TriggerMessage {
        TriggerMessage {
            message_id: "test-123".to_string(),
            user_id: user.to_string(),
            user_name: format!("User {}", user),
            channel_id: channel.to_string(),
            text: text.to_string(),
            thread_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_router_platform_match() {
        let registry = Arc::new(FlowRegistry::new());

        let flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-bot
spec:
  trigger:
    type: Slack
  nodes:
    - id: process
      type: End
"#).unwrap();

        registry.register(flow);

        let router = FlowRouter::new(registry);
        let message = create_test_message("C123", "U456", "hello");

        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].flow.metadata.name, "slack-bot");
    }

    #[test]
    fn test_router_channel_filter() {
        let registry = Arc::new(FlowRegistry::new());

        let flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: prod-channel-bot
spec:
  trigger:
    type: Slack
    config:
      channels:
        - C123-prod
  nodes:
    - id: process
      type: End
"#).unwrap();

        registry.register(flow);

        let router = FlowRouter::new(registry);

        // Should match
        let message = create_test_message("C123-prod", "U456", "hello");
        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 1);

        // Should not match
        let message = create_test_message("C999-other", "U456", "hello");
        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_router_pattern_filter() {
        let registry = Arc::new(FlowRegistry::new());

        let flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: kubectl-bot
spec:
  trigger:
    type: Slack
    config:
      patterns:
        - "kubectl.*"
        - "k8s.*"
  nodes:
    - id: process
      type: End
"#).unwrap();

        registry.register(flow);

        let router = FlowRouter::new(registry);

        // Should match kubectl
        let message = create_test_message("C123", "U456", "kubectl get pods");
        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 1);

        // Should match k8s
        let message = create_test_message("C123", "U456", "k8s status please");
        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 1);

        // Should not match
        let message = create_test_message("C123", "U456", "hello world");
        let matches = router.route("slack", &message);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_router_default_flow() {
        let registry = Arc::new(FlowRegistry::new());

        // Default flow (no filters)
        let default_flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: default-bot
spec:
  trigger:
    type: Slack
  nodes:
    - id: process
      type: End
"#).unwrap();

        // Specific flow (with filter)
        let specific_flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: prod-bot
spec:
  trigger:
    type: Slack
    config:
      channels:
        - C-prod
  nodes:
    - id: process
      type: End
"#).unwrap();

        registry.register(default_flow);
        registry.register(specific_flow);

        let router = FlowRouter::with_default(registry, "default-bot");

        // Message to prod channel should match prod-bot
        let message = create_test_message("C-prod", "U456", "hello");
        let best = router.route_best("slack", &message);
        assert!(best.is_some());
        assert_eq!(best.unwrap().flow.metadata.name, "prod-bot");

        // Message to other channel should fall back to default
        let message = create_test_message("C-other", "U456", "hello");
        let best = router.route_best("slack", &message);
        assert!(best.is_some());
        assert_eq!(best.unwrap().flow.metadata.name, "default-bot");
    }
}
