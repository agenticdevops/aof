//! Tool implementations
//!
//! Each tool category is in its own module and can be enabled/disabled via feature flags.

#[cfg(feature = "file")]
pub mod file;

#[cfg(feature = "shell")]
pub mod shell;

#[cfg(feature = "kubectl")]
pub mod kubectl;

#[cfg(feature = "docker")]
pub mod docker;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "terraform")]
pub mod terraform;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "observability")]
pub mod observability;

/// Common utilities for tool implementations
pub mod common {
    use aof_core::ToolConfig;
    use std::collections::HashMap;

    /// Create a standard JSON schema for a tool with required and optional parameters
    pub fn create_schema(
        properties: serde_json::Value,
        required: Vec<&str>,
    ) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    /// Create a basic tool config
    pub fn tool_config(
        name: &str,
        description: &str,
        parameters: serde_json::Value,
    ) -> ToolConfig {
        ToolConfig {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            tool_type: aof_core::ToolType::Custom,
            timeout_secs: 30,
            extra: HashMap::new(),
        }
    }

    /// Create a tool config with custom timeout
    pub fn tool_config_with_timeout(
        name: &str,
        description: &str,
        parameters: serde_json::Value,
        timeout_secs: u64,
    ) -> ToolConfig {
        ToolConfig {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            tool_type: aof_core::ToolType::Custom,
            timeout_secs,
            extra: HashMap::new(),
        }
    }

    /// Execute a command and return structured output
    pub async fn execute_command(
        program: &str,
        args: &[&str],
        working_dir: Option<&str>,
        timeout_secs: u64,
    ) -> Result<CommandOutput, String> {
        use tokio::process::Command;
        use std::time::Duration;

        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Capture output
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn {}: {}", program, e))?;

        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| format!("Command timed out after {}s", timeout_secs))?
        .map_err(|e| format!("Command failed: {}", e))?;

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }

    /// Command execution output
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct CommandOutput {
        pub exit_code: i32,
        pub stdout: String,
        pub stderr: String,
        pub success: bool,
    }

    impl CommandOutput {
        pub fn to_json(&self) -> serde_json::Value {
            serde_json::to_value(self).unwrap_or_default()
        }
    }
}
