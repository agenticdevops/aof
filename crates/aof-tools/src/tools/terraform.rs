//! Terraform Tools
//!
//! Tools for Terraform Infrastructure as Code operations.
//!
//! ## Available Tools
//!
//! - `terraform_init` - Initialize Terraform working directory
//! - `terraform_plan` - Create execution plan
//! - `terraform_apply` - Apply changes
//! - `terraform_destroy` - Destroy infrastructure
//! - `terraform_output` - Get outputs
//!
//! ## Prerequisites
//!
//! - Terraform must be installed and in PATH
//! - Valid provider credentials configured
//!
//! ## MCP Alternative
//!
//! For MCP-based Terraform operations, use a Terraform MCP server.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all Terraform tools
pub struct TerraformTools;

impl TerraformTools {
    /// Get all Terraform tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(TerraformInitTool::new()),
            Box::new(TerraformPlanTool::new()),
            Box::new(TerraformApplyTool::new()),
            Box::new(TerraformDestroyTool::new()),
            Box::new(TerraformOutputTool::new()),
        ]
    }

    /// Check if terraform is available
    pub fn is_available() -> bool {
        which::which("terraform").is_ok()
    }
}

// ============================================================================
// Terraform Init Tool
// ============================================================================

/// Initialize Terraform working directory
pub struct TerraformInitTool {
    config: ToolConfig,
}

impl TerraformInitTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to Terraform configuration",
                    "default": "."
                },
                "upgrade": {
                    "type": "boolean",
                    "description": "Upgrade providers and modules",
                    "default": false
                },
                "reconfigure": {
                    "type": "boolean",
                    "description": "Reconfigure backend",
                    "default": false
                },
                "backend_config": {
                    "type": "object",
                    "description": "Backend configuration values",
                    "additionalProperties": { "type": "string" }
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "terraform_init",
                "Initialize a Terraform working directory. Downloads providers and modules.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for TerraformInitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformInitTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let upgrade: bool = input.get_arg("upgrade").unwrap_or(false);
        let reconfigure: bool = input.get_arg("reconfigure").unwrap_or(false);
        let backend_config: std::collections::HashMap<String, String> = input
            .get_arg("backend_config")
            .unwrap_or_default();

        let mut args = vec!["init".to_string(), "-no-color".to_string()];

        if upgrade {
            args.push("-upgrade".to_string());
        }

        if reconfigure {
            args.push("-reconfigure".to_string());
        }

        for (key, value) in &backend_config {
            args.push(format!("-backend-config={}={}", key, value));
        }

        debug!(args = ?args, path = %path, "Executing terraform init");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("terraform", &args_str, Some(&path), 300).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "initialized": true,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "terraform init failed: {}",
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
// Terraform Plan Tool
// ============================================================================

/// Create Terraform execution plan
pub struct TerraformPlanTool {
    config: ToolConfig,
}

impl TerraformPlanTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to Terraform configuration",
                    "default": "."
                },
                "out": {
                    "type": "string",
                    "description": "Save plan to file"
                },
                "var": {
                    "type": "object",
                    "description": "Variable values",
                    "additionalProperties": { "type": "string" }
                },
                "var_file": {
                    "type": "string",
                    "description": "Path to variable file"
                },
                "target": {
                    "type": "array",
                    "description": "Resource addresses to target",
                    "items": { "type": "string" }
                },
                "destroy": {
                    "type": "boolean",
                    "description": "Create destroy plan",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "terraform_plan",
                "Create a Terraform execution plan. Shows what changes will be made.",
                parameters,
                600,
            ),
        }
    }
}

impl Default for TerraformPlanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformPlanTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let out: Option<String> = input.get_arg("out").ok();
        let var: std::collections::HashMap<String, String> = input.get_arg("var").unwrap_or_default();
        let var_file: Option<String> = input.get_arg("var_file").ok();
        let target: Vec<String> = input.get_arg("target").unwrap_or_default();
        let destroy: bool = input.get_arg("destroy").unwrap_or(false);

        let mut args = vec!["plan".to_string(), "-no-color".to_string()];

        if let Some(ref o) = out {
            args.push(format!("-out={}", o));
        }

        for (key, value) in &var {
            args.push(format!("-var={}={}", key, value));
        }

        if let Some(ref vf) = var_file {
            args.push(format!("-var-file={}", vf));
        }

        for t in &target {
            args.push(format!("-target={}", t));
        }

        if destroy {
            args.push("-destroy".to_string());
        }

        debug!(args = ?args, path = %path, "Executing terraform plan");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("terraform", &args_str, Some(&path), 600).await;

        match result {
            Ok(output) => {
                if output.success {
                    // Parse plan summary
                    let stdout = &output.stdout;
                    let has_changes = !stdout.contains("No changes.");

                    Ok(ToolResult::success(serde_json::json!({
                        "planned": true,
                        "has_changes": has_changes,
                        "plan_file": out,
                        "output": stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "terraform plan failed: {}",
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
// Terraform Apply Tool
// ============================================================================

/// Apply Terraform changes
pub struct TerraformApplyTool {
    config: ToolConfig,
}

impl TerraformApplyTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to Terraform configuration or plan file",
                    "default": "."
                },
                "auto_approve": {
                    "type": "boolean",
                    "description": "Skip interactive approval",
                    "default": true
                },
                "var": {
                    "type": "object",
                    "description": "Variable values",
                    "additionalProperties": { "type": "string" }
                },
                "var_file": {
                    "type": "string",
                    "description": "Path to variable file"
                },
                "target": {
                    "type": "array",
                    "description": "Resource addresses to target",
                    "items": { "type": "string" }
                },
                "plan_file": {
                    "type": "string",
                    "description": "Apply a saved plan file"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "terraform_apply",
                "Apply Terraform changes to infrastructure.",
                parameters,
                1800, // 30 minute timeout for apply
            ),
        }
    }
}

impl Default for TerraformApplyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformApplyTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let auto_approve: bool = input.get_arg("auto_approve").unwrap_or(true);
        let var: std::collections::HashMap<String, String> = input.get_arg("var").unwrap_or_default();
        let var_file: Option<String> = input.get_arg("var_file").ok();
        let target: Vec<String> = input.get_arg("target").unwrap_or_default();
        let plan_file: Option<String> = input.get_arg("plan_file").ok();

        let mut args = vec!["apply".to_string(), "-no-color".to_string()];

        if auto_approve {
            args.push("-auto-approve".to_string());
        }

        // If applying a plan file, don't add var/target options
        if let Some(ref pf) = plan_file {
            args.push(pf.clone());
        } else {
            for (key, value) in &var {
                args.push(format!("-var={}={}", key, value));
            }

            if let Some(ref vf) = var_file {
                args.push(format!("-var-file={}", vf));
            }

            for t in &target {
                args.push(format!("-target={}", t));
            }
        }

        debug!(args = ?args, path = %path, "Executing terraform apply");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("terraform", &args_str, Some(&path), 1800).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "applied": true,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "terraform apply failed: {}",
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
// Terraform Destroy Tool
// ============================================================================

/// Destroy Terraform infrastructure
pub struct TerraformDestroyTool {
    config: ToolConfig,
}

impl TerraformDestroyTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to Terraform configuration",
                    "default": "."
                },
                "auto_approve": {
                    "type": "boolean",
                    "description": "Skip interactive approval",
                    "default": false
                },
                "var": {
                    "type": "object",
                    "description": "Variable values",
                    "additionalProperties": { "type": "string" }
                },
                "var_file": {
                    "type": "string",
                    "description": "Path to variable file"
                },
                "target": {
                    "type": "array",
                    "description": "Resource addresses to target",
                    "items": { "type": "string" }
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "terraform_destroy",
                "Destroy Terraform-managed infrastructure. USE WITH CAUTION.",
                parameters,
                1800,
            ),
        }
    }
}

impl Default for TerraformDestroyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformDestroyTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let auto_approve: bool = input.get_arg("auto_approve").unwrap_or(false);
        let var: std::collections::HashMap<String, String> = input.get_arg("var").unwrap_or_default();
        let var_file: Option<String> = input.get_arg("var_file").ok();
        let target: Vec<String> = input.get_arg("target").unwrap_or_default();

        // Safety check - require explicit auto_approve for destroy
        if !auto_approve {
            return Ok(ToolResult::error(
                "terraform destroy requires 'auto_approve: true' for safety. This is a destructive operation."
            ));
        }

        let mut args = vec![
            "destroy".to_string(),
            "-no-color".to_string(),
            "-auto-approve".to_string(),
        ];

        for (key, value) in &var {
            args.push(format!("-var={}={}", key, value));
        }

        if let Some(ref vf) = var_file {
            args.push(format!("-var-file={}", vf));
        }

        for t in &target {
            args.push(format!("-target={}", t));
        }

        debug!(args = ?args, path = %path, "Executing terraform destroy");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("terraform", &args_str, Some(&path), 1800).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "destroyed": true,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "terraform destroy failed: {}",
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
// Terraform Output Tool
// ============================================================================

/// Get Terraform outputs
pub struct TerraformOutputTool {
    config: ToolConfig,
}

impl TerraformOutputTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to Terraform configuration",
                    "default": "."
                },
                "name": {
                    "type": "string",
                    "description": "Specific output name (omit for all)"
                },
                "json": {
                    "type": "boolean",
                    "description": "Output as JSON",
                    "default": true
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "terraform_output",
                "Read output values from Terraform state.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for TerraformOutputTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerraformOutputTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let name: Option<String> = input.get_arg("name").ok();
        let json: bool = input.get_arg("json").unwrap_or(true);

        let mut args = vec!["output".to_string(), "-no-color".to_string()];

        if json {
            args.push("-json".to_string());
        }

        if let Some(ref n) = name {
            args.push(n.clone());
        }

        debug!(args = ?args, path = %path, "Executing terraform output");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("terraform", &args_str, Some(&path), 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let outputs = if json {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "outputs": outputs
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "terraform output failed: {}",
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
