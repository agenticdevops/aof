//! Safety decision visualization

use crate::RenderConfig;

/// Safety decision type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyDecision {
    /// Operation is allowed
    Allowed,
    /// Operation requires approval
    RequiresApproval,
    /// Operation is blocked
    Blocked,
}

impl SafetyDecision {
    /// Get emoji for decision
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Allowed => "✅",
            Self::RequiresApproval => "⚠️",
            Self::Blocked => "🚫",
        }
    }

    /// Get color name for terminal rendering
    pub fn color(&self) -> &'static str {
        match self {
            Self::Allowed => "green",
            Self::RequiresApproval => "yellow",
            Self::Blocked => "red",
        }
    }
}

/// Renders safety decisions
pub struct SafetyRenderer {
    config: RenderConfig,
}

impl SafetyRenderer {
    /// Create a new safety renderer
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Render a simple safety decision
    pub fn render_decision(&self, decision: SafetyDecision, message: &str) -> String {
        format!("{} {}", decision.emoji(), message)
    }

    /// Render a blocked operation message
    pub fn render_blocked(&self, reason: &str, suggestion: Option<&str>) -> String {
        let mut lines = vec![
            format!("🚫 **Operation Blocked**"),
            String::new(),
            reason.to_string(),
        ];

        if let Some(sug) = suggestion {
            lines.push(String::new());
            lines.push(format!("💡 {}", sug));
        }

        lines.join("\n")
    }

    /// Render an approval request
    pub fn render_approval_request(
        &self,
        command: &str,
        reason: &str,
        timeout_minutes: u32,
    ) -> String {
        if self.config.compact {
            format!(
                "⚠️ Approval needed\n{}\n\n{}\n\nExpires in {} min",
                command, reason, timeout_minutes
            )
        } else {
            format!(
                "┌─ ⚠️ Approval Required ─────────\n\
                 │\n\
                 │ Command:\n\
                 │   {}\n\
                 │\n\
                 │ Reason:\n\
                 │   {}\n\
                 │\n\
                 │ Expires in {} minutes\n\
                 │\n\
                 │ React ✅ to approve, ❌ to deny\n\
                 └",
                command, reason, timeout_minutes
            )
        }
    }

    /// Render tool classification info
    pub fn render_classification(
        &self,
        tool: &str,
        verb: &str,
        class: &str,
        platform: &str,
    ) -> String {
        if self.config.compact {
            format!("🔍 {} {} → {} ({})", tool, verb, class, platform)
        } else {
            format!(
                "┌─ Classification ────\n\
                 │ Tool:     {}\n\
                 │ Verb:     {}\n\
                 │ Class:    {}\n\
                 │ Platform: {}\n\
                 └",
                tool, verb, class, platform
            )
        }
    }

    /// Render platform policy summary
    pub fn render_policy_summary(&self, platform: &str, allowed: &[&str], blocked: &[&str]) -> String {
        let allowed_str = if allowed.is_empty() {
            "none".to_string()
        } else {
            allowed.join(", ")
        };

        let blocked_str = if blocked.is_empty() {
            "none".to_string()
        } else {
            blocked.join(", ")
        };

        if self.config.compact {
            format!(
                "📋 {} Policy\n✅ {}\n🚫 {}",
                platform, allowed_str, blocked_str
            )
        } else {
            format!(
                "┌─ {} Policy ─────────\n\
                 │ ✅ Allowed: {}\n\
                 │ 🚫 Blocked: {}\n\
                 └",
                platform, allowed_str, blocked_str
            )
        }
    }
}

impl Default for SafetyRenderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_decision_emoji() {
        assert_eq!(SafetyDecision::Allowed.emoji(), "✅");
        assert_eq!(SafetyDecision::RequiresApproval.emoji(), "⚠️");
        assert_eq!(SafetyDecision::Blocked.emoji(), "🚫");
    }

    #[test]
    fn test_render_decision() {
        let renderer = SafetyRenderer::new(RenderConfig::default());
        let result = renderer.render_decision(SafetyDecision::Allowed, "Operation permitted");
        assert!(result.contains("✅"));
        assert!(result.contains("Operation permitted"));
    }

    #[test]
    fn test_render_blocked() {
        let renderer = SafetyRenderer::new(RenderConfig::default());
        let result = renderer.render_blocked(
            "Delete operations not allowed",
            Some("Use CLI instead"),
        );
        assert!(result.contains("🚫"));
        assert!(result.contains("Delete operations"));
        assert!(result.contains("CLI"));
    }

    #[test]
    fn test_render_approval_request() {
        let renderer = SafetyRenderer::new(RenderConfig::default());
        let result = renderer.render_approval_request(
            "kubectl scale deployment/app --replicas=5",
            "Write operation requires approval",
            30,
        );
        assert!(result.contains("⚠️"));
        assert!(result.contains("kubectl scale"));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_render_classification() {
        let renderer = SafetyRenderer::new(RenderConfig::default());
        let result = renderer.render_classification("kubectl", "delete", "delete", "telegram");
        assert!(result.contains("kubectl"));
        assert!(result.contains("delete"));
        assert!(result.contains("telegram"));
    }
}
