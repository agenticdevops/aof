//! Command parsing and execution
//!
//! This module handles parsing natural language and slash commands
//! into structured TriggerCommand objects that can be executed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::platforms::TriggerMessage;

/// Command parsing errors
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Invalid command format: {0}")]
    InvalidFormat(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Invalid target: {0}")]
    InvalidTarget(String),
}

/// Command type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandType {
    /// Run an agent or task
    Run,

    /// Create a new agent/fleet/flow
    Create,

    /// Get status of task/agent
    Status,

    /// Cancel a running task
    Cancel,

    /// List agents/tasks/fleets
    List,

    /// Show help information
    Help,

    /// Show system info
    Info,
}

impl CommandType {
    /// Parse command type from string
    pub fn from_str(s: &str) -> Result<Self, CommandError> {
        match s.to_lowercase().as_str() {
            "run" | "execute" | "start" => Ok(Self::Run),
            "create" | "new" | "spawn" => Ok(Self::Create),
            "status" | "check" | "info" => Ok(Self::Status),
            "cancel" | "stop" | "abort" => Ok(Self::Cancel),
            "list" | "ls" | "show" => Ok(Self::List),
            "help" | "h" => Ok(Self::Help),
            _ => Err(CommandError::UnknownCommand(s.to_string())),
        }
    }

    /// Get command description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Run => "Run an agent or execute a task",
            Self::Create => "Create a new agent, fleet, or flow",
            Self::Status => "Check status of a task or agent",
            Self::Cancel => "Cancel a running task",
            Self::List => "List agents, tasks, or fleets",
            Self::Help => "Show help information",
            Self::Info => "Show system information",
        }
    }
}

/// Target type for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerTarget {
    /// Single agent
    Agent,

    /// Individual task
    Task,

    /// Fleet of agents
    Fleet,

    /// Workflow/flow
    Flow,
}

impl TriggerTarget {
    /// Parse target from string
    pub fn from_str(s: &str) -> Result<Self, CommandError> {
        match s.to_lowercase().as_str() {
            "agent" | "ag" => Ok(Self::Agent),
            "task" | "t" => Ok(Self::Task),
            "fleet" | "f" => Ok(Self::Fleet),
            "flow" | "workflow" | "w" => Ok(Self::Flow),
            _ => Err(CommandError::InvalidTarget(s.to_string())),
        }
    }
}

/// Command execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    /// User who issued the command
    pub user_id: String,

    /// Username for display
    pub username: Option<String>,

    /// Channel/chat where command was issued
    pub channel_id: String,

    /// Platform (telegram, slack, etc.)
    pub platform: String,

    /// Message ID (for threading responses)
    pub message_id: String,

    /// Thread ID if in thread
    pub thread_id: Option<String>,

    /// Additional context metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CommandContext {
    /// Create context from TriggerMessage
    pub fn from_message(msg: &TriggerMessage) -> Self {
        Self {
            user_id: msg.user.id.clone(),
            username: msg.user.username.clone(),
            channel_id: msg.channel_id.clone(),
            platform: msg.platform.clone(),
            message_id: msg.id.clone(),
            thread_id: msg.thread_id.clone(),
            metadata: msg.metadata.clone(),
        }
    }
}

/// Parsed trigger command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCommand {
    /// Command type
    pub command_type: CommandType,

    /// Target type
    pub target: TriggerTarget,

    /// Command arguments
    pub args: Vec<String>,

    /// Named parameters
    pub params: HashMap<String, String>,

    /// Execution context
    pub context: CommandContext,
}

impl TriggerCommand {
    /// Create a new trigger command
    pub fn new(
        command_type: CommandType,
        target: TriggerTarget,
        args: Vec<String>,
        context: CommandContext,
    ) -> Self {
        Self {
            command_type,
            target,
            args,
            params: HashMap::new(),
            context,
        }
    }

    /// Add a named parameter
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.params.insert(key, value);
        self
    }

    /// Get required argument by index
    pub fn get_arg(&self, index: usize) -> Result<&str, CommandError> {
        self.args
            .get(index)
            .map(|s| s.as_str())
            .ok_or_else(|| CommandError::MissingArgument(format!("argument at index {}", index)))
    }

    /// Get optional parameter
    pub fn get_param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Parse command from message text
    ///
    /// Supported formats:
    /// - `/run agent agent-name task description`
    /// - `/create fleet fleet-name --size=5`
    /// - `/status task task-id`
    /// - `/list agents`
    /// - `/help`
    pub fn parse(msg: &TriggerMessage) -> Result<Self, CommandError> {
        let text = msg.text.trim();

        // Must start with /
        if !text.starts_with('/') {
            return Err(CommandError::InvalidFormat(
                "Command must start with /".to_string(),
            ));
        }

        let parts: Vec<&str> = text[1..].split_whitespace().collect();

        if parts.is_empty() {
            return Err(CommandError::InvalidFormat("Empty command".to_string()));
        }

        let command_type = CommandType::from_str(parts[0])?;
        let context = CommandContext::from_message(msg);

        // Handle help command (no target needed)
        if command_type == CommandType::Help {
            return Ok(Self::new(
                command_type,
                TriggerTarget::Agent, // Default, unused for help
                Vec::new(),
                context,
            ));
        }

        // Parse target (second part)
        if parts.len() < 2 {
            return Err(CommandError::MissingArgument("target".to_string()));
        }

        let target = TriggerTarget::from_str(parts[1])?;

        // Remaining parts are arguments
        let mut args = Vec::new();
        let mut params = HashMap::new();

        for part in parts.iter().skip(2) {
            if part.starts_with("--") {
                // Named parameter: --key=value
                if let Some((key, value)) = part[2..].split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                } else {
                    // Flag: --key (value = "true")
                    params.insert(part[2..].to_string(), "true".to_string());
                }
            } else {
                args.push(part.to_string());
            }
        }

        let mut cmd = Self::new(command_type, target, args, context);
        cmd.params = params;

        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platforms::TriggerUser;

    fn create_test_message(text: &str) -> TriggerMessage {
        let user = TriggerUser {
            id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            is_bot: false,
        };

        TriggerMessage::new(
            "msg123".to_string(),
            "telegram".to_string(),
            "chat456".to_string(),
            user,
            text.to_string(),
        )
    }

    #[test]
    fn test_parse_run_command() {
        let msg = create_test_message("/run agent my-agent do something cool");
        let cmd = TriggerCommand::parse(&msg).unwrap();

        assert_eq!(cmd.command_type, CommandType::Run);
        assert_eq!(cmd.target, TriggerTarget::Agent);
        assert_eq!(cmd.args.len(), 4);
        assert_eq!(cmd.get_arg(0).unwrap(), "my-agent");
    }

    #[test]
    fn test_parse_with_params() {
        let msg = create_test_message("/create fleet my-fleet --size=5 --region=us-east");
        let cmd = TriggerCommand::parse(&msg).unwrap();

        assert_eq!(cmd.command_type, CommandType::Create);
        assert_eq!(cmd.target, TriggerTarget::Fleet);
        assert_eq!(cmd.get_param("size").unwrap(), "5");
        assert_eq!(cmd.get_param("region").unwrap(), "us-east");
    }

    #[test]
    fn test_parse_help() {
        let msg = create_test_message("/help");
        let cmd = TriggerCommand::parse(&msg).unwrap();

        assert_eq!(cmd.command_type, CommandType::Help);
    }

    #[test]
    fn test_parse_invalid_command() {
        let msg = create_test_message("/invalid agent test");
        assert!(TriggerCommand::parse(&msg).is_err());
    }

    #[test]
    fn test_command_type_from_str() {
        assert_eq!(CommandType::from_str("run").unwrap(), CommandType::Run);
        assert_eq!(CommandType::from_str("execute").unwrap(), CommandType::Run);
        assert_eq!(CommandType::from_str("list").unwrap(), CommandType::List);
        assert!(CommandType::from_str("invalid").is_err());
    }

    #[test]
    fn test_target_from_str() {
        assert_eq!(TriggerTarget::from_str("agent").unwrap(), TriggerTarget::Agent);
        assert_eq!(TriggerTarget::from_str("task").unwrap(), TriggerTarget::Task);
        assert_eq!(TriggerTarget::from_str("fleet").unwrap(), TriggerTarget::Fleet);
        assert!(TriggerTarget::from_str("invalid").is_err());
    }
}
