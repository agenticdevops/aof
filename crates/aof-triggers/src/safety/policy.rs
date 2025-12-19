//! Platform Policy - Enforce platform-specific access controls
//!
//! Different platforms have different trust levels:
//! - CLI: Highest trust (authenticated, local)
//! - Slack: Medium trust (enterprise, desktop)
//! - Telegram: Lower trust (mobile, less controlled)
//! - WhatsApp: Lowest trust (personal devices)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::classifier::ActionClass;

/// Policy decision for an operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Operation is allowed
    Allow,
    /// Operation requires human approval
    RequireApproval {
        /// Reason for requiring approval
        reason: String,
        /// Timeout in minutes
        timeout_minutes: u32,
    },
    /// Operation is blocked
    Block {
        /// Reason for blocking
        reason: String,
        /// Alternative suggestion
        suggestion: Option<String>,
    },
}

impl PolicyDecision {
    /// Check if the decision is Allow
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Check if the decision requires approval
    pub fn requires_approval(&self) -> bool {
        matches!(self, Self::RequireApproval { .. })
    }

    /// Check if the decision is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block { .. })
    }
}

/// Platform-specific policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPolicy {
    /// Action classes that are blocked on this platform
    #[serde(default)]
    pub blocked_classes: Vec<ActionClass>,

    /// Action classes that require approval on this platform
    #[serde(default)]
    pub approval_classes: Vec<ActionClass>,

    /// Action classes that are allowed on this platform
    #[serde(default)]
    pub allowed_classes: Vec<ActionClass>,

    /// Custom message when operations are blocked
    #[serde(default)]
    pub blocked_message: Option<String>,

    /// Approval timeout in minutes (default: 30)
    #[serde(default = "default_approval_timeout")]
    pub approval_timeout_minutes: u32,
}

fn default_approval_timeout() -> u32 {
    30
}

impl Default for PlatformPolicy {
    fn default() -> Self {
        Self {
            blocked_classes: vec![],
            approval_classes: vec![],
            allowed_classes: vec![ActionClass::Read],
            blocked_message: None,
            approval_timeout_minutes: 30,
        }
    }
}

impl PlatformPolicy {
    /// Create a new read-only policy
    pub fn read_only() -> Self {
        Self {
            blocked_classes: vec![ActionClass::Write, ActionClass::Delete, ActionClass::Dangerous],
            approval_classes: vec![],
            allowed_classes: vec![ActionClass::Read],
            blocked_message: Some("This platform is read-only.".to_string()),
            approval_timeout_minutes: 30,
        }
    }

    /// Create a policy requiring approval for writes
    pub fn require_write_approval() -> Self {
        Self {
            blocked_classes: vec![ActionClass::Dangerous],
            approval_classes: vec![ActionClass::Write, ActionClass::Delete],
            allowed_classes: vec![ActionClass::Read],
            blocked_message: None,
            approval_timeout_minutes: 30,
        }
    }

    /// Create a permissive policy (all allowed)
    pub fn permissive() -> Self {
        Self {
            blocked_classes: vec![],
            approval_classes: vec![],
            allowed_classes: vec![
                ActionClass::Read,
                ActionClass::Write,
                ActionClass::Delete,
                ActionClass::Dangerous,
            ],
            blocked_message: None,
            approval_timeout_minutes: 30,
        }
    }

    /// Evaluate this policy against an action class
    pub fn evaluate(&self, action_class: ActionClass) -> PolicyDecision {
        // Check blocked first
        if self.blocked_classes.contains(&action_class) {
            return PolicyDecision::Block {
                reason: format!(
                    "{} operations are blocked on this platform",
                    action_class
                ),
                suggestion: self.blocked_message.clone(),
            };
        }

        // Check approval required
        if self.approval_classes.contains(&action_class) {
            return PolicyDecision::RequireApproval {
                reason: format!(
                    "{} operations require approval on this platform",
                    action_class
                ),
                timeout_minutes: self.approval_timeout_minutes,
            };
        }

        // Default: allow if in allowed list, otherwise block
        if self.allowed_classes.contains(&action_class) {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Block {
                reason: format!(
                    "{} operations are not explicitly allowed on this platform",
                    action_class
                ),
                suggestion: None,
            }
        }
    }
}

/// Policy engine that manages policies for multiple platforms
pub struct PolicyEngine {
    /// Policies by platform name
    policies: HashMap<String, PlatformPolicy>,
    /// Default policy for unknown platforms
    default_policy: PlatformPolicy,
    /// Users allowed to approve operations
    approval_allowed_users: Vec<String>,
}

impl PolicyEngine {
    /// Create a new policy engine with default policies
    pub fn new() -> Self {
        let mut policies = HashMap::new();

        // Default policies for known platforms
        policies.insert("cli".to_string(), PlatformPolicy::permissive());
        policies.insert("slack".to_string(), PlatformPolicy::require_write_approval());
        policies.insert("telegram".to_string(), PlatformPolicy::read_only());
        policies.insert("whatsapp".to_string(), PlatformPolicy::read_only());
        policies.insert("discord".to_string(), PlatformPolicy::require_write_approval());

        Self {
            policies,
            default_policy: PlatformPolicy::read_only(), // Fail secure
            approval_allowed_users: vec![],
        }
    }

    /// Set policy for a platform
    pub fn set_policy(&mut self, platform: &str, policy: PlatformPolicy) {
        self.policies.insert(platform.to_lowercase(), policy);
    }

    /// Set the default policy for unknown platforms
    pub fn set_default_policy(&mut self, policy: PlatformPolicy) {
        self.default_policy = policy;
    }

    /// Set users who can approve operations
    pub fn set_approval_users(&mut self, users: Vec<String>) {
        self.approval_allowed_users = users;
    }

    /// Add a user who can approve operations
    pub fn add_approval_user(&mut self, user: &str) {
        self.approval_allowed_users.push(user.to_string());
    }

    /// Check if a user can approve operations
    pub fn can_user_approve(&self, user_id: &str) -> bool {
        if self.approval_allowed_users.is_empty() {
            // If no users specified, anyone can approve
            return true;
        }

        self.approval_allowed_users.iter().any(|u| {
            if u.starts_with('@') {
                // Role/group check (e.g., "@oncall")
                // For now, just check if user_id contains the role name
                user_id.contains(&u[1..])
            } else {
                u == user_id
            }
        })
    }

    /// Get policy for a platform
    pub fn get_policy(&self, platform: &str) -> &PlatformPolicy {
        self.policies
            .get(&platform.to_lowercase())
            .unwrap_or(&self.default_policy)
    }

    /// Evaluate an action on a platform
    pub fn evaluate(&self, platform: &str, action_class: ActionClass) -> PolicyDecision {
        let policy = self.get_policy(platform);
        policy.evaluate(action_class)
    }

    /// Evaluate with additional context
    pub fn evaluate_with_context(
        &self,
        platform: &str,
        action_class: ActionClass,
        user_id: &str,
        _namespace: Option<&str>,
    ) -> PolicyDecision {
        let decision = self.evaluate(platform, action_class);

        // Check if user can bypass approval
        if let PolicyDecision::RequireApproval { .. } = &decision {
            if self.can_user_approve(user_id) && self.approval_allowed_users.contains(&user_id.to_string()) {
                // Privileged users can skip approval for their own actions
                // (but still need approval from others for critical actions)
            }
        }

        decision
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_only_policy() {
        let policy = PlatformPolicy::read_only();

        assert!(policy.evaluate(ActionClass::Read).is_allowed());
        assert!(policy.evaluate(ActionClass::Write).is_blocked());
        assert!(policy.evaluate(ActionClass::Delete).is_blocked());
        assert!(policy.evaluate(ActionClass::Dangerous).is_blocked());
    }

    #[test]
    fn test_require_write_approval_policy() {
        let policy = PlatformPolicy::require_write_approval();

        assert!(policy.evaluate(ActionClass::Read).is_allowed());
        assert!(policy.evaluate(ActionClass::Write).requires_approval());
        assert!(policy.evaluate(ActionClass::Delete).requires_approval());
        assert!(policy.evaluate(ActionClass::Dangerous).is_blocked());
    }

    #[test]
    fn test_permissive_policy() {
        let policy = PlatformPolicy::permissive();

        assert!(policy.evaluate(ActionClass::Read).is_allowed());
        assert!(policy.evaluate(ActionClass::Write).is_allowed());
        assert!(policy.evaluate(ActionClass::Delete).is_allowed());
        assert!(policy.evaluate(ActionClass::Dangerous).is_allowed());
    }

    #[test]
    fn test_policy_engine_defaults() {
        let engine = PolicyEngine::new();

        // CLI is permissive
        assert!(engine.evaluate("cli", ActionClass::Dangerous).is_allowed());

        // Telegram is read-only
        assert!(engine.evaluate("telegram", ActionClass::Read).is_allowed());
        assert!(engine.evaluate("telegram", ActionClass::Write).is_blocked());

        // Slack requires approval for writes
        assert!(engine.evaluate("slack", ActionClass::Read).is_allowed());
        assert!(engine.evaluate("slack", ActionClass::Write).requires_approval());
    }

    #[test]
    fn test_policy_engine_custom_policy() {
        let mut engine = PolicyEngine::new();

        let custom = PlatformPolicy {
            blocked_classes: vec![ActionClass::Dangerous],
            approval_classes: vec![ActionClass::Delete],
            allowed_classes: vec![ActionClass::Read, ActionClass::Write],
            blocked_message: Some("No dangerous ops!".to_string()),
            approval_timeout_minutes: 15,
        };

        engine.set_policy("custom-platform", custom);

        assert!(engine.evaluate("custom-platform", ActionClass::Read).is_allowed());
        assert!(engine.evaluate("custom-platform", ActionClass::Write).is_allowed());
        assert!(engine.evaluate("custom-platform", ActionClass::Delete).requires_approval());
        assert!(engine.evaluate("custom-platform", ActionClass::Dangerous).is_blocked());
    }

    #[test]
    fn test_approval_users() {
        let mut engine = PolicyEngine::new();
        engine.set_approval_users(vec!["admin".to_string(), "@oncall".to_string()]);

        assert!(engine.can_user_approve("admin"));
        assert!(engine.can_user_approve("oncall-engineer")); // Contains "oncall"
        assert!(!engine.can_user_approve("regular-user"));
    }

    #[test]
    fn test_empty_approval_users_allows_all() {
        let engine = PolicyEngine::new();
        assert!(engine.can_user_approve("anyone"));
    }
}
