//! Status rendering for agent execution

use crate::RenderConfig;

/// Execution status states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Waiting to start
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Success,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
    /// Waiting for approval
    WaitingApproval,
}

impl ExecutionStatus {
    /// Get emoji for status
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Pending => "⏳",
            Self::Running => "🔄",
            Self::Success => "✅",
            Self::Failed => "❌",
            Self::Cancelled => "🚫",
            Self::WaitingApproval => "⏸️",
        }
    }

    /// Get ASCII indicator for status
    pub fn ascii(&self) -> &'static str {
        match self {
            Self::Pending => "[.]",
            Self::Running => "[~]",
            Self::Success => "[+]",
            Self::Failed => "[x]",
            Self::Cancelled => "[-]",
            Self::WaitingApproval => "[?]",
        }
    }

    /// Get status label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Success => "Success",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::WaitingApproval => "Awaiting Approval",
        }
    }
}

/// Style for status rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusStyle {
    /// Use emoji indicators
    Emoji,
    /// Use ASCII art
    Ascii,
    /// Minimal text only
    Text,
}

/// Renders execution status
pub struct StatusRenderer {
    config: RenderConfig,
    style: StatusStyle,
}

impl StatusRenderer {
    /// Create a new status renderer
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            style: StatusStyle::Emoji,
        }
    }

    /// Set rendering style
    pub fn with_style(mut self, style: StatusStyle) -> Self {
        self.style = style;
        self
    }

    /// Render a simple status line
    pub fn render_status(&self, status: ExecutionStatus, message: &str) -> String {
        let indicator = match self.style {
            StatusStyle::Emoji => status.emoji(),
            StatusStyle::Ascii => status.ascii(),
            StatusStyle::Text => "",
        };

        if self.config.compact {
            format!("{} {}", indicator, message)
        } else {
            format!("{} {} - {}", indicator, status.label(), message)
        }
    }

    /// Render thinking/processing indicator
    pub fn render_thinking(&self) -> String {
        match self.style {
            StatusStyle::Emoji => "🤔 Thinking...".to_string(),
            StatusStyle::Ascii => "[~] Thinking...".to_string(),
            StatusStyle::Text => "Thinking...".to_string(),
        }
    }

    /// Render agent header
    pub fn render_agent_header(&self, agent_name: &str) -> String {
        if self.config.compact {
            format!("🤖 {}", agent_name)
        } else {
            let width = self.config.max_width;
            let line = "─".repeat(width.saturating_sub(4));
            format!("┌{}┐\n│ 🤖 {} │\n└{}┘", line, agent_name, line)
        }
    }

    /// Render execution progress
    pub fn render_progress(&self, current: usize, total: usize, label: &str) -> String {
        let percent = if total > 0 { (current * 100) / total } else { 0 };

        if self.config.compact {
            format!("{} [{}/{}] {}%", label, current, total, percent)
        } else {
            let bar_width = self.config.max_width.saturating_sub(20);
            let filled = (bar_width * current) / total.max(1);
            let empty = bar_width.saturating_sub(filled);

            let bar = format!(
                "[{}{}]",
                "█".repeat(filled),
                "░".repeat(empty)
            );

            format!("{} {} {}%", label, bar, percent)
        }
    }

    /// Render a boxed message
    pub fn render_box(&self, title: &str, content: &str) -> String {
        if !self.config.use_unicode {
            return format!("--- {} ---\n{}\n---", title, content);
        }

        let width = self.config.max_width;
        let top = format!("┌─ {} {}", title, "─".repeat(width.saturating_sub(title.len() + 4)));
        let bottom = "└".to_string() + &"─".repeat(width.saturating_sub(1));

        let mut lines = vec![top];
        for line in content.lines() {
            lines.push(format!("│ {}", line));
        }
        lines.push(bottom);

        lines.join("\n")
    }
}

impl Default for StatusRenderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_status_emoji() {
        assert_eq!(ExecutionStatus::Running.emoji(), "🔄");
        assert_eq!(ExecutionStatus::Success.emoji(), "✅");
        assert_eq!(ExecutionStatus::Failed.emoji(), "❌");
    }

    #[test]
    fn test_render_status_compact() {
        let renderer = StatusRenderer::new(RenderConfig::default());
        let result = renderer.render_status(ExecutionStatus::Running, "Processing request");
        assert!(result.contains("🔄"));
        assert!(result.contains("Processing request"));
    }

    #[test]
    fn test_render_thinking() {
        let renderer = StatusRenderer::new(RenderConfig::default());
        assert_eq!(renderer.render_thinking(), "🤔 Thinking...");
    }

    #[test]
    fn test_render_progress() {
        let renderer = StatusRenderer::new(RenderConfig::default());
        let result = renderer.render_progress(5, 10, "Loading");
        assert!(result.contains("50%"));
        assert!(result.contains("[5/10]"));
    }
}
