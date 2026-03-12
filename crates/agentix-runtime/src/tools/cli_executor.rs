//! CLI and shell tool executor for the ReAct loop.
//!
//! Executes `DirectoryToolType::Cli` and `DirectoryToolType::Shell` tools
//! as async subprocesses with configurable timeout enforcement.

use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use agentix_core::{DirectoryToolType, ToolEntry};

use crate::executor::react_loop::ToolExecutor;

/// Default tool execution timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Executes CLI and shell tools as async subprocesses.
///
/// Features:
/// - Async process execution (non-blocking)
/// - Configurable timeout (default: 30s)
/// - `{{variable}}` template substitution in command and args
/// - stdout returned on success; stderr appended on failure
/// - MCP tools return a descriptive error (implemented in plan 14-03)
pub struct CliToolExecutor {
    /// Wall-clock timeout for a single tool execution.
    timeout: Duration,
}

impl CliToolExecutor {
    /// Create a new executor with the default 30-second timeout.
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Create a new executor with a custom timeout in seconds.
    pub fn with_timeout(secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(secs),
        }
    }
}

impl Default for CliToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ToolExecutor for CliToolExecutor {
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        match tool.tool_type {
            DirectoryToolType::Mcp => {
                Err(format!(
                    "MCP tool '{}' not yet connected. Wire up agentix-mcp in Phase 14 plan 14-03.",
                    tool.name
                ))
            }

            DirectoryToolType::Cli | DirectoryToolType::Shell => {
                let command_str = tool
                    .command
                    .as_deref()
                    .ok_or_else(|| format!("Tool '{}' has no command defined", tool.name))?;

                // Substitute {{var}} templates in command string
                let resolved_command = substitute_templates(command_str, &input);

                // Substitute {{var}} templates in each arg
                let resolved_args: Vec<String> = tool
                    .args
                    .iter()
                    .map(|a| substitute_templates(a, &input))
                    .collect();

                execute_with_timeout(
                    &resolved_command,
                    &resolved_args,
                    self.timeout,
                    &tool.name,
                )
                .await
            }
        }
    }
}

/// Execute a shell command with the given timeout.
///
/// The command is always executed via `sh -c` to support pipes and redirects.
async fn execute_with_timeout(
    command: &str,
    args: &[String],
    timeout_duration: Duration,
    tool_name: &str,
) -> Result<String, String> {
    // Build the full command string (args joined to command)
    let full_cmd = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    let run_future = async {
        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .output()
            .await
    };

    match timeout(timeout_duration, run_future).await {
        Err(_elapsed) => Err(format!(
            "Tool '{}' timed out after {}s",
            tool_name,
            timeout_duration.as_secs()
        )),
        Ok(Err(io_err)) => Err(format!(
            "Tool '{}' failed to start: {}",
            tool_name, io_err
        )),
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if output.status.success() {
                // Return stdout; if empty, return stderr (some tools write to stderr on success)
                Ok(if stdout.is_empty() {
                    stderr
                } else {
                    stdout
                })
            } else {
                let exit_code = output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let error_msg = if !stderr.is_empty() {
                    format!(
                        "Tool '{}' exited with code {}: {}",
                        tool_name, exit_code, stderr
                    )
                } else if !stdout.is_empty() {
                    format!(
                        "Tool '{}' exited with code {}: {}",
                        tool_name, exit_code, stdout
                    )
                } else {
                    format!("Tool '{}' exited with code {}", tool_name, exit_code)
                };
                Err(error_msg)
            }
        }
    }
}

/// Substitute `{{variable}}` placeholders in a template string using JSON input.
///
/// - String values are substituted directly.
/// - Non-string values (numbers, booleans) are JSON-serialized.
/// - Unknown variables are left as-is (not substituted).
fn substitute_templates(template: &str, input: &serde_json::Value) -> String {
    let mut result = template.to_string();

    if let Some(obj) = input.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
    }

    result
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_substitute_string_var() {
        let input = serde_json::json!({"name": "world"});
        let result = substitute_templates("hello {{name}}", &input);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_substitute_numeric_var() {
        let input = serde_json::json!({"count": 42});
        let result = substitute_templates("count={{count}}", &input);
        assert_eq!(result, "count=42");
    }

    #[test]
    fn test_substitute_unknown_var_unchanged() {
        let input = serde_json::json!({});
        let result = substitute_templates("hello {{unknown}}", &input);
        assert_eq!(result, "hello {{unknown}}");
    }

    #[test]
    fn test_substitute_multiple_vars() {
        let input = serde_json::json!({"a": "foo", "b": "bar"});
        let result = substitute_templates("{{a}}-{{b}}", &input);
        assert_eq!(result, "foo-bar");
    }
}
