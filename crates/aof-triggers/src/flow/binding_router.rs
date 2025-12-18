//! BindingRouter - Routes events using FlowBinding resources
//!
//! This router uses the composable architecture where:
//! - Triggers are standalone resources
//! - Contexts define environment boundaries
//! - FlowBindings tie triggers, contexts, and flows together
//!
//! This enables multi-tenant deployments and flexible routing.

use aof_core::{
    AgentFlow, Context, FlowBinding, Trigger,
    registry::{
        BindingRegistry, ContextRegistry, FlowRegistry, TriggerRegistry,
        Registry,
    },
};
use tracing::{info, warn};

use crate::TriggerMessage;

/// Result of binding-based routing
#[derive(Debug, Clone)]
pub struct BindingMatch {
    /// The matched binding
    pub binding: FlowBinding,

    /// The resolved flow
    pub flow: AgentFlow,

    /// The resolved context (if specified)
    pub context: Option<Context>,

    /// The resolved trigger
    pub trigger: Trigger,

    /// Match score (higher = better match)
    pub score: i32,
}

/// BindingRouter routes events using FlowBinding resources
///
/// This router supports the composable multi-tenant architecture where
/// resources are decoupled and composed via bindings.
pub struct BindingRouter {
    /// Binding registry
    bindings: BindingRegistry,

    /// Flow registry
    flows: FlowRegistry,

    /// Context registry
    contexts: ContextRegistry,

    /// Trigger registry
    triggers: TriggerRegistry,

    /// Default context name (used when binding doesn't specify one)
    default_context: Option<String>,
}

impl BindingRouter {
    /// Create a new binding router with registries
    pub fn new(
        bindings: BindingRegistry,
        flows: FlowRegistry,
        contexts: ContextRegistry,
        triggers: TriggerRegistry,
    ) -> Self {
        Self {
            bindings,
            flows,
            contexts,
            triggers,
            default_context: None,
        }
    }

    /// Set the default context for bindings that don't specify one
    pub fn with_default_context(mut self, context_name: impl Into<String>) -> Self {
        self.default_context = Some(context_name.into());
        self
    }

    /// Set default context (mutable)
    pub fn set_default_context(&mut self, context_name: impl Into<String>) {
        self.default_context = Some(context_name.into());
    }

    /// Route a message to matching bindings
    ///
    /// Returns all matching bindings sorted by score (highest first).
    pub fn route(
        &self,
        platform: &str,
        message: &TriggerMessage,
    ) -> Vec<BindingMatch> {
        let mut matches = Vec::new();

        // Find all bindings that reference triggers for this platform
        for binding in self.bindings.get_all() {
            // Skip disabled bindings
            if !binding.spec.enabled {
                continue;
            }

            // Get the trigger referenced by this binding
            let trigger = match self.triggers.get(binding.trigger_ref()) {
                Some(t) => t,
                None => {
                    warn!(
                        "Binding '{}' references non-existent trigger '{}'",
                        binding.name(),
                        binding.trigger_ref()
                    );
                    continue;
                }
            };

            // Check trigger-level filtering (includes platform check)
            if !trigger.matches(
                platform,
                Some(&message.channel_id),
                Some(&message.user_id),
                Some(&message.text),
            ) {
                continue;
            }

            // Check binding-level filtering
            if !binding.matches(
                Some(&message.channel_id),
                Some(&message.user_id),
                Some(&message.text),
            ) {
                continue;
            }

            // Get the flow
            let flow = match self.flows.get(binding.flow_ref()) {
                Some(f) => f,
                None => {
                    warn!(
                        "Binding '{}' references non-existent flow '{}'",
                        binding.name(),
                        binding.flow_ref()
                    );
                    continue;
                }
            };

            // Get context (if specified)
            let context = binding
                .context_ref()
                .or(self.default_context.as_deref())
                .and_then(|ctx_name| self.contexts.get(ctx_name).cloned());

            // Calculate combined score
            let trigger_score = trigger.match_score(
                platform,
                Some(&message.channel_id),
                Some(&message.user_id),
                Some(&message.text),
            ) as i32;
            let binding_score = binding.match_score(
                Some(&message.channel_id),
                Some(&message.user_id),
                Some(&message.text),
            );
            let total_score = trigger_score + binding_score;

            info!(
                "Binding '{}' matched (trigger_score={}, binding_score={}, total={})",
                binding.name(),
                trigger_score,
                binding_score,
                total_score
            );

            matches.push(BindingMatch {
                binding: binding.clone(),
                flow: flow.clone(),
                context,
                trigger: trigger.clone(),
                score: total_score,
            });
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.cmp(&a.score));

        matches
    }

    /// Get the best matching binding
    pub fn route_best(
        &self,
        platform: &str,
        message: &TriggerMessage,
    ) -> Option<BindingMatch> {
        self.route(platform, message).into_iter().next()
    }

    /// Find all bindings for a specific trigger
    pub fn bindings_for_trigger(&self, trigger_name: &str) -> Vec<&FlowBinding> {
        self.bindings
            .get_all()
            .into_iter()
            .filter(|b| b.trigger_ref() == trigger_name)
            .collect()
    }

    /// Find all bindings for a specific context
    pub fn bindings_for_context(&self, context_name: &str) -> Vec<&FlowBinding> {
        self.bindings
            .get_all()
            .into_iter()
            .filter(|b| b.context_ref() == Some(context_name))
            .collect()
    }

    /// Find all bindings for a specific flow
    pub fn bindings_for_flow(&self, flow_name: &str) -> Vec<&FlowBinding> {
        self.bindings
            .get_all()
            .into_iter()
            .filter(|b| b.flow_ref() == flow_name)
            .collect()
    }

    /// Get registry references for inspection
    pub fn bindings(&self) -> &BindingRegistry {
        &self.bindings
    }

    pub fn flows(&self) -> &FlowRegistry {
        &self.flows
    }

    pub fn contexts(&self) -> &ContextRegistry {
        &self.contexts
    }

    pub fn triggers(&self) -> &TriggerRegistry {
        &self.triggers
    }
}

/// Execution context with resolved binding information
///
/// This struct provides all the context needed to execute a flow
/// after routing through a binding.
#[derive(Debug, Clone)]
pub struct ResolvedExecutionContext {
    /// The flow to execute
    pub flow: AgentFlow,

    /// Environment variables from context
    pub env_vars: std::collections::HashMap<String, String>,

    /// Whether approval is required for this execution
    pub requires_approval: bool,

    /// Allowed approvers (if approval required)
    pub allowed_approvers: Vec<String>,

    /// Context name (for logging/audit)
    pub context_name: Option<String>,

    /// Trigger name (for logging/audit)
    pub trigger_name: String,

    /// Binding name (for logging/audit)
    pub binding_name: String,

    /// The original trigger message
    pub trigger_data: serde_json::Value,
}

impl BindingMatch {
    /// Convert this match into an execution context
    ///
    /// This resolves all the context settings and prepares
    /// the environment for flow execution.
    pub fn into_execution_context(
        self,
        message: &TriggerMessage,
        platform: &str,
    ) -> ResolvedExecutionContext {
        // Build trigger data
        let trigger_data = serde_json::json!({
            "event": {
                "type": "message",
                "text": message.text,
                "user": {
                    "id": message.user_id,
                    "username": message.user_name,
                },
                "channel_id": message.channel_id,
                "thread_id": message.thread_id,
                "timestamp": message.timestamp.to_rfc3339(),
                "metadata": message.metadata,
            },
            "platform": platform,
            "trigger_name": self.trigger.name(),
            "binding_name": self.binding.name(),
            "context_name": self.context.as_ref().map(|c| c.name()),
            "flow_name": self.flow.metadata.name,
            "match_score": self.score,
        });

        // Extract context settings
        let (env_vars, requires_approval, allowed_approvers, context_name) =
            if let Some(ref ctx) = self.context {
                let env = ctx.get_env_vars();
                let requires = ctx.requires_approval(&message.text);
                let approvers = ctx
                    .spec
                    .approval
                    .as_ref()
                    .map(|a| a.allowed_users.clone())
                    .unwrap_or_default();
                (env, requires, approvers, Some(ctx.name().to_string()))
            } else {
                (std::collections::HashMap::new(), false, vec![], None)
            };

        ResolvedExecutionContext {
            flow: self.flow,
            env_vars,
            requires_approval,
            allowed_approvers,
            context_name,
            trigger_name: self.trigger.name().to_string(),
            binding_name: self.binding.name().to_string(),
            trigger_data,
        }
    }

    /// Check if the current user can approve actions in this context
    pub fn can_approve(&self, user_id: &str) -> bool {
        if let Some(ref ctx) = self.context {
            ctx.is_approver(user_id)
        } else {
            // No context = no approval restrictions
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::registry::{BindingRegistry, ContextRegistry, FlowRegistry, TriggerRegistry};
    use chrono::Utc;
    use std::collections::HashMap;

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
    fn test_binding_router_creation() {
        let router = BindingRouter::new(
            BindingRegistry::new(),
            FlowRegistry::new(),
            ContextRegistry::new(),
            TriggerRegistry::new(),
        );

        assert!(router.default_context.is_none());
    }

    #[test]
    fn test_binding_router_with_default_context() {
        let router = BindingRouter::new(
            BindingRegistry::new(),
            FlowRegistry::new(),
            ContextRegistry::new(),
            TriggerRegistry::new(),
        )
        .with_default_context("production");

        assert_eq!(router.default_context, Some("production".to_string()));
    }

    #[test]
    fn test_route_empty_registries() {
        let router = BindingRouter::new(
            BindingRegistry::new(),
            FlowRegistry::new(),
            ContextRegistry::new(),
            TriggerRegistry::new(),
        );

        let message = create_test_message("C123", "U456", "kubectl get pods");
        let matches = router.route("slack", &message);

        assert!(matches.is_empty());
    }
}
