//! Approval workflow types for OpenAgentiX — human-in-the-loop pause/resume.
//!
//! This module provides the foundational approval primitives:
//! - `ApprovalRequest` — a pending approval for a flagged action
//! - `ApprovalStatus` — lifecycle state of an approval request (rich enum with data)
//! - `ApprovalDecision` — structured decision record for audit trails
//! - `ApprovalPolicy` — per-agent approval rules (flagged tools, patterns, timeout)
//!
//! # Design
//!
//! `ApprovalStatus` uses tagged variants so that approver identity and timestamp
//! are embedded directly in the status, eliminating the need for a separate
//! `decision` field and enforcing single-decision semantics at the type level.
//!
//! `ApprovalPolicy` references `AgentMode` (from `crate::agent`) to unify the
//! autonomy mode with the agent definition's `mode` field.

use crate::agent::AgentMode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// ApprovalStatus
// ---------------------------------------------------------------------------

/// Lifecycle state of an approval request.
///
/// Each non-Pending variant carries the data produced by that transition,
/// making it impossible to retrieve decision metadata without knowing the
/// decision outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Waiting for a human to approve or deny.
    Pending,

    /// Approved by a human approver.
    Approved {
        /// Identity of the approver (email, username, Slack ID, etc.).
        approver: String,
        /// When the approval was granted.
        decided_at: DateTime<Utc>,
    },

    /// Denied by a human approver.
    Denied {
        /// Identity of the approver who denied the request.
        approver: String,
        /// Optional reason for the denial.
        reason: Option<String>,
        /// When the denial was recorded.
        decided_at: DateTime<Utc>,
    },

    /// Timed out without a decision.
    TimedOut {
        /// When the request expired.
        expired_at: DateTime<Utc>,
    },
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "pending"),
            ApprovalStatus::Approved { .. } => write!(f, "approved"),
            ApprovalStatus::Denied { .. } => write!(f, "denied"),
            ApprovalStatus::TimedOut { .. } => write!(f, "timed_out"),
        }
    }
}

// ---------------------------------------------------------------------------
// ApprovalRequest
// ---------------------------------------------------------------------------

/// A pending approval request created when an agent action needs human review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique UUID identifier for this approval request.
    pub id: String,

    /// Run ID where the approval was requested.
    pub run_id: String,

    /// Name of the agent that triggered this request.
    pub agent_name: String,

    /// Human-readable description of the action requiring approval.
    pub action_description: String,

    /// Name of the tool being called (None if not a tool-specific request).
    pub tool_name: Option<String>,

    /// JSON arguments to the tool (None if not tool-specific).
    pub tool_input: Option<serde_json::Value>,

    /// When the request was created.
    pub requested_at: DateTime<Utc>,

    /// How long (in seconds) before this request expires.
    pub timeout_secs: u32,

    /// Current lifecycle status.
    pub status: ApprovalStatus,
}

impl ApprovalRequest {
    /// Create a new approval request in `Pending` state.
    ///
    /// Generates a UUID `id` and sets `requested_at` to `Utc::now()`.
    pub fn new(
        run_id: String,
        agent_name: String,
        action_description: String,
        tool_name: Option<String>,
        tool_input: Option<serde_json::Value>,
        timeout_secs: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            run_id,
            agent_name,
            action_description,
            tool_name,
            tool_input,
            requested_at: Utc::now(),
            timeout_secs,
            status: ApprovalStatus::Pending,
        }
    }

    /// Transition a `Pending` request to `Approved`.
    ///
    /// Returns `Err("already decided")` if the request is not `Pending`.
    pub fn approve(&mut self, approver: &str) -> Result<(), String> {
        if !matches!(self.status, ApprovalStatus::Pending) {
            return Err("already decided".to_string());
        }
        self.status = ApprovalStatus::Approved {
            approver: approver.to_string(),
            decided_at: Utc::now(),
        };
        Ok(())
    }

    /// Transition a `Pending` request to `Denied`.
    ///
    /// Returns `Err("already decided")` if the request is not `Pending`.
    pub fn deny(&mut self, approver: &str, reason: Option<String>) -> Result<(), String> {
        if !matches!(self.status, ApprovalStatus::Pending) {
            return Err("already decided".to_string());
        }
        self.status = ApprovalStatus::Denied {
            approver: approver.to_string(),
            reason,
            decided_at: Utc::now(),
        };
        Ok(())
    }

    /// Transition a `Pending` request to `TimedOut`.
    ///
    /// No-op if the request has already been decided.
    pub fn expire(&mut self) {
        if matches!(self.status, ApprovalStatus::Pending) {
            self.status = ApprovalStatus::TimedOut {
                expired_at: Utc::now(),
            };
        }
    }

    /// Returns `true` if the request is `Pending` and the timeout has elapsed.
    ///
    /// A request with `timeout_secs=0` is considered expired immediately.
    pub fn is_expired(&self) -> bool {
        if !matches!(self.status, ApprovalStatus::Pending) {
            return false;
        }
        let deadline = self.requested_at
            + chrono::Duration::seconds(self.timeout_secs as i64);
        Utc::now() >= deadline
    }
}

// ---------------------------------------------------------------------------
// ApprovalDecision
// ---------------------------------------------------------------------------

/// A structured record of an approver's decision, for use in audit trails and
/// API responses. This is separate from `ApprovalStatus` so it can be stored
/// and transmitted independently of the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    /// The ID of the `ApprovalRequest` this decision belongs to.
    pub request_id: String,

    /// `true` = approved, `false` = denied.
    pub approved: bool,

    /// Identity of the approver (email, username, Slack ID, etc.).
    pub approver: String,

    /// Optional reason (typically provided on denial).
    pub reason: Option<String>,

    /// When the decision was made.
    pub decided_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// ApprovalPolicy
// ---------------------------------------------------------------------------

/// Per-agent approval policy configuration.
///
/// Controls when the agent pauses for human approval based on the `mode`
/// and optional lists of flagged tools and input patterns.
///
/// ```yaml
/// approval:
///   mode: semi-autonomous
///   flagged_tools:
///     - kubectl
///     - aws
///   flagged_patterns:
///     - delete
///     - drop
///     - terminate
///   approvers:
///     - "slack:U015ADMIN"
///     - "email:sre-lead@company.com"
///   default_timeout_secs: 300
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// The agent's autonomy mode.
    #[serde(default)]
    pub mode: AgentMode,

    /// Tool names that always require approval in `SemiAutonomous` mode.
    #[serde(default)]
    pub flagged_tools: Vec<String>,

    /// Substrings (case-insensitive) matched against the serialized tool input.
    /// Any match triggers an approval gate in `SemiAutonomous` mode.
    #[serde(default)]
    pub flagged_patterns: Vec<String>,

    /// Default timeout (seconds) before an unanswered request expires.
    #[serde(default = "default_timeout_secs")]
    pub default_timeout_secs: u32,

    /// Authorized approver identities (email, Slack ID, etc.).
    #[serde(default)]
    pub approvers: Vec<String>,
}

fn default_timeout_secs() -> u32 {
    300
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self {
            mode: AgentMode::Autonomous,
            flagged_tools: Vec::new(),
            flagged_patterns: Vec::new(),
            default_timeout_secs: default_timeout_secs(),
            approvers: Vec::new(),
        }
    }
}

impl ApprovalPolicy {
    /// Determine whether a tool call requires approval under this policy.
    ///
    /// - `Autonomous` — always returns `false`
    /// - `Manual` — always returns `true`
    /// - `SemiAutonomous` — returns `true` if:
    ///   1. `tool_name` is in `flagged_tools`, OR
    ///   2. The serialized `tool_input` contains any string in `flagged_patterns`
    ///      (case-insensitive substring match)
    pub fn requires_approval(&self, tool_name: &str, tool_input: &serde_json::Value) -> bool {
        match self.mode {
            AgentMode::Autonomous => false,
            AgentMode::Manual => true,
            AgentMode::SemiAutonomous => {
                // Check flagged tool names
                if self.flagged_tools.iter().any(|t| t == tool_name) {
                    return true;
                }
                // Check flagged patterns against serialized input (case-insensitive)
                let input_str = tool_input.to_string().to_lowercase();
                self.flagged_patterns
                    .iter()
                    .any(|p| input_str.contains(&p.to_lowercase()))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request() -> ApprovalRequest {
        ApprovalRequest::new(
            "run-test".to_string(),
            "test-agent".to_string(),
            "Run kubectl delete pod nginx".to_string(),
            Some("kubectl".to_string()),
            Some(serde_json::json!({"args": "delete pod nginx"})),
            300,
        )
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest::new
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_request_is_pending() {
        let req = make_request();
        assert!(matches!(req.status, ApprovalStatus::Pending));
        assert!(!req.id.is_empty());
        assert_eq!(req.run_id, "run-test");
        assert_eq!(req.agent_name, "test-agent");
        assert_eq!(req.timeout_secs, 300);
    }

    // -----------------------------------------------------------------------
    // approve / deny transitions
    // -----------------------------------------------------------------------

    #[test]
    fn test_approve_transitions_to_approved() {
        let mut req = make_request();
        req.approve("admin@co.com").unwrap();
        assert!(matches!(req.status, ApprovalStatus::Approved { .. }));
    }

    #[test]
    fn test_deny_transitions_to_denied() {
        let mut req = make_request();
        req.deny("lead", Some("Too risky".to_string())).unwrap();
        assert!(matches!(req.status, ApprovalStatus::Denied { .. }));
    }

    #[test]
    fn test_double_approve_fails() {
        let mut req = make_request();
        req.approve("admin").unwrap();
        assert!(req.approve("admin2").is_err());
    }

    #[test]
    fn test_approve_after_deny_fails() {
        let mut req = make_request();
        req.deny("lead", None).unwrap();
        assert!(req.approve("admin").is_err());
    }

    // -----------------------------------------------------------------------
    // is_expired
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_expired_zero_timeout() {
        let req = ApprovalRequest::new(
            "run".to_string(),
            "agent".to_string(),
            "action".to_string(),
            None,
            None,
            0,
        );
        assert!(req.is_expired());
    }

    #[test]
    fn test_is_not_expired_long_timeout() {
        let req = ApprovalRequest::new(
            "run".to_string(),
            "agent".to_string(),
            "action".to_string(),
            None,
            None,
            3600,
        );
        assert!(!req.is_expired());
    }

    // -----------------------------------------------------------------------
    // ApprovalPolicy
    // -----------------------------------------------------------------------

    #[test]
    fn test_autonomous_never_requires_approval() {
        let policy = ApprovalPolicy {
            mode: AgentMode::Autonomous,
            ..Default::default()
        };
        assert!(!policy.requires_approval("kubectl", &serde_json::json!({})));
        assert!(!policy.requires_approval("any-tool", &serde_json::json!({})));
    }

    #[test]
    fn test_manual_always_requires_approval() {
        let policy = ApprovalPolicy {
            mode: AgentMode::Manual,
            ..Default::default()
        };
        assert!(policy.requires_approval("kubectl", &serde_json::json!({})));
        assert!(policy.requires_approval("any-tool", &serde_json::json!({})));
    }

    #[test]
    fn test_semi_autonomous_flagged_tool() {
        let policy = ApprovalPolicy {
            mode: AgentMode::SemiAutonomous,
            flagged_tools: vec!["kubectl".to_string()],
            ..Default::default()
        };
        assert!(policy.requires_approval("kubectl", &serde_json::json!({})));
        assert!(!policy.requires_approval("git", &serde_json::json!({})));
    }

    #[test]
    fn test_semi_autonomous_flagged_pattern() {
        let policy = ApprovalPolicy {
            mode: AgentMode::SemiAutonomous,
            flagged_patterns: vec!["delete".to_string()],
            ..Default::default()
        };
        assert!(policy.requires_approval(
            "shell",
            &serde_json::json!({"command": "kubectl delete pod"})
        ));
        assert!(!policy.requires_approval(
            "shell",
            &serde_json::json!({"command": "kubectl get pods"})
        ));
    }

    #[test]
    fn test_policy_default() {
        let policy = ApprovalPolicy::default();
        assert_eq!(policy.mode, AgentMode::Autonomous);
        assert!(policy.flagged_tools.is_empty());
        assert!(policy.flagged_patterns.is_empty());
        assert_eq!(policy.default_timeout_secs, 300);
        assert!(policy.approvers.is_empty());
    }
}
