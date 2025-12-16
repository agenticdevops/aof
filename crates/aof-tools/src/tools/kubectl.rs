//! Kubectl Tools
//!
//! Tools for Kubernetes operations via kubectl CLI.
//!
//! ## Available Tools
//!
//! - `kubectl_get` - Get resources (pods, deployments, services, etc.)
//! - `kubectl_apply` - Apply manifests
//! - `kubectl_delete` - Delete resources
//! - `kubectl_logs` - Get pod logs
//! - `kubectl_exec` - Execute commands in containers
//! - `kubectl_describe` - Describe resources
//!
//! ## Prerequisites
//!
//! - kubectl must be installed and in PATH
//! - Valid kubeconfig with cluster access
//!
//! ## MCP Alternative
//!
//! Use kubectl-ai MCP server for enhanced K8s operations:
//! ```yaml
//! mcp_servers:
//!   - name: kubectl-ai
//!     transport: stdio
//!     command: kubectl-ai
//!     args: ["--mcp-server"]
//! ```

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all kubectl tools
pub struct KubectlTools;

impl KubectlTools {
    /// Get all kubectl tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(KubectlGetTool::new()),
            Box::new(KubectlApplyTool::new()),
            Box::new(KubectlDeleteTool::new()),
            Box::new(KubectlLogsTool::new()),
            Box::new(KubectlExecTool::new()),
            Box::new(KubectlDescribeTool::new()),
        ]
    }

    /// Check if kubectl is available
    pub fn is_available() -> bool {
        which::which("kubectl").is_ok()
    }
}

// ============================================================================
// Kubectl Get Tool
// ============================================================================

/// Get Kubernetes resources
pub struct KubectlGetTool {
    config: ToolConfig,
}

impl KubectlGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "resource": {
                    "type": "string",
                    "description": "Resource type (e.g., pods, deployments, services, nodes)"
                },
                "name": {
                    "type": "string",
                    "description": "Resource name (optional, omit to list all)"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace (default: current context namespace)"
                },
                "all_namespaces": {
                    "type": "boolean",
                    "description": "Get resources from all namespaces",
                    "default": false
                },
                "output": {
                    "type": "string",
                    "description": "Output format: json, yaml, wide, name",
                    "enum": ["json", "yaml", "wide", "name"],
                    "default": "json"
                },
                "selector": {
                    "type": "string",
                    "description": "Label selector (e.g., 'app=nginx,env=prod')"
                },
                "field_selector": {
                    "type": "string",
                    "description": "Field selector (e.g., 'status.phase=Running')"
                }
            }),
            vec!["resource"],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_get",
                "Get Kubernetes resources. Returns resource details in specified format.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for KubectlGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let resource: String = input.get_arg("resource")?;
        let name: Option<String> = input.get_arg("name").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let all_namespaces: bool = input.get_arg("all_namespaces").unwrap_or(false);
        let output: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());
        let selector: Option<String> = input.get_arg("selector").ok();
        let field_selector: Option<String> = input.get_arg("field_selector").ok();

        let mut args = vec!["get", &resource];

        if let Some(ref n) = name {
            args.push(n);
        }

        let ns_flag;
        if all_namespaces {
            args.push("-A");
        } else if let Some(ref ns) = namespace {
            ns_flag = format!("-n={}", ns);
            args.push(&ns_flag);
        }

        let output_flag = format!("-o={}", output);
        args.push(&output_flag);

        let selector_flag;
        if let Some(ref sel) = selector {
            selector_flag = format!("-l={}", sel);
            args.push(&selector_flag);
        }

        let field_flag;
        if let Some(ref field) = field_selector {
            field_flag = format!("--field-selector={}", field);
            args.push(&field_flag);
        }

        debug!(args = ?args, "Executing kubectl get");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
        let result = execute_command("kubectl", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    // Try to parse JSON output
                    let data = if output_flag == "-o=json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "resource": resource,
                        "namespace": namespace
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "kubectl get failed: {}",
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
// Kubectl Apply Tool
// ============================================================================

/// Apply Kubernetes manifests
pub struct KubectlApplyTool {
    config: ToolConfig,
}

impl KubectlApplyTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "manifest": {
                    "type": "string",
                    "description": "YAML manifest content to apply"
                },
                "file": {
                    "type": "string",
                    "description": "Path to manifest file (alternative to inline manifest)"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                },
                "dry_run": {
                    "type": "string",
                    "description": "Dry run mode: none, client, server",
                    "enum": ["none", "client", "server"],
                    "default": "none"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force apply (delete and recreate)",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_apply",
                "Apply a Kubernetes manifest. Creates or updates resources.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for KubectlApplyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlApplyTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let manifest: Option<String> = input.get_arg("manifest").ok();
        let file: Option<String> = input.get_arg("file").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let dry_run: String = input.get_arg("dry_run").unwrap_or_else(|_| "none".to_string());
        let force: bool = input.get_arg("force").unwrap_or(false);

        if manifest.is_none() && file.is_none() {
            return Ok(ToolResult::error("Either 'manifest' or 'file' is required"));
        }

        let mut args = vec!["apply".to_string()];

        if let Some(ref f) = file {
            args.push(format!("-f={}", f));
        } else if let Some(ref m) = manifest {
            // Write manifest to temp file
            let temp_path = "/tmp/kubectl-apply-manifest.yaml";
            if let Err(e) = tokio::fs::write(temp_path, m).await {
                return Ok(ToolResult::error(format!("Failed to write temp manifest: {}", e)));
            }
            args.push(format!("-f={}", temp_path));
        }

        if let Some(ref ns) = namespace {
            args.push(format!("-n={}", ns));
        }

        if dry_run != "none" {
            args.push(format!("--dry-run={}", dry_run));
        }

        if force {
            args.push("--force".to_string());
        }

        args.push("-o=json".to_string());

        debug!(args = ?args, "Executing kubectl apply");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("kubectl", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                        serde_json::json!({ "output": output.stdout })
                    });
                    Ok(ToolResult::success(serde_json::json!({
                        "applied": data,
                        "dry_run": dry_run != "none"
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "kubectl apply failed: {}",
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
// Kubectl Delete Tool
// ============================================================================

/// Delete Kubernetes resources
pub struct KubectlDeleteTool {
    config: ToolConfig,
}

impl KubectlDeleteTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "resource": {
                    "type": "string",
                    "description": "Resource type (e.g., pod, deployment, service)"
                },
                "name": {
                    "type": "string",
                    "description": "Resource name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                },
                "selector": {
                    "type": "string",
                    "description": "Label selector for bulk delete"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force delete (immediate)",
                    "default": false
                },
                "grace_period": {
                    "type": "integer",
                    "description": "Grace period in seconds",
                    "default": 30
                }
            }),
            vec!["resource"],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_delete",
                "Delete Kubernetes resources.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for KubectlDeleteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlDeleteTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let resource: String = input.get_arg("resource")?;
        let name: Option<String> = input.get_arg("name").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let selector: Option<String> = input.get_arg("selector").ok();
        let force: bool = input.get_arg("force").unwrap_or(false);
        let grace_period: i32 = input.get_arg("grace_period").unwrap_or(30);

        if name.is_none() && selector.is_none() {
            return Ok(ToolResult::error("Either 'name' or 'selector' is required"));
        }

        let mut args = vec!["delete".to_string(), resource.clone()];

        if let Some(ref n) = name {
            args.push(n.clone());
        }

        if let Some(ref ns) = namespace {
            args.push(format!("-n={}", ns));
        }

        if let Some(ref sel) = selector {
            args.push(format!("-l={}", sel));
        }

        if force {
            args.push("--force".to_string());
            args.push("--grace-period=0".to_string());
        } else {
            args.push(format!("--grace-period={}", grace_period));
        }

        debug!(args = ?args, "Executing kubectl delete");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("kubectl", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "deleted": true,
                        "resource": resource,
                        "name": name,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "kubectl delete failed: {}",
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
// Kubectl Logs Tool
// ============================================================================

/// Get pod logs
pub struct KubectlLogsTool {
    config: ToolConfig,
}

impl KubectlLogsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "pod": {
                    "type": "string",
                    "description": "Pod name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                },
                "container": {
                    "type": "string",
                    "description": "Container name (for multi-container pods)"
                },
                "tail": {
                    "type": "integer",
                    "description": "Number of lines from the end",
                    "default": 100
                },
                "since": {
                    "type": "string",
                    "description": "Show logs since duration (e.g., '5m', '1h')"
                },
                "previous": {
                    "type": "boolean",
                    "description": "Get logs from previous container instance",
                    "default": false
                },
                "timestamps": {
                    "type": "boolean",
                    "description": "Include timestamps",
                    "default": false
                }
            }),
            vec!["pod"],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_logs",
                "Get logs from a Kubernetes pod.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for KubectlLogsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlLogsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let pod: String = input.get_arg("pod")?;
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let container: Option<String> = input.get_arg("container").ok();
        let tail: i32 = input.get_arg("tail").unwrap_or(100);
        let since: Option<String> = input.get_arg("since").ok();
        let previous: bool = input.get_arg("previous").unwrap_or(false);
        let timestamps: bool = input.get_arg("timestamps").unwrap_or(false);

        let mut args = vec!["logs".to_string(), pod.clone()];

        if let Some(ref ns) = namespace {
            args.push(format!("-n={}", ns));
        }

        if let Some(ref c) = container {
            args.push(format!("-c={}", c));
        }

        args.push(format!("--tail={}", tail));

        if let Some(ref s) = since {
            args.push(format!("--since={}", s));
        }

        if previous {
            args.push("--previous".to_string());
        }

        if timestamps {
            args.push("--timestamps".to_string());
        }

        debug!(args = ?args, "Executing kubectl logs");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("kubectl", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "logs": output.stdout,
                        "pod": pod,
                        "container": container,
                        "lines": output.stdout.lines().count()
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "kubectl logs failed: {}",
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
// Kubectl Exec Tool
// ============================================================================

/// Execute commands in containers
pub struct KubectlExecTool {
    config: ToolConfig,
}

impl KubectlExecTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "pod": {
                    "type": "string",
                    "description": "Pod name"
                },
                "command": {
                    "type": "string",
                    "description": "Command to execute in the container"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                },
                "container": {
                    "type": "string",
                    "description": "Container name (for multi-container pods)"
                }
            }),
            vec!["pod", "command"],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_exec",
                "Execute a command inside a Kubernetes pod container.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for KubectlExecTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlExecTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let pod: String = input.get_arg("pod")?;
        let command: String = input.get_arg("command")?;
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let container: Option<String> = input.get_arg("container").ok();

        let mut args = vec!["exec".to_string(), pod.clone()];

        if let Some(ref ns) = namespace {
            args.push(format!("-n={}", ns));
        }

        if let Some(ref c) = container {
            args.push(format!("-c={}", c));
        }

        args.push("--".to_string());
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push(command.clone());

        debug!(args = ?args, "Executing kubectl exec");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("kubectl", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                Ok(ToolResult::success(serde_json::json!({
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "success": output.success,
                    "pod": pod,
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
// Kubectl Describe Tool
// ============================================================================

/// Describe Kubernetes resources
pub struct KubectlDescribeTool {
    config: ToolConfig,
}

impl KubectlDescribeTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "resource": {
                    "type": "string",
                    "description": "Resource type (e.g., pod, deployment, service)"
                },
                "name": {
                    "type": "string",
                    "description": "Resource name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                }
            }),
            vec!["resource", "name"],
        );

        Self {
            config: tool_config_with_timeout(
                "kubectl_describe",
                "Get detailed description of a Kubernetes resource.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for KubectlDescribeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlDescribeTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let resource: String = input.get_arg("resource")?;
        let name: String = input.get_arg("name")?;
        let namespace: Option<String> = input.get_arg("namespace").ok();

        let mut args = vec!["describe".to_string(), resource.clone(), name.clone()];

        if let Some(ref ns) = namespace {
            args.push(format!("-n={}", ns));
        }

        debug!(args = ?args, "Executing kubectl describe");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("kubectl", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "description": output.stdout,
                        "resource": resource,
                        "name": name
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "kubectl describe failed: {}",
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
