use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Operation classification for approval routing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationCategory {
    /// DELETE, DROP, SCALE_DOWN, DESTROY - requires approval
    Destructive,
    /// UPDATE, PATCH, REBOOT, MODIFY - may require approval depending on blast radius
    Risky,
    /// READ, DESCRIBE, LIST, GET - no approval needed
    Safe,
}

impl std::fmt::Display for OperationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Destructive => write!(f, "Destructive"),
            Self::Risky => write!(f, "Risky"),
            Self::Safe => write!(f, "Safe"),
        }
    }
}

/// Environment determines auto-approval for Risky operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "Development"),
            Self::Staging => write!(f, "Staging"),
            Self::Production => write!(f, "Production"),
        }
    }
}

/// Blast radius heuristic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    pub affected_services: Vec<String>,
    pub estimated_impact_pct: f32,
    pub data_at_risk: bool,
}

/// Approval policy determines if approval is needed
#[derive(Debug, Clone)]
pub struct ApprovalPolicy {
    pub environment: Environment,
    pub require_approval_for_risky: bool,
    pub approval_timeout_secs: u64,
    pub auto_reject_timeout: bool,
}

impl ApprovalPolicy {
    /// Determine if operation needs approval
    pub fn requires_approval(
        &self,
        category: OperationCategory,
        _blast_radius: Option<&BlastRadius>,
    ) -> bool {
        match category {
            OperationCategory::Destructive => true,
            OperationCategory::Risky => {
                // Require approval in prod, auto-approve in dev/staging
                self.environment == Environment::Production || self.require_approval_for_risky
            }
            OperationCategory::Safe => false,
        }
    }
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub operation_id: String,
    pub category: OperationCategory,
    pub description: String,
    pub operator: String, // Who initiated
    pub blast_radius: Option<BlastRadius>,
    pub created_at: SystemTime,
    pub decision: Option<ApprovalDecision>,
    pub decided_at: Option<SystemTime>,
    pub decided_by: Option<String>, // Who approved/rejected
    pub reason: Option<String>,     // Reason for decision
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Approved => write!(f, "Approved"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub status: ApprovalStatus,
    pub timestamp: SystemTime,
    pub decided_by: String,
    pub reason: Option<String>,
}

/// Classification helper - map operation names to categories
pub fn classify_operation(operation: &str) -> OperationCategory {
    let lower = operation.to_lowercase();

    // Destructive patterns
    if lower.contains("delete")
        || lower.contains("drop")
        || lower.contains("destroy")
        || lower.contains("remove")
        || lower.contains("scale_down")
        || lower.contains("terminate")
    {
        return OperationCategory::Destructive;
    }

    // Risky patterns
    if lower.contains("update")
        || lower.contains("patch")
        || lower.contains("reboot")
        || lower.contains("restart")
        || lower.contains("modify")
        || lower.contains("apply")
    {
        return OperationCategory::Risky;
    }

    // Default to Safe (read-only)
    OperationCategory::Safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_operations() {
        assert_eq!(
            classify_operation("delete_pod"),
            OperationCategory::Destructive
        );
        assert_eq!(
            classify_operation("update_config"),
            OperationCategory::Risky
        );
        assert_eq!(
            classify_operation("describe_nodes"),
            OperationCategory::Safe
        );
        assert_eq!(
            classify_operation("get_pods"),
            OperationCategory::Safe
        );
        assert_eq!(
            classify_operation("DROP TABLE users"),
            OperationCategory::Destructive
        );
        assert_eq!(
            classify_operation("PATCH deployment"),
            OperationCategory::Risky
        );
    }

    #[test]
    fn test_approval_policy() {
        let policy = ApprovalPolicy {
            environment: Environment::Production,
            require_approval_for_risky: true,
            approval_timeout_secs: 300,
            auto_reject_timeout: false,
        };

        assert!(policy.requires_approval(OperationCategory::Destructive, None));
        assert!(policy.requires_approval(OperationCategory::Risky, None));
        assert!(!policy.requires_approval(OperationCategory::Safe, None));
    }

    #[test]
    fn test_approval_policy_staging() {
        let policy = ApprovalPolicy {
            environment: Environment::Staging,
            require_approval_for_risky: false,
            approval_timeout_secs: 300,
            auto_reject_timeout: false,
        };

        assert!(policy.requires_approval(OperationCategory::Destructive, None));
        assert!(!policy.requires_approval(OperationCategory::Risky, None));
        assert!(!policy.requires_approval(OperationCategory::Safe, None));
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Production.to_string(), "Production");
        assert_eq!(Environment::Staging.to_string(), "Staging");
        assert_eq!(Environment::Development.to_string(), "Development");
    }

    #[test]
    fn test_operation_category_display() {
        assert_eq!(OperationCategory::Destructive.to_string(), "Destructive");
        assert_eq!(OperationCategory::Risky.to_string(), "Risky");
        assert_eq!(OperationCategory::Safe.to_string(), "Safe");
    }
}
