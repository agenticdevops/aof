//! Docker Tools
//!
//! Tools for Docker container operations.
//!
//! ## Available Tools
//!
//! - `docker_ps` - List running containers
//! - `docker_build` - Build images
//! - `docker_run` - Run containers
//! - `docker_logs` - Get container logs
//! - `docker_exec` - Execute commands in containers
//! - `docker_images` - List images
//!
//! ## Prerequisites
//!
//! - Docker must be installed and running
//! - User must have Docker socket access
//!
//! ## MCP Alternative
//!
//! For MCP-based Docker operations, use a Docker MCP server.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all Docker tools
pub struct DockerTools;

impl DockerTools {
    /// Get all Docker tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(DockerPsTool::new()),
            Box::new(DockerBuildTool::new()),
            Box::new(DockerRunTool::new()),
            Box::new(DockerLogsTool::new()),
            Box::new(DockerExecTool::new()),
            Box::new(DockerImagesTool::new()),
        ]
    }

    /// Check if Docker is available
    pub fn is_available() -> bool {
        which::which("docker").is_ok()
    }
}

// ============================================================================
// Docker PS Tool
// ============================================================================

/// List Docker containers
pub struct DockerPsTool {
    config: ToolConfig,
}

impl DockerPsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "all": {
                    "type": "boolean",
                    "description": "Show all containers (default shows just running)",
                    "default": false
                },
                "filter": {
                    "type": "string",
                    "description": "Filter output (e.g., 'status=running', 'name=web')"
                },
                "format": {
                    "type": "string",
                    "description": "Output format (json for structured data)",
                    "default": "json"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_ps",
                "List Docker containers. Returns container IDs, names, and status.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for DockerPsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerPsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let all: bool = input.get_arg("all").unwrap_or(false);
        let filter: Option<String> = input.get_arg("filter").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["ps".to_string()];

        if all {
            args.push("-a".to_string());
        }

        if let Some(ref f) = filter {
            args.push(format!("--filter={}", f));
        }

        if format == "json" {
            args.push("--format={{json .}}".to_string());
        }

        debug!(args = ?args, "Executing docker ps");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    // Parse JSON lines output
                    let containers: Vec<serde_json::Value> = output
                        .stdout
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();

                    Ok(ToolResult::success(serde_json::json!({
                        "containers": containers,
                        "count": containers.len()
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "docker ps failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Docker Build Tool
// ============================================================================

/// Build Docker images
pub struct DockerBuildTool {
    config: ToolConfig,
}

impl DockerBuildTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Build context path (default: current directory)",
                    "default": "."
                },
                "tag": {
                    "type": "string",
                    "description": "Image tag (e.g., 'myapp:latest')"
                },
                "dockerfile": {
                    "type": "string",
                    "description": "Dockerfile path (default: Dockerfile)"
                },
                "build_args": {
                    "type": "object",
                    "description": "Build arguments",
                    "additionalProperties": { "type": "string" }
                },
                "no_cache": {
                    "type": "boolean",
                    "description": "Don't use cache",
                    "default": false
                },
                "target": {
                    "type": "string",
                    "description": "Target build stage"
                },
                "platform": {
                    "type": "string",
                    "description": "Target platform (e.g., 'linux/amd64')"
                }
            }),
            vec!["tag"],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_build",
                "Build a Docker image from a Dockerfile.",
                parameters,
                600, // 10 minute timeout for builds
            ),
        }
    }
}

impl Default for DockerBuildTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerBuildTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let tag: String = input.get_arg("tag")?;
        let dockerfile: Option<String> = input.get_arg("dockerfile").ok();
        let build_args: std::collections::HashMap<String, String> = input
            .get_arg("build_args")
            .unwrap_or_default();
        let no_cache: bool = input.get_arg("no_cache").unwrap_or(false);
        let target: Option<String> = input.get_arg("target").ok();
        let platform: Option<String> = input.get_arg("platform").ok();

        let mut args = vec!["build".to_string(), "-t".to_string(), tag.clone()];

        if let Some(ref df) = dockerfile {
            args.push("-f".to_string());
            args.push(df.clone());
        }

        for (key, value) in &build_args {
            args.push("--build-arg".to_string());
            args.push(format!("{}={}", key, value));
        }

        if no_cache {
            args.push("--no-cache".to_string());
        }

        if let Some(ref t) = target {
            args.push("--target".to_string());
            args.push(t.clone());
        }

        if let Some(ref p) = platform {
            args.push("--platform".to_string());
            args.push(p.clone());
        }

        args.push(path.clone());

        debug!(args = ?args, "Executing docker build");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 600).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "image": tag,
                        "built": true,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "docker build failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Docker Run Tool
// ============================================================================

/// Run Docker containers
pub struct DockerRunTool {
    config: ToolConfig,
}

impl DockerRunTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "image": {
                    "type": "string",
                    "description": "Image to run"
                },
                "name": {
                    "type": "string",
                    "description": "Container name"
                },
                "command": {
                    "type": "string",
                    "description": "Command to run in container"
                },
                "detach": {
                    "type": "boolean",
                    "description": "Run in background",
                    "default": false
                },
                "rm": {
                    "type": "boolean",
                    "description": "Remove container after exit",
                    "default": true
                },
                "ports": {
                    "type": "array",
                    "description": "Port mappings (e.g., ['8080:80', '443:443'])",
                    "items": { "type": "string" }
                },
                "volumes": {
                    "type": "array",
                    "description": "Volume mounts (e.g., ['/host:/container'])",
                    "items": { "type": "string" }
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables",
                    "additionalProperties": { "type": "string" }
                },
                "network": {
                    "type": "string",
                    "description": "Network to connect to"
                }
            }),
            vec!["image"],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_run",
                "Run a Docker container.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for DockerRunTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerRunTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let image: String = input.get_arg("image")?;
        let name: Option<String> = input.get_arg("name").ok();
        let command: Option<String> = input.get_arg("command").ok();
        let detach: bool = input.get_arg("detach").unwrap_or(false);
        let rm: bool = input.get_arg("rm").unwrap_or(true);
        let ports: Vec<String> = input.get_arg("ports").unwrap_or_default();
        let volumes: Vec<String> = input.get_arg("volumes").unwrap_or_default();
        let env: std::collections::HashMap<String, String> = input.get_arg("env").unwrap_or_default();
        let network: Option<String> = input.get_arg("network").ok();

        let mut args = vec!["run".to_string()];

        if detach {
            args.push("-d".to_string());
        }

        if rm {
            args.push("--rm".to_string());
        }

        if let Some(ref n) = name {
            args.push("--name".to_string());
            args.push(n.clone());
        }

        for port in &ports {
            args.push("-p".to_string());
            args.push(port.clone());
        }

        for vol in &volumes {
            args.push("-v".to_string());
            args.push(vol.clone());
        }

        for (key, value) in &env {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        if let Some(ref n) = network {
            args.push("--network".to_string());
            args.push(n.clone());
        }

        args.push(image.clone());

        if let Some(ref cmd) = command {
            args.extend(cmd.split_whitespace().map(String::from));
        }

        debug!(args = ?args, "Executing docker run");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 300).await;

        match result {
            Ok(output) => {
                if output.success {
                    let container_id = output.stdout.trim().to_string();
                    Ok(ToolResult::success(serde_json::json!({
                        "container_id": container_id,
                        "image": image,
                        "detached": detach,
                        "output": if detach { container_id } else { output.stdout }
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "docker run failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Docker Logs Tool
// ============================================================================

/// Get container logs
pub struct DockerLogsTool {
    config: ToolConfig,
}

impl DockerLogsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "container": {
                    "type": "string",
                    "description": "Container name or ID"
                },
                "tail": {
                    "type": "integer",
                    "description": "Number of lines from end",
                    "default": 100
                },
                "since": {
                    "type": "string",
                    "description": "Show logs since (e.g., '5m', '1h')"
                },
                "timestamps": {
                    "type": "boolean",
                    "description": "Show timestamps",
                    "default": false
                }
            }),
            vec!["container"],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_logs",
                "Get logs from a Docker container.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for DockerLogsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerLogsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let container: String = input.get_arg("container")?;
        let tail: i32 = input.get_arg("tail").unwrap_or(100);
        let since: Option<String> = input.get_arg("since").ok();
        let timestamps: bool = input.get_arg("timestamps").unwrap_or(false);

        let mut args = vec!["logs".to_string()];

        args.push(format!("--tail={}", tail));

        if let Some(ref s) = since {
            args.push(format!("--since={}", s));
        }

        if timestamps {
            args.push("--timestamps".to_string());
        }

        args.push(container.clone());

        debug!(args = ?args, "Executing docker logs");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                // Docker logs go to stderr for some reason
                let logs = if output.stdout.is_empty() {
                    output.stderr.clone()
                } else {
                    output.stdout.clone()
                };

                Ok(ToolResult::success(serde_json::json!({
                    "logs": logs,
                    "container": container,
                    "lines": logs.lines().count()
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Docker Exec Tool
// ============================================================================

/// Execute commands in containers
pub struct DockerExecTool {
    config: ToolConfig,
}

impl DockerExecTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "container": {
                    "type": "string",
                    "description": "Container name or ID"
                },
                "command": {
                    "type": "string",
                    "description": "Command to execute"
                },
                "user": {
                    "type": "string",
                    "description": "User to run as"
                },
                "workdir": {
                    "type": "string",
                    "description": "Working directory"
                }
            }),
            vec!["container", "command"],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_exec",
                "Execute a command in a running container.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for DockerExecTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerExecTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let container: String = input.get_arg("container")?;
        let command: String = input.get_arg("command")?;
        let user: Option<String> = input.get_arg("user").ok();
        let workdir: Option<String> = input.get_arg("workdir").ok();

        let mut args = vec!["exec".to_string()];

        if let Some(ref u) = user {
            args.push("-u".to_string());
            args.push(u.clone());
        }

        if let Some(ref w) = workdir {
            args.push("-w".to_string());
            args.push(w.clone());
        }

        args.push(container.clone());
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push(command.clone());

        debug!(args = ?args, "Executing docker exec");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "container": container,
                    "command": command
                })))
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Docker Images Tool
// ============================================================================

/// List Docker images
pub struct DockerImagesTool {
    config: ToolConfig,
}

impl DockerImagesTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "all": {
                    "type": "boolean",
                    "description": "Show all images (including intermediates)",
                    "default": false
                },
                "filter": {
                    "type": "string",
                    "description": "Filter output (e.g., 'dangling=true')"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "docker_images",
                "List Docker images.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for DockerImagesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerImagesTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let all: bool = input.get_arg("all").unwrap_or(false);
        let filter: Option<String> = input.get_arg("filter").ok();

        let mut args = vec!["images".to_string(), "--format={{json .}}".to_string()];

        if all {
            args.push("-a".to_string());
        }

        if let Some(ref f) = filter {
            args.push(format!("--filter={}", f));
        }

        debug!(args = ?args, "Executing docker images");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("docker", &args_str, None, 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    let images: Vec<serde_json::Value> = output
                        .stdout
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();

                    Ok(ToolResult::success(serde_json::json!({
                        "images": images,
                        "count": images.len()
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "docker images failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
