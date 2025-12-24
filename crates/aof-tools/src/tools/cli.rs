//! Unified CLI Tools
//!
//! Simple, unified tools for CLI operations where the LLM constructs the full command.
//! These are the recommended tools for DevOps workflows - simpler and more flexible
//! than the legacy per-operation tools.
//!
//! ## Available Tools
//!
//! - `kubectl` - Execute any kubectl command
//! - `git` - Execute any git command
//! - `docker` - Execute any docker command
//! - `terraform` - Execute any terraform command
//! - `aws` - Execute any AWS CLI command
//! - `helm` - Execute any helm command
//!
//! ## Design Philosophy
//!
//! Instead of creating separate tools like `kubectl_get`, `kubectl_apply`, etc.,
//! we provide a single `kubectl` tool that takes the full command. The LLM is
//! smart enough to construct the right command based on context.
//!
//! This approach:
//! - Reduces complexity (fewer tools to maintain)
//! - Is more flexible (supports any subcommand)
//! - Leverages LLM intelligence for command construction
//!
//! ## Example
//!
//! ```yaml
//! tools:
//!   - kubectl    # Can run: kubectl get pods, kubectl apply -f x.yaml, etc.
//!   - git        # Can run: git status, git commit -m "msg", etc.
//!   - docker     # Can run: docker ps, docker build -t x ., etc.
//! ```

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, tool_config_with_timeout};

/// Unified kubectl tool - executes any kubectl command
pub struct KubectlTool {
    config: ToolConfig,
}

impl KubectlTool {
    pub fn new() -> Self {
        // Note: Schema must be Gemini-compatible (no additionalProperties)
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The kubectl command to execute (without 'kubectl' prefix). Examples: 'get pods -n production', 'apply -f deployment.yaml', 'logs my-pod --tail=100'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for command execution (optional)"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 120)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "kubectl",
                "Execute kubectl commands for Kubernetes operations. Supports all kubectl subcommands: get, apply, delete, logs, exec, describe, port-forward, etc.",
                parameters,
                120,
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("kubectl").is_ok()
    }
}

impl Default for KubectlTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(120);

        // Parse the command into arguments
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        debug!(command = %command, "Executing kubectl");

        let result = execute_command(
            "kubectl",
            &args,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("kubectl {}", command)
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Unified git tool - executes any git command
pub struct GitTool {
    config: ToolConfig,
}

impl GitTool {
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The git command to execute (without 'git' prefix). Examples: 'status', 'commit -m \"message\"', 'push origin main', 'log --oneline -10'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory (repository path)"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 120)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "git",
                "Execute git commands for version control operations. Supports all git subcommands: status, commit, push, pull, branch, checkout, merge, rebase, log, diff, etc.",
                parameters,
                120,
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("git").is_ok()
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(120);

        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        debug!(command = %command, "Executing git");

        let result = execute_command(
            "git",
            &args,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("git {}", command)
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Unified docker tool - executes any docker command
pub struct DockerTool {
    config: ToolConfig,
}

impl DockerTool {
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The docker command to execute (without 'docker' prefix). Examples: 'ps -a', 'build -t myapp .', 'run --rm nginx', 'logs container-id'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for command execution"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 300)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "docker",
                "Execute docker commands for container operations. Supports all docker subcommands: ps, build, run, exec, logs, images, pull, push, compose, etc. Auto-injects --no-stream to stats command for reliability.",
                parameters,
                300, // Longer timeout for builds
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("docker").is_ok()
    }
}

impl Default for DockerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(300);

        let mut args: Vec<String> = command.split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        // Smart injection: Add --no-stream to stats command if not present
        // This prevents stats from running continuously like `top`
        if args[0] == "stats" && !args.iter().any(|a| a.contains("--no-stream") || a == "-n") {
            args.insert(1, "--no-stream".to_string());
            debug!("Auto-injected --no-stream flag to docker stats command");
        }

        debug!(command = %command, "Executing docker");

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let result = execute_command(
            "docker",
            &args_refs,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("docker {}", args.join(" "))
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Unified terraform tool - executes any terraform command
pub struct TerraformTool {
    config: ToolConfig,
}

impl TerraformTool {
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The terraform command to execute (without 'terraform' prefix). Examples: 'init', 'plan -out=tfplan', 'apply -auto-approve', 'destroy -auto-approve'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory containing Terraform configuration"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 600)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "terraform",
                "Execute terraform commands for infrastructure as code. Supports all terraform subcommands: init, plan, apply, destroy, output, state, import, etc.",
                parameters,
                600, // Long timeout for terraform operations
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("terraform").is_ok()
    }
}

impl Default for TerraformTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(600);

        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        debug!(command = %command, "Executing terraform");

        let result = execute_command(
            "terraform",
            &args,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("terraform {}", command)
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Unified AWS CLI tool - executes any aws command
pub struct AwsTool {
    config: ToolConfig,
}

impl AwsTool {
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The AWS CLI command to execute (without 'aws' prefix). Examples: 's3 ls', 'ec2 describe-instances', 'logs filter-log-events --log-group-name /aws/lambda/func'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for command execution"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 120)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "aws",
                "Execute AWS CLI commands. Supports all AWS services: s3, ec2, ecs, lambda, logs, iam, rds, cloudformation, etc.",
                parameters,
                120,
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("aws").is_ok()
    }
}

impl Default for AwsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(120);

        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        debug!(command = %command, "Executing aws");

        let result = execute_command(
            "aws",
            &args,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("aws {}", command)
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

/// Unified helm tool - executes any helm command
pub struct HelmTool {
    config: ToolConfig,
}

impl HelmTool {
    pub fn new() -> Self {
        let parameters = serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The helm command to execute (without 'helm' prefix). Examples: 'list -A', 'install myapp ./chart', 'upgrade myapp ./chart', 'repo add bitnami https://charts.bitnami.com/bitnami'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for command execution"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds (default: 300)"
                }
            },
            "required": ["command"]
        });

        Self {
            config: tool_config_with_timeout(
                "helm",
                "Execute helm commands for Kubernetes package management. Supports all helm subcommands: install, upgrade, uninstall, list, repo, search, template, etc.",
                parameters,
                300,
            ),
        }
    }

    pub fn is_available() -> bool {
        which::which("helm").is_ok()
    }
}

impl Default for HelmTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for HelmTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let working_dir: Option<String> = input.get_arg("working_dir").ok();
        let timeout_secs: u64 = input.get_arg("timeout_secs").unwrap_or(300);

        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Ok(ToolResult::error("Empty command provided"));
        }

        debug!(command = %command, "Executing helm");

        let result = execute_command(
            "helm",
            &args,
            working_dir.as_deref(),
            timeout_secs,
        ).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "command": format!("helm {}", command)
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubectl_tool_config() {
        let tool = KubectlTool::new();
        assert_eq!(tool.config().name, "kubectl");
    }

    #[test]
    fn test_git_tool_config() {
        let tool = GitTool::new();
        assert_eq!(tool.config().name, "git");
    }

    #[test]
    fn test_docker_tool_config() {
        let tool = DockerTool::new();
        assert_eq!(tool.config().name, "docker");
    }

    #[test]
    fn test_terraform_tool_config() {
        let tool = TerraformTool::new();
        assert_eq!(tool.config().name, "terraform");
    }

    #[test]
    fn test_aws_tool_config() {
        let tool = AwsTool::new();
        assert_eq!(tool.config().name, "aws");
    }

    #[test]
    fn test_helm_tool_config() {
        let tool = HelmTool::new();
        assert_eq!(tool.config().name, "helm");
    }
}
