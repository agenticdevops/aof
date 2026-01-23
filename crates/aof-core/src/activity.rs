//! Agent Activity Events for TUI logging
//!
//! This module provides activity event types that agents emit during execution,
//! allowing the TUI to display real-time agent thinking, analyzing, and tool usage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

/// Activity event types for agent execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityType {
    /// Agent is processing/thinking
    Thinking,
    /// Agent is analyzing context or data
    Analyzing,
    /// Agent is calling an LLM
    LlmCall,
    /// Agent is waiting for LLM response
    LlmWaiting,
    /// LLM response received
    LlmResponse,
    /// Agent is discovering/loading tools
    ToolDiscovery,
    /// Agent is executing a tool
    ToolExecuting,
    /// Tool execution completed
    ToolComplete,
    /// Tool execution failed
    ToolFailed,
    /// Memory operation (read/write)
    Memory,
    /// MCP server communication
    McpCall,
    /// Validation (schema, output)
    Validation,
    /// Warning condition
    Warning,
    /// Error condition
    Error,
    /// Information message
    Info,
    /// Debug message
    Debug,
    /// Execution started
    Started,
    /// Execution completed
    Completed,
    /// Execution cancelled
    Cancelled,
}

impl ActivityType {
    /// Get emoji/icon for this activity type
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityType::Thinking => "🧠",
            ActivityType::Analyzing => "🔍",
            ActivityType::LlmCall => "📤",
            ActivityType::LlmWaiting => "⏳",
            ActivityType::LlmResponse => "📥",
            ActivityType::ToolDiscovery => "🔧",
            ActivityType::ToolExecuting => "⚙️",
            ActivityType::ToolComplete => "✓",
            ActivityType::ToolFailed => "✗",
            ActivityType::Memory => "💾",
            ActivityType::McpCall => "🔌",
            ActivityType::Validation => "📋",
            ActivityType::Warning => "⚠️",
            ActivityType::Error => "❌",
            ActivityType::Info => "ℹ️",
            ActivityType::Debug => "🐛",
            ActivityType::Started => "▶",
            ActivityType::Completed => "●",
            ActivityType::Cancelled => "⏹",
        }
    }

    /// Get ANSI color code for TUI display
    pub fn color(&self) -> &'static str {
        match self {
            ActivityType::Thinking | ActivityType::Analyzing => "cyan",
            ActivityType::LlmCall | ActivityType::LlmWaiting | ActivityType::LlmResponse => "blue",
            ActivityType::ToolDiscovery => "magenta",
            ActivityType::ToolExecuting => "yellow",
            ActivityType::ToolComplete | ActivityType::Completed => "green",
            ActivityType::ToolFailed | ActivityType::Error => "red",
            ActivityType::Memory => "cyan",
            ActivityType::McpCall => "magenta",
            ActivityType::Validation => "blue",
            ActivityType::Warning => "yellow",
            ActivityType::Info | ActivityType::Debug => "gray",
            ActivityType::Started => "green",
            ActivityType::Cancelled => "yellow",
        }
    }

    /// Get short label for this activity type
    pub fn label(&self) -> &'static str {
        match self {
            ActivityType::Thinking => "THINK",
            ActivityType::Analyzing => "ANALYZE",
            ActivityType::LlmCall => "LLM→",
            ActivityType::LlmWaiting => "WAIT",
            ActivityType::LlmResponse => "LLM←",
            ActivityType::ToolDiscovery => "TOOLS",
            ActivityType::ToolExecuting => "EXEC",
            ActivityType::ToolComplete => "DONE",
            ActivityType::ToolFailed => "FAIL",
            ActivityType::Memory => "MEM",
            ActivityType::McpCall => "MCP",
            ActivityType::Validation => "VALID",
            ActivityType::Warning => "WARN",
            ActivityType::Error => "ERROR",
            ActivityType::Info => "INFO",
            ActivityType::Debug => "DEBUG",
            ActivityType::Started => "START",
            ActivityType::Completed => "DONE",
            ActivityType::Cancelled => "CANCEL",
        }
    }
}

/// An activity event emitted during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// Type of activity
    pub activity_type: ActivityType,
    /// Human-readable message
    pub message: String,
    /// Timestamp when the activity occurred
    pub timestamp: DateTime<Utc>,
    /// Optional additional details (e.g., tool name, duration)
    pub details: Option<ActivityDetails>,
}

/// Additional details for an activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetails {
    /// Tool name (for tool-related activities)
    pub tool_name: Option<String>,
    /// Tool arguments (for tool execution)
    pub tool_args: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Token counts
    pub tokens: Option<TokenCount>,
    /// Error message
    pub error: Option<String>,
    /// Additional key-value metadata
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Token count details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    pub input: u32,
    pub output: u32,
}

impl ActivityEvent {
    /// Create a new activity event
    pub fn new(activity_type: ActivityType, message: impl Into<String>) -> Self {
        Self {
            activity_type,
            message: message.into(),
            timestamp: Utc::now(),
            details: None,
        }
    }

    /// Add details to this event
    pub fn with_details(mut self, details: ActivityDetails) -> Self {
        self.details = Some(details);
        self
    }

    /// Add tool name
    pub fn with_tool(mut self, tool_name: impl Into<String>) -> Self {
        let details = self.details.get_or_insert(ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: None,
        });
        details.tool_name = Some(tool_name.into());
        self
    }

    /// Add tool arguments
    pub fn with_args(mut self, args: impl Into<String>) -> Self {
        let details = self.details.get_or_insert(ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: None,
        });
        details.tool_args = Some(args.into());
        self
    }

    /// Add duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        let details = self.details.get_or_insert(ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: None,
        });
        details.duration_ms = Some(duration_ms);
        self
    }

    /// Add token counts
    pub fn with_tokens(mut self, input: u32, output: u32) -> Self {
        let details = self.details.get_or_insert(ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: None,
        });
        details.tokens = Some(TokenCount { input, output });
        self
    }

    /// Add error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        let details = self.details.get_or_insert(ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: None,
            metadata: None,
        });
        details.error = Some(error.into());
        self
    }

    /// Format for display in TUI
    pub fn format_display(&self) -> String {
        let icon = self.activity_type.icon();
        let label = self.activity_type.label();
        let time = self.timestamp.format("%H:%M:%S");

        let mut output = format!("[{}] {} {}: {}", time, icon, label, self.message);

        if let Some(ref details) = self.details {
            if let Some(ref tool) = details.tool_name {
                output.push_str(&format!(" [{}]", tool));
            }
            if let Some(duration) = details.duration_ms {
                output.push_str(&format!(" ({}ms)", duration));
            }
            if let Some(ref tokens) = details.tokens {
                output.push_str(&format!(" [{}→{}]", tokens.input, tokens.output));
            }
        }

        output
    }

    /// Format for display without timestamp (compact)
    pub fn format_compact(&self) -> String {
        let icon = self.activity_type.icon();

        let mut output = format!("{} {}", icon, self.message);

        if let Some(ref details) = self.details {
            if let Some(ref tool) = details.tool_name {
                output.push_str(&format!(" [{}]", tool));
            }
            if let Some(duration) = details.duration_ms {
                output.push_str(&format!(" ({}ms)", duration));
            }
        }

        output
    }
}

// Convenience constructors for common activity types
impl ActivityEvent {
    pub fn thinking(message: impl Into<String>) -> Self {
        Self::new(ActivityType::Thinking, message)
    }

    pub fn analyzing(message: impl Into<String>) -> Self {
        Self::new(ActivityType::Analyzing, message)
    }

    pub fn llm_call(message: impl Into<String>) -> Self {
        Self::new(ActivityType::LlmCall, message)
    }

    pub fn llm_waiting() -> Self {
        Self::new(ActivityType::LlmWaiting, "Waiting for LLM response...")
    }

    pub fn llm_response(input_tokens: u32, output_tokens: u32) -> Self {
        Self::new(ActivityType::LlmResponse, "Received LLM response")
            .with_tokens(input_tokens, output_tokens)
    }

    pub fn tool_discovery(count: usize) -> Self {
        Self::new(
            ActivityType::ToolDiscovery,
            format!("Discovered {} available tools", count),
        )
    }

    pub fn tool_executing(tool_name: impl Into<String>, args: Option<String>) -> Self {
        let name = tool_name.into();
        let msg = format!("Executing tool: {}", name);
        let mut event = Self::new(ActivityType::ToolExecuting, msg).with_tool(&name);
        if let Some(a) = args {
            // Truncate args for display
            let truncated = if a.len() > 100 {
                format!("{}...", &a[..100])
            } else {
                a
            };
            event = event.with_args(truncated);
        }
        event
    }

    pub fn tool_complete(tool_name: impl Into<String>, duration_ms: u64) -> Self {
        let name = tool_name.into();
        Self::new(ActivityType::ToolComplete, format!("Tool completed: {}", name))
            .with_tool(name)
            .with_duration(duration_ms)
    }

    pub fn tool_failed(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        let name = tool_name.into();
        Self::new(ActivityType::ToolFailed, format!("Tool failed: {}", name))
            .with_tool(name)
            .with_error(error)
    }

    pub fn memory(operation: &str, key: &str) -> Self {
        Self::new(
            ActivityType::Memory,
            format!("Memory {}: {}", operation, key),
        )
    }

    pub fn mcp_call(server: &str, method: &str) -> Self {
        Self::new(ActivityType::McpCall, format!("MCP {} → {}", server, method))
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(ActivityType::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ActivityType::Error, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ActivityType::Info, message)
    }

    pub fn started(agent_name: &str) -> Self {
        Self::new(
            ActivityType::Started,
            format!("Starting execution for agent: {}", agent_name),
        )
    }

    pub fn completed(duration_ms: u64) -> Self {
        Self::new(
            ActivityType::Completed,
            format!("Execution completed in {}ms", duration_ms),
        )
        .with_duration(duration_ms)
    }

    pub fn cancelled() -> Self {
        Self::new(ActivityType::Cancelled, "Execution cancelled by user")
    }
}

/// Activity logger that can be passed to executors
#[derive(Clone)]
pub struct ActivityLogger {
    sender: Sender<ActivityEvent>,
}

impl ActivityLogger {
    /// Create a new activity logger with a channel sender
    pub fn new(sender: Sender<ActivityEvent>) -> Self {
        Self { sender }
    }

    /// Log an activity event
    pub fn log(&self, event: ActivityEvent) {
        // Ignore send errors (receiver may be dropped)
        let _ = self.sender.send(event);
    }

    /// Log a thinking activity
    pub fn thinking(&self, message: impl Into<String>) {
        self.log(ActivityEvent::thinking(message));
    }

    /// Log an analyzing activity
    pub fn analyzing(&self, message: impl Into<String>) {
        self.log(ActivityEvent::analyzing(message));
    }

    /// Log an LLM call
    pub fn llm_call(&self, message: impl Into<String>) {
        self.log(ActivityEvent::llm_call(message));
    }

    /// Log LLM waiting
    pub fn llm_waiting(&self) {
        self.log(ActivityEvent::llm_waiting());
    }

    /// Log LLM response
    pub fn llm_response(&self, input_tokens: u32, output_tokens: u32) {
        self.log(ActivityEvent::llm_response(input_tokens, output_tokens));
    }

    /// Log tool execution start
    pub fn tool_executing(&self, tool_name: impl Into<String>, args: Option<String>) {
        self.log(ActivityEvent::tool_executing(tool_name, args));
    }

    /// Log tool completion
    pub fn tool_complete(&self, tool_name: impl Into<String>, duration_ms: u64) {
        self.log(ActivityEvent::tool_complete(tool_name, duration_ms));
    }

    /// Log tool failure
    pub fn tool_failed(&self, tool_name: impl Into<String>, error: impl Into<String>) {
        self.log(ActivityEvent::tool_failed(tool_name, error));
    }

    /// Log warning
    pub fn warning(&self, message: impl Into<String>) {
        self.log(ActivityEvent::warning(message));
    }

    /// Log error
    pub fn error(&self, message: impl Into<String>) {
        self.log(ActivityEvent::error(message));
    }

    /// Log info
    pub fn info(&self, message: impl Into<String>) {
        self.log(ActivityEvent::info(message));
    }

    /// Log execution started
    pub fn started(&self, agent_name: &str) {
        self.log(ActivityEvent::started(agent_name));
    }

    /// Log execution completed
    pub fn completed(&self, duration_ms: u64) {
        self.log(ActivityEvent::completed(duration_ms));
    }

    /// Log execution cancelled
    pub fn cancelled(&self) {
        self.log(ActivityEvent::cancelled());
    }
}

/// No-op activity logger for when activity logging is disabled
pub struct NoopActivityLogger;

impl NoopActivityLogger {
    pub fn log(&self, _event: ActivityEvent) {}
    pub fn thinking(&self, _message: impl Into<String>) {}
    pub fn analyzing(&self, _message: impl Into<String>) {}
    pub fn llm_call(&self, _message: impl Into<String>) {}
    pub fn llm_waiting(&self) {}
    pub fn llm_response(&self, _input_tokens: u32, _output_tokens: u32) {}
    pub fn tool_executing(&self, _tool_name: impl Into<String>, _args: Option<String>) {}
    pub fn tool_complete(&self, _tool_name: impl Into<String>, _duration_ms: u64) {}
    pub fn tool_failed(&self, _tool_name: impl Into<String>, _error: impl Into<String>) {}
    pub fn warning(&self, _message: impl Into<String>) {}
    pub fn error(&self, _message: impl Into<String>) {}
    pub fn info(&self, _message: impl Into<String>) {}
    pub fn started(&self, _agent_name: &str) {}
    pub fn completed(&self, _duration_ms: u64) {}
    pub fn cancelled(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_event_creation() {
        let event = ActivityEvent::thinking("Processing user request");
        assert_eq!(event.activity_type, ActivityType::Thinking);
        assert_eq!(event.message, "Processing user request");
    }

    #[test]
    fn test_activity_event_with_details() {
        let event = ActivityEvent::tool_executing("kubectl", Some("get pods".to_string()));
        assert!(event.details.is_some());
        let details = event.details.unwrap();
        assert_eq!(details.tool_name, Some("kubectl".to_string()));
    }

    #[test]
    fn test_activity_event_formatting() {
        let event = ActivityEvent::tool_complete("kubectl", 234);
        let formatted = event.format_compact();
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("234ms"));
    }

    #[test]
    fn test_activity_type_icons() {
        assert_eq!(ActivityType::Thinking.icon(), "🧠");
        assert_eq!(ActivityType::ToolExecuting.icon(), "⚙️");
        assert_eq!(ActivityType::Error.icon(), "❌");
    }
}
