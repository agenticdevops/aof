//! Flux Tools
//!
//! Tools for interacting with Flux CD GitOps toolkit via CLI.
//!
//! ## Available Tools
//!
//! - `flux_kustomization_list` - List all Kustomizations
//! - `flux_kustomization_get` - Get Kustomization details
//! - `flux_helmrelease_list` - List all HelmReleases
//! - `flux_helmrelease_get` - Get HelmRelease details
//! - `flux_reconcile` - Trigger immediate reconciliation
//! - `flux_suspend` - Suspend reconciliation
//! - `flux_resume` - Resume reconciliation
//! - `flux_logs` - Get Flux controller logs
//!
//! ## Prerequisites
//!
//! - Requires `cicd` feature flag
//! - Flux CLI must be installed and in PATH
//! - kubectl configured with cluster access
//! - Flux controllers running in cluster
//!
//! ## Authentication
//!
//! Uses kubectl/kubeconfig for cluster authentication.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tokio::process::Command;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Flux tools
pub struct FluxTools;

impl FluxTools {
    /// Get all Flux tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(FluxKustomizationListTool::new()),
            Box::new(FluxKustomizationGetTool::new()),
            Box::new(FluxHelmReleaseListTool::new()),
            Box::new(FluxHelmReleaseGetTool::new()),
            Box::new(FluxReconcileTool::new()),
            Box::new(FluxSuspendTool::new()),
            Box::new(FluxResumeTool::new()),
            Box::new(FluxLogsTool::new()),
        ]
    }
}

/// Execute a Flux CLI command and return result
async fn execute_flux_command(
    args: &[&str],
    kubeconfig: Option<&str>,
    context: Option<&str>,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    let mut cmd = Command::new("flux");
    cmd.args(args);

    // Add JSON output flag if not already present
    if !args.contains(&"-o") && !args.contains(&"--output") {
        cmd.arg("-o").arg("json");
    }

    // Set kubeconfig if provided
    if let Some(kc) = kubeconfig {
        cmd.env("KUBECONFIG", kc);
    }

    // Add context if provided
    if let Some(ctx) = context {
        cmd.arg("--context").arg(ctx);
    }

    // Capture output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn flux command: {}. Is Flux CLI installed?", e))?;

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| format!("Command timed out after {}s", timeout_secs))?
    .map_err(|e| format!("Command failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        // Check for common errors
        if stderr.contains("command not found") || stderr.contains("not recognized") {
            return Err(
                "flux command not found. Install Flux CLI: curl -s https://fluxcd.io/install.sh | sudo bash"
                    .to_string(),
            );
        }
        if stderr.contains("connection refused") || stderr.contains("unable to connect") {
            return Err("Unable to connect to cluster. Check kubeconfig and cluster access.".to_string());
        }
        if stderr.contains("not installed") {
            return Err("Flux not installed in cluster. Run: flux install".to_string());
        }
        return Err(format!("Flux command failed: {}", stderr));
    }

    // Try to parse as JSON
    if stdout.trim().is_empty() {
        return Ok(serde_json::json!({"message": "Command completed successfully"}));
    }

    serde_json::from_str(&stdout).map_err(|e| {
        // If JSON parsing fails, return raw output
        format!(
            "Failed to parse JSON output: {}. Raw output: {}",
            e,
            stdout.trim()
        )
    })
}

// ============================================================================
// Flux Kustomization List Tool
// ============================================================================

/// List all Flux Kustomizations
pub struct FluxKustomizationListTool {
    config: ToolConfig,
}

impl FluxKustomizationListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "namespace": {
                    "type": "string",
                    "description": "Filter by namespace"
                },
                "all_namespaces": {
                    "type": "boolean",
                    "description": "List from all namespaces",
                    "default": true
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_kustomization_list",
                "List all Flux Kustomizations in the cluster. Shows status, path, and revision.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxKustomizationListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxKustomizationListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let all_namespaces: bool = input.get_arg("all_namespaces").unwrap_or(true);
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!("Listing Flux kustomizations");

        let mut args = vec!["get", "kustomizations"];

        if let Some(ref ns) = namespace {
            args.push("-n");
            args.push(ns);
        } else if all_namespaces {
            args.push("-A");
        }

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(data) => {
                // Count items if it's an array
                let count = data.as_array().map(|a| a.len()).unwrap_or(0);
                Ok(ToolResult::success(serde_json::json!({
                    "kustomizations": data,
                    "count": count
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
// Flux Kustomization Get Tool
// ============================================================================

/// Get Flux Kustomization details
pub struct FluxKustomizationGetTool {
    config: ToolConfig,
}

impl FluxKustomizationGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "name": {
                    "type": "string",
                    "description": "Kustomization name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace",
                    "default": "flux-system"
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec!["name"],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_kustomization_get",
                "Get detailed status of a specific Flux Kustomization including conditions and events.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxKustomizationGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxKustomizationGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let name: String = input.get_arg("name")?;
        let namespace: String = input.get_arg("namespace").unwrap_or_else(|_| "flux-system".to_string());
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(name = %name, namespace = %namespace, "Getting Flux kustomization");

        let args = vec!["get", "kustomization", &name, "-n", &namespace];

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(data) => Ok(ToolResult::success(data)),
            Err(e) => {
                if e.contains("not found") {
                    Ok(ToolResult::error(format!(
                        "Kustomization '{}' not found in namespace '{}'",
                        name, namespace
                    )))
                } else {
                    Ok(ToolResult::error(e))
                }
            }
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Flux HelmRelease List Tool
// ============================================================================

/// List all Flux HelmReleases
pub struct FluxHelmReleaseListTool {
    config: ToolConfig,
}

impl FluxHelmReleaseListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "namespace": {
                    "type": "string",
                    "description": "Filter by namespace"
                },
                "all_namespaces": {
                    "type": "boolean",
                    "description": "List from all namespaces",
                    "default": true
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_helmrelease_list",
                "List all Flux HelmReleases in the cluster. Shows chart version and status.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxHelmReleaseListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxHelmReleaseListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let all_namespaces: bool = input.get_arg("all_namespaces").unwrap_or(true);
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!("Listing Flux HelmReleases");

        let mut args = vec!["get", "helmreleases"];

        if let Some(ref ns) = namespace {
            args.push("-n");
            args.push(ns);
        } else if all_namespaces {
            args.push("-A");
        }

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(data) => {
                let count = data.as_array().map(|a| a.len()).unwrap_or(0);
                Ok(ToolResult::success(serde_json::json!({
                    "helmreleases": data,
                    "count": count
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
// Flux HelmRelease Get Tool
// ============================================================================

/// Get Flux HelmRelease details
pub struct FluxHelmReleaseGetTool {
    config: ToolConfig,
}

impl FluxHelmReleaseGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "name": {
                    "type": "string",
                    "description": "HelmRelease name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace"
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec!["name", "namespace"],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_helmrelease_get",
                "Get detailed status of a specific Flux HelmRelease including chart info and conditions.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxHelmReleaseGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxHelmReleaseGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let name: String = input.get_arg("name")?;
        let namespace: String = input.get_arg("namespace")?;
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(name = %name, namespace = %namespace, "Getting Flux HelmRelease");

        let args = vec!["get", "helmrelease", &name, "-n", &namespace];

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(data) => Ok(ToolResult::success(data)),
            Err(e) => {
                if e.contains("not found") {
                    Ok(ToolResult::error(format!(
                        "HelmRelease '{}' not found in namespace '{}'",
                        name, namespace
                    )))
                } else {
                    Ok(ToolResult::error(e))
                }
            }
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Flux Reconcile Tool
// ============================================================================

/// Trigger Flux resource reconciliation
pub struct FluxReconcileTool {
    config: ToolConfig,
}

impl FluxReconcileTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "kind": {
                    "type": "string",
                    "description": "Resource kind",
                    "enum": ["kustomization", "helmrelease", "source", "gitrepository", "helmrepository", "helmchart"]
                },
                "name": {
                    "type": "string",
                    "description": "Resource name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace",
                    "default": "flux-system"
                },
                "with_source": {
                    "type": "boolean",
                    "description": "Reconcile source first",
                    "default": false
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec!["kind", "name"],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_reconcile",
                "Trigger immediate reconciliation of a Flux resource. Syncs changes from Git.",
                parameters,
                120, // Longer timeout for reconciliation
            ),
        }
    }
}

impl Default for FluxReconcileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxReconcileTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let kind: String = input.get_arg("kind")?;
        let name: String = input.get_arg("name")?;
        let namespace: String = input.get_arg("namespace").unwrap_or_else(|_| "flux-system".to_string());
        let with_source: bool = input.get_arg("with_source").unwrap_or(false);
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(kind = %kind, name = %name, namespace = %namespace, "Reconciling Flux resource");

        let mut args = vec!["reconcile", &kind, &name, "-n", &namespace];

        if with_source {
            args.push("--with-source");
        }

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            120,
        )
        .await
        {
            Ok(data) => Ok(ToolResult::success(serde_json::json!({
                "reconciled": true,
                "kind": kind,
                "name": name,
                "namespace": namespace,
                "message": "Reconciliation completed",
                "details": data
            }))),
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Flux Suspend Tool
// ============================================================================

/// Suspend Flux resource reconciliation
pub struct FluxSuspendTool {
    config: ToolConfig,
}

impl FluxSuspendTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "kind": {
                    "type": "string",
                    "description": "Resource kind",
                    "enum": ["kustomization", "helmrelease"]
                },
                "name": {
                    "type": "string",
                    "description": "Resource name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace",
                    "default": "flux-system"
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec!["kind", "name"],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_suspend",
                "Suspend reconciliation of a Flux resource. Useful during maintenance.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxSuspendTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxSuspendTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let kind: String = input.get_arg("kind")?;
        let name: String = input.get_arg("name")?;
        let namespace: String = input.get_arg("namespace").unwrap_or_else(|_| "flux-system".to_string());
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(kind = %kind, name = %name, namespace = %namespace, "Suspending Flux resource");

        let args = vec!["suspend", &kind, &name, "-n", &namespace];

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(_) => Ok(ToolResult::success(serde_json::json!({
                "suspended": true,
                "kind": kind,
                "name": name,
                "namespace": namespace,
                "message": "Reconciliation suspended"
            }))),
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Flux Resume Tool
// ============================================================================

/// Resume Flux resource reconciliation
pub struct FluxResumeTool {
    config: ToolConfig,
}

impl FluxResumeTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "kind": {
                    "type": "string",
                    "description": "Resource kind",
                    "enum": ["kustomization", "helmrelease"]
                },
                "name": {
                    "type": "string",
                    "description": "Resource name"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace",
                    "default": "flux-system"
                },
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec!["kind", "name"],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_resume",
                "Resume reconciliation of a suspended Flux resource.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxResumeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxResumeTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let kind: String = input.get_arg("kind")?;
        let name: String = input.get_arg("name")?;
        let namespace: String = input.get_arg("namespace").unwrap_or_else(|_| "flux-system".to_string());
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(kind = %kind, name = %name, namespace = %namespace, "Resuming Flux resource");

        let args = vec!["resume", &kind, &name, "-n", &namespace];

        match execute_flux_command(
            &args,
            kubeconfig.as_deref(),
            context.as_deref(),
            30,
        )
        .await
        {
            Ok(_) => Ok(ToolResult::success(serde_json::json!({
                "resumed": true,
                "kind": kind,
                "name": name,
                "namespace": namespace,
                "message": "Reconciliation resumed"
            }))),
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Flux Logs Tool
// ============================================================================

/// Get Flux controller logs
pub struct FluxLogsTool {
    config: ToolConfig,
}

impl FluxLogsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "kind": {
                    "type": "string",
                    "description": "Controller kind",
                    "enum": ["source-controller", "kustomize-controller", "helm-controller", "notification-controller", "image-reflector-controller", "image-automation-controller"],
                    "default": "kustomize-controller"
                },
                "namespace": {
                    "type": "string",
                    "description": "Flux namespace",
                    "default": "flux-system"
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
                "kubeconfig": {
                    "type": "string",
                    "description": "Path to kubeconfig file"
                },
                "context": {
                    "type": "string",
                    "description": "Kubernetes context to use"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "flux_logs",
                "Get logs from Flux controllers for troubleshooting.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for FluxLogsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FluxLogsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let kind: String = input
            .get_arg("kind")
            .unwrap_or_else(|_| "kustomize-controller".to_string());
        let namespace: String = input.get_arg("namespace").unwrap_or_else(|_| "flux-system".to_string());
        let tail: i32 = input.get_arg("tail").unwrap_or(100);
        let since: Option<String> = input.get_arg("since").ok();
        let kubeconfig: Option<String> = input.get_arg("kubeconfig").ok();
        let context: Option<String> = input.get_arg("context").ok();

        debug!(kind = %kind, namespace = %namespace, "Getting Flux logs");

        // Use kubectl to get logs since flux logs command output isn't JSON
        let mut cmd = Command::new("kubectl");
        cmd.args(&[
            "logs",
            "-l",
            &format!("app={}", kind),
            "-n",
            &namespace,
            "--tail",
            &tail.to_string(),
        ]);

        if let Some(ref s) = since {
            cmd.args(&["--since", s]);
        }

        if let Some(ref kc) = kubeconfig {
            cmd.env("KUBECONFIG", kc);
        }

        if let Some(ref ctx) = context {
            cmd.args(&["--context", ctx]);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to spawn kubectl: {}", e)))?;

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| aof_core::AofError::tool("Command timed out".to_string()))?
        .map_err(|e| aof_core::AofError::tool(format!("Command failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(ToolResult::error(format!(
                "Failed to get logs: {}",
                stderr
            )));
        }

        let log_lines: Vec<&str> = stdout.lines().collect();

        Ok(ToolResult::success(serde_json::json!({
            "controller": kind,
            "namespace": namespace,
            "logs": stdout.to_string(),
            "log_lines": log_lines.len()
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_kustomization_list_tool_creation() {
        let tool = FluxKustomizationListTool::new();
        assert_eq!(tool.config().name, "flux_kustomization_list");
    }

    #[test]
    fn test_flux_kustomization_get_tool_creation() {
        let tool = FluxKustomizationGetTool::new();
        assert_eq!(tool.config().name, "flux_kustomization_get");
    }

    #[test]
    fn test_flux_helmrelease_list_tool_creation() {
        let tool = FluxHelmReleaseListTool::new();
        assert_eq!(tool.config().name, "flux_helmrelease_list");
    }

    #[test]
    fn test_flux_helmrelease_get_tool_creation() {
        let tool = FluxHelmReleaseGetTool::new();
        assert_eq!(tool.config().name, "flux_helmrelease_get");
    }

    #[test]
    fn test_flux_reconcile_tool_creation() {
        let tool = FluxReconcileTool::new();
        assert_eq!(tool.config().name, "flux_reconcile");
    }

    #[test]
    fn test_flux_suspend_tool_creation() {
        let tool = FluxSuspendTool::new();
        assert_eq!(tool.config().name, "flux_suspend");
    }

    #[test]
    fn test_flux_resume_tool_creation() {
        let tool = FluxResumeTool::new();
        assert_eq!(tool.config().name, "flux_resume");
    }

    #[test]
    fn test_flux_logs_tool_creation() {
        let tool = FluxLogsTool::new();
        assert_eq!(tool.config().name, "flux_logs");
    }

    #[test]
    fn test_flux_tools_all() {
        let tools = FluxTools::all();
        assert_eq!(tools.len(), 8);
    }
}
