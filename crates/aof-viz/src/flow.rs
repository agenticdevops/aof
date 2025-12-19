//! Flow visualization for AgentFlow execution

use crate::RenderConfig;

/// Status of a flow node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Not yet executed
    Pending,
    /// Currently executing
    Active,
    /// Completed successfully
    Complete,
    /// Skipped (condition not met)
    Skipped,
    /// Failed with error
    Failed,
}

impl NodeStatus {
    /// Get emoji for node status
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Active => "◉",
            Self::Complete => "●",
            Self::Skipped => "◌",
            Self::Failed => "✖",
        }
    }

    /// Get ASCII representation
    pub fn ascii(&self) -> &'static str {
        match self {
            Self::Pending => "[ ]",
            Self::Active => "[>]",
            Self::Complete => "[x]",
            Self::Skipped => "[-]",
            Self::Failed => "[!]",
        }
    }
}

/// Flow node for rendering
#[derive(Debug, Clone)]
pub struct FlowNode {
    /// Node ID
    pub id: String,
    /// Node type (Agent, Fleet, HTTP, etc.)
    pub node_type: String,
    /// Node label/name
    pub label: String,
    /// Current status
    pub status: NodeStatus,
    /// Duration in milliseconds (if completed)
    pub duration_ms: Option<u64>,
}

/// Renders flow execution progress
pub struct FlowRenderer {
    config: RenderConfig,
}

impl FlowRenderer {
    /// Create a new flow renderer
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Render a linear flow (simple list)
    pub fn render_linear(&self, nodes: &[FlowNode]) -> String {
        let mut lines = Vec::new();

        for (i, node) in nodes.iter().enumerate() {
            let status_icon = node.status.emoji();
            let connector = if i < nodes.len() - 1 { "│" } else { " " };

            let duration = node.duration_ms
                .map(|d| format!(" ({}ms)", d))
                .unwrap_or_default();

            if self.config.compact {
                lines.push(format!("{} {}{}", status_icon, node.label, duration));
            } else {
                lines.push(format!(
                    "{} {} [{}]{}",
                    status_icon,
                    node.label,
                    node.node_type,
                    duration
                ));
            }

            if i < nodes.len() - 1 {
                lines.push(format!("{}  ↓", connector));
            }
        }

        lines.join("\n")
    }

    /// Render flow as compact inline
    pub fn render_inline(&self, nodes: &[FlowNode]) -> String {
        let parts: Vec<String> = nodes
            .iter()
            .map(|n| format!("{}{}", n.status.emoji(), n.label))
            .collect();

        parts.join(" → ")
    }

    /// Render flow header
    pub fn render_header(&self, flow_name: &str, trigger_type: &str) -> String {
        if self.config.compact {
            format!("📋 {} ({})", flow_name, trigger_type)
        } else {
            format!(
                "╭─ Flow: {} ─╮\n│ Trigger: {} │\n╰{}╯",
                flow_name,
                trigger_type,
                "─".repeat(flow_name.len() + trigger_type.len() + 10)
            )
        }
    }

    /// Render flow summary
    pub fn render_summary(&self, nodes: &[FlowNode]) -> String {
        let total = nodes.len();
        let complete = nodes.iter().filter(|n| n.status == NodeStatus::Complete).count();
        let failed = nodes.iter().filter(|n| n.status == NodeStatus::Failed).count();
        let skipped = nodes.iter().filter(|n| n.status == NodeStatus::Skipped).count();

        let total_duration: u64 = nodes.iter().filter_map(|n| n.duration_ms).sum();

        if self.config.compact {
            format!(
                "Done: {}/{} | Failed: {} | Time: {}ms",
                complete, total, failed, total_duration
            )
        } else {
            format!(
                "━━━ Summary ━━━\n✓ Complete: {}\n✖ Failed: {}\n◌ Skipped: {}\n⏱ Duration: {}ms",
                complete, failed, skipped, total_duration
            )
        }
    }

    /// Render a branching flow (for conditionals)
    pub fn render_branch(&self, condition: &str, true_branch: &str, false_branch: Option<&str>) -> String {
        let mut lines = vec![
            format!("◇ {}", condition),
            "├─ ✓ true:".to_string(),
            format!("│    {}", true_branch),
        ];

        if let Some(false_b) = false_branch {
            lines.push("├─ ✗ false:".to_string());
            lines.push(format!("│    {}", false_b));
        }

        lines.push("└".to_string());
        lines.join("\n")
    }
}

impl Default for FlowRenderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_nodes() -> Vec<FlowNode> {
        vec![
            FlowNode {
                id: "start".into(),
                node_type: "Trigger".into(),
                label: "HTTP Webhook".into(),
                status: NodeStatus::Complete,
                duration_ms: Some(10),
            },
            FlowNode {
                id: "agent1".into(),
                node_type: "Agent".into(),
                label: "k8s-status".into(),
                status: NodeStatus::Active,
                duration_ms: None,
            },
            FlowNode {
                id: "notify".into(),
                node_type: "Slack".into(),
                label: "Send Alert".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            },
        ]
    }

    #[test]
    fn test_render_linear() {
        let renderer = FlowRenderer::new(RenderConfig::default());
        let result = renderer.render_linear(&sample_nodes());

        assert!(result.contains("HTTP Webhook"));
        assert!(result.contains("k8s-status"));
        assert!(result.contains("Send Alert"));
    }

    #[test]
    fn test_render_inline() {
        let renderer = FlowRenderer::new(RenderConfig::default());
        let result = renderer.render_inline(&sample_nodes());

        assert!(result.contains("→"));
        assert!(result.contains("HTTP Webhook"));
    }

    #[test]
    fn test_render_summary() {
        let renderer = FlowRenderer::new(RenderConfig::default());
        let nodes = sample_nodes();
        let result = renderer.render_summary(&nodes);

        assert!(result.contains("1/3") || result.contains("Complete: 1"));
    }

    #[test]
    fn test_node_status_emoji() {
        assert_eq!(NodeStatus::Active.emoji(), "◉");
        assert_eq!(NodeStatus::Complete.emoji(), "●");
        assert_eq!(NodeStatus::Failed.emoji(), "✖");
    }
}
