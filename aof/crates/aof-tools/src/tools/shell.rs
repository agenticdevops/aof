//! Shell Tool
//!
//! Execute shell commands with safety controls and output capture.
//!
//! ## Security Considerations
//!
//! - Commands run in a subprocess with captured stdout/stderr
//! - Timeout protection prevents runaway processes
//! - Working directory can be specified for sandboxing
//!
//! ## MCP Alternative
//!
//! For MCP-based shell execution, you can use custom MCP servers
//! or the built-in shell tool in stdio mode.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

use super::common::tool_config_with_timeout;

/// Shell command execution tool
pub struct ShellTool {
    config: ToolConfig,
    /// Default shell to use
    shell: String,
    /// Allowed commands (empty = allow all)
    allowed_commands: Vec<String>,
    /// Blocked commands
    blocked_commands: Vec<String>,
}

impl ShellTool {
    /// Create a new shell tool with default settings
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for command execution"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 60)",
                    "default": 60
                },
                "env": {
                    "type": "object",
                    "description": "Additional environment variables",
                    "additionalProperties": { "type": "string" }
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "shell",
                "Execute a shell command. Returns stdout, stderr, and exit code.",
                parameters,
                120, // 2 minute max timeout
            ),
            shell: Self::detect_shell(),
            allowed_commands: vec![],
            blocked_commands: Self::default_blocked_commands(),
        }
    }

    /// Create with custom allowed commands (whitelist mode)
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }

    /// Create with additional blocked commands
    pub fn with_blocked_commands(mut self, commands: Vec<String>) -> Self {
        self.blocked_commands.extend(commands);
        self
    }

    /// Create with custom shell
    pub fn with_shell(mut self, shell: String) -> Self {
        self.shell = shell;
        self
    }

    fn detect_shell() -> String {
        std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })
    }

    fn default_blocked_commands() -> Vec<String> {
        vec![
            // Dangerous system commands
            "rm -rf /".to_string(),
            "mkfs".to_string(),
            "dd if=/dev".to_string(),
            ":(){:|:&};:".to_string(), // Fork bomb
            // Network attacks
            "nc -l".to_string(),   // Netcat listener (could be used for reverse shell)
        ]
    }

    fn is_command_allowed(&self, command: &str) -> Result<(), String> {
        // Check blocked commands
        for blocked in &self.blocked_commands {
            if command.contains(blocked) {
                return Err(format!("Command contains blocked pattern: {}", blocked));
            }
        }

        // Check allowed commands if whitelist mode is enabled
        if !self.allowed_commands.is_empty() {
            let cmd_name = command.split_whitespace().next().unwrap_or("");
            if !self.allowed_commands.iter().any(|a| cmd_name.starts_with(a)) {
                return Err(format!("Command not in allowed list: {}", cmd_name));
            }
        }

        Ok(())
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ShellTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(60);
        let env_vars: HashMap<String, String> = input
            .get_arg("env")
            .unwrap_or_default();

        // Security check
        if let Err(e) = self.is_command_allowed(&command) {
            warn!(command = %command, error = %e, "Command blocked by security policy");
            return Ok(ToolResult::error(format!("Security policy violation: {}", e)));
        }

        debug!(command = %command, shell = %self.shell, "Executing shell command");

        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", &command]);
            c
        } else {
            let mut c = Command::new(&self.shell);
            c.args(["-c", &command]);
            c
        };

        // Set working directory if specified
        if let Some(dir) = &working_dir {
            cmd.current_dir(dir);
        }

        // Add environment variables
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }

        // Capture output
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn and wait with timeout
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to spawn command: {}", e)));
            }
        };

        let output = match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(ToolResult::error(format!("Command execution failed: {}", e)));
            }
            Err(_) => {
                return Ok(ToolResult::error(format!(
                    "Command timed out after {} seconds",
                    timeout_secs
                )));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        Ok(ToolResult::success(serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "success": success,
            "command": command
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_echo() {
        let tool = ShellTool::new();
        let input = ToolInput::new(serde_json::json!({
            "command": "echo 'Hello, World!'"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert!(result.data["stdout"].as_str().unwrap().contains("Hello, World!"));
        assert_eq!(result.data["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_shell_exit_code() {
        let tool = ShellTool::new();
        let input = ToolInput::new(serde_json::json!({
            "command": "exit 42"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success); // Tool succeeded, but command returned non-zero
        assert_eq!(result.data["exit_code"], 42);
        assert!(!result.data["success"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_shell_blocked_command() {
        let tool = ShellTool::new();
        let input = ToolInput::new(serde_json::json!({
            "command": "rm -rf /"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Security policy"));
    }

    #[tokio::test]
    async fn test_shell_with_env() {
        let tool = ShellTool::new();
        let input = ToolInput::new(serde_json::json!({
            "command": "echo $MY_VAR",
            "env": {
                "MY_VAR": "test_value"
            }
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert!(result.data["stdout"].as_str().unwrap().contains("test_value"));
    }

    #[tokio::test]
    async fn test_shell_timeout() {
        let tool = ShellTool::new();
        let input = ToolInput::new(serde_json::json!({
            "command": "sleep 10",
            "timeout_secs": 1
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("timed out"));
    }
}
