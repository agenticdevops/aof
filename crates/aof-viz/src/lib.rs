//! AOF Visualization - ASCII art for agent execution and workflows
//!
//! This crate provides ASCII visualization for:
//! - Agent execution status with spinners
//! - Flow node execution progress
//! - Tool call results
//! - Safety policy decisions
//!
//! Designed for mobile-friendly output on Telegram, Slack, and terminals.

mod status;
mod flow;
mod tools;
mod safety;
mod progress;

pub use status::{StatusRenderer, ExecutionStatus, StatusStyle};
pub use flow::{FlowRenderer, NodeStatus};
pub use tools::{ToolRenderer, ToolResult};
pub use safety::{SafetyRenderer, SafetyDecision};
pub use progress::{ProgressBar, Spinner, SpinnerType, StepProgress};

/// Render configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Maximum width for output (default: 40 for mobile)
    pub max_width: usize,
    /// Use Unicode box drawing characters
    pub use_unicode: bool,
    /// Use ANSI colors (terminal only)
    pub use_colors: bool,
    /// Compact mode (fewer lines)
    pub compact: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            max_width: 40,
            use_unicode: true,
            use_colors: false, // Safe for Telegram/Slack
            compact: true,
        }
    }
}

impl RenderConfig {
    /// Configuration for terminal output
    pub fn terminal() -> Self {
        Self {
            max_width: 80,
            use_unicode: true,
            use_colors: true,
            compact: false,
        }
    }

    /// Configuration for Telegram
    pub fn telegram() -> Self {
        Self {
            max_width: 35,
            use_unicode: true,
            use_colors: false,
            compact: true,
        }
    }

    /// Configuration for Slack
    pub fn slack() -> Self {
        Self {
            max_width: 50,
            use_unicode: true,
            use_colors: false,
            compact: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_config_defaults() {
        let config = RenderConfig::default();
        assert_eq!(config.max_width, 40);
        assert!(config.use_unicode);
        assert!(!config.use_colors);
        assert!(config.compact);
    }

    #[test]
    fn test_render_config_terminal() {
        let config = RenderConfig::terminal();
        assert_eq!(config.max_width, 80);
        assert!(config.use_colors);
    }

    #[test]
    fn test_render_config_telegram() {
        let config = RenderConfig::telegram();
        assert_eq!(config.max_width, 35);
        assert!(!config.use_colors);
        assert!(config.compact);
    }
}
