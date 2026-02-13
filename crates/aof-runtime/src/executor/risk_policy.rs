//! Risk-based sandboxing decision engine
//!
//! This module evaluates whether tools should execute in sandboxed containers
//! based on execution context (dev vs prod) and operation type (read vs destructive).

use aof_core::error::AofError;
use serde::{Deserialize, Serialize};
use crate::executor::sandbox::SandboxConfig;

/// Execution environment context
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionContext {
    /// Development environment
    Development,
    /// Production environment
    Production,
    /// Custom environment
    Custom(String),
}

impl ExecutionContext {
    pub fn is_production(&self) -> bool {
        matches!(self, ExecutionContext::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, ExecutionContext::Development)
    }
}

/// Risk level of an operation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Read-only operations
    Low,
    /// Write operations
    Medium,
    /// Destructive operations
    High,
    /// Privilege escalation or secret access
    Critical,
}

/// Sandboxing decision
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxingDecision {
    /// Run in Docker container with restrictions
    Sandbox,
    /// Run on host with seccomp restrictions
    HostWithRestrictions,
    /// Run on host without restrictions
    HostTrusted,
}

/// Risk-based sandboxing policy engine
pub struct RiskPolicy {
    // TODO: Add configurable thresholds
}

impl RiskPolicy {
    /// Create a new risk policy
    pub fn new() -> Self {
        Self {}
    }

    /// Determine if a tool should be sandboxed
    pub fn should_sandbox(
        &self,
        context: &ExecutionContext,
        tool: &str,
        args: &[String],
    ) -> SandboxingDecision {
        let risk_level = self.assess_risk(tool, args);

        match (context.is_production(), risk_level) {
            // High risk always sandbox
            (_, RiskLevel::High) | (_, RiskLevel::Critical) => SandboxingDecision::Sandbox,
            // Prod writes sandbox
            (true, RiskLevel::Medium) => SandboxingDecision::Sandbox,
            // Prod reads on host (trusted)
            (true, RiskLevel::Low) => SandboxingDecision::HostTrusted,
            // Dev always sandbox
            (false, _) => SandboxingDecision::Sandbox,
        }
    }

    /// Assess risk level of an operation
    fn assess_risk(&self, tool: &str, args: &[String]) -> RiskLevel {
        if self.is_destructive(tool, args) {
            RiskLevel::High
        } else if self.is_write(tool, args) {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Check if operation is destructive
    fn is_destructive(&self, tool: &str, args: &[String]) -> bool {
        let destructive_cmds = vec![
            "delete", "remove", "rm", "rmi", "kill", "stop", "restart", "scale",
            "terminate", "destroy", "drop", "truncate",
        ];

        let tool_lower = tool.to_lowercase();
        let cmd_str = if args.is_empty() {
            String::new()
        } else {
            format!("{} {}", tool, args.join(" ")).to_lowercase()
        };

        destructive_cmds
            .iter()
            .any(|cmd| tool_lower.contains(cmd) || cmd_str.contains(cmd))
    }

    /// Check if operation is a write (non-destructive modification)
    fn is_write(&self, tool: &str, args: &[String]) -> bool {
        let write_cmds = vec!["apply", "patch", "create", "set", "update", "edit"];

        let tool_lower = tool.to_lowercase();
        let cmd_str = if args.is_empty() {
            String::new()
        } else {
            format!("{} {}", tool, args.join(" ")).to_lowercase()
        };

        write_cmds
            .iter()
            .any(|cmd| tool_lower.contains(cmd) || cmd_str.contains(cmd))
    }

    /// Get sandbox restrictions for a decision
    pub fn get_sandbox_restrictions(&self, decision: &SandboxingDecision) -> SandboxConfig {
        match decision {
            SandboxingDecision::Sandbox => SandboxConfig::default(),
            SandboxingDecision::HostWithRestrictions => {
                // TODO: Return seccomp-only config
                SandboxConfig::default()
            }
            SandboxingDecision::HostTrusted => {
                // TODO: Return empty config
                SandboxConfig::default()
            }
        }
    }
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_destructive() {
        let policy = RiskPolicy::new();

        assert!(policy.is_destructive("kubectl", &["delete".to_string(), "pod".to_string()]));
        assert!(policy.is_destructive("docker", &["rm".to_string()]));
        assert!(!policy.is_destructive("kubectl", &["get".to_string(), "pods".to_string()]));
    }

    #[test]
    fn test_is_write() {
        let policy = RiskPolicy::new();

        assert!(policy.is_write("kubectl", &["apply".to_string()]));
        assert!(policy.is_write("kubectl", &["patch".to_string()]));
        assert!(!policy.is_write("kubectl", &["get".to_string()]));
        assert!(!policy.is_write("kubectl", &["delete".to_string()]));
    }

    #[test]
    fn test_should_sandbox_dev() {
        let policy = RiskPolicy::new();
        let dev = ExecutionContext::Development;

        // Dev always sandboxes
        assert_eq!(
            policy.should_sandbox(&dev, "kubectl", &["get".to_string()]),
            SandboxingDecision::Sandbox
        );
        assert_eq!(
            policy.should_sandbox(&dev, "kubectl", &["delete".to_string()]),
            SandboxingDecision::Sandbox
        );
    }

    #[test]
    fn test_should_sandbox_prod() {
        let policy = RiskPolicy::new();
        let prod = ExecutionContext::Production;

        // Prod destructive: sandbox
        assert_eq!(
            policy.should_sandbox(&prod, "kubectl", &["delete".to_string()]),
            SandboxingDecision::Sandbox
        );

        // Prod write: sandbox
        assert_eq!(
            policy.should_sandbox(&prod, "kubectl", &["apply".to_string()]),
            SandboxingDecision::Sandbox
        );

        // Prod read: host trusted
        assert_eq!(
            policy.should_sandbox(&prod, "kubectl", &["get".to_string()]),
            SandboxingDecision::HostTrusted
        );
    }

    #[test]
    fn test_execution_context() {
        let dev = ExecutionContext::Development;
        let prod = ExecutionContext::Production;

        assert!(dev.is_development());
        assert!(!dev.is_production());
        assert!(prod.is_production());
        assert!(!prod.is_development());
    }
}
