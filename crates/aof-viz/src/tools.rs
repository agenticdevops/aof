//! Tool call visualization

use crate::RenderConfig;

/// Result of a tool call
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Tool name (e.g., "kubectl", "docker")
    pub tool: String,
    /// Command or action performed
    pub command: String,
    /// Whether it succeeded
    pub success: bool,
    /// Output or error message
    pub output: String,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Renders tool call results
pub struct ToolRenderer {
    config: RenderConfig,
}

impl ToolRenderer {
    /// Create a new tool renderer
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Render a tool call header (before execution)
    pub fn render_call(&self, tool: &str, command: &str) -> String {
        if self.config.compact {
            format!("🔧 {} {}", tool, Self::truncate(command, 25))
        } else {
            format!("┌─ Tool: {} ──────────────\n│ {}\n└", tool, command)
        }
    }

    /// Render a tool result
    pub fn render_result(&self, result: &ToolResult) -> String {
        let status = if result.success { "✅" } else { "❌" };
        let duration = result.duration_ms
            .map(|d| format!(" ({}ms)", d))
            .unwrap_or_default();

        if self.config.compact {
            let output = Self::truncate(&result.output, self.config.max_width - 10);
            format!("{} {}{}\n{}", status, result.tool, duration, output)
        } else {
            let mut lines = vec![
                format!("┌─ {} {} ─{}", result.tool, status, duration),
                format!("│ $ {}", result.command),
                "├───────────────".to_string(),
            ];

            for line in result.output.lines().take(10) {
                lines.push(format!("│ {}", Self::truncate(line, self.config.max_width - 4)));
            }

            if result.output.lines().count() > 10 {
                lines.push("│ ... (truncated)".to_string());
            }

            lines.push("└".to_string());
            lines.join("\n")
        }
    }

    /// Render kubectl-style table output
    pub fn render_table(&self, headers: &[&str], rows: &[Vec<&str>]) -> String {
        if rows.is_empty() {
            return "No results".to_string();
        }

        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Compact mode: limit to 3 columns
        if self.config.compact && widths.len() > 3 {
            widths.truncate(3);
        }

        // Limit column width
        let max_col = if self.config.compact { 12 } else { 20 };
        widths = widths.iter().map(|w| (*w).min(max_col)).collect();

        let mut lines = Vec::new();

        // Header
        let header: Vec<String> = headers
            .iter()
            .take(widths.len())
            .enumerate()
            .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
            .collect();
        lines.push(header.join("  "));

        // Separator
        let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
        lines.push(sep.join("──"));

        // Rows
        for row in rows.iter().take(if self.config.compact { 5 } else { 15 }) {
            let cells: Vec<String> = row
                .iter()
                .take(widths.len())
                .enumerate()
                .map(|(i, c)| {
                    let truncated = Self::truncate(c, widths[i]);
                    format!("{:width$}", truncated, width = widths[i])
                })
                .collect();
            lines.push(cells.join("  "));
        }

        if rows.len() > (if self.config.compact { 5 } else { 15 }) {
            lines.push(format!("... {} more rows", rows.len() - 5));
        }

        lines.join("\n")
    }

    /// Render a code block
    pub fn render_code(&self, language: &str, code: &str) -> String {
        if self.config.compact {
            let truncated = Self::truncate(code, self.config.max_width * 3);
            format!("```{}\n{}\n```", language, truncated)
        } else {
            format!("```{}\n{}\n```", language, code)
        }
    }

    /// Truncate string to max length
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len > 3 {
            format!("{}...", &s[..max_len - 3])
        } else {
            s[..max_len].to_string()
        }
    }
}

impl Default for ToolRenderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_call() {
        let renderer = ToolRenderer::new(RenderConfig::default());
        let result = renderer.render_call("kubectl", "get pods -n production");
        assert!(result.contains("kubectl"));
        assert!(result.contains("🔧"));
    }

    #[test]
    fn test_render_result_success() {
        let renderer = ToolRenderer::new(RenderConfig::default());
        let result = ToolResult {
            tool: "kubectl".into(),
            command: "get pods".into(),
            success: true,
            output: "NAME        READY\npod-abc     1/1".into(),
            duration_ms: Some(150),
        };

        let output = renderer.render_result(&result);
        assert!(output.contains("✅"));
        assert!(output.contains("kubectl"));
        assert!(output.contains("150ms"));
    }

    #[test]
    fn test_render_result_failure() {
        let renderer = ToolRenderer::new(RenderConfig::default());
        let result = ToolResult {
            tool: "kubectl".into(),
            command: "delete pod".into(),
            success: false,
            output: "Error: forbidden".into(),
            duration_ms: Some(50),
        };

        let output = renderer.render_result(&result);
        assert!(output.contains("❌"));
    }

    #[test]
    fn test_render_table() {
        let renderer = ToolRenderer::new(RenderConfig::default());
        let headers = vec!["NAME", "READY", "STATUS"];
        let rows = vec![
            vec!["pod-1", "1/1", "Running"],
            vec!["pod-2", "2/2", "Running"],
        ];

        let output = renderer.render_table(&headers, &rows);
        assert!(output.contains("NAME"));
        assert!(output.contains("pod-1"));
        assert!(output.contains("Running"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(ToolRenderer::truncate("hello world", 8), "hello...");
        assert_eq!(ToolRenderer::truncate("short", 10), "short");
    }
}
