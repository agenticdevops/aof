//! Azure CLI Tools
//!
//! Tools for Azure operations via Azure CLI.
//!
//! ## Available Tools
//!
//! - `azure_vm` - VM operations (list, show, start, stop, restart, delete)
//! - `azure_storage` - Storage operations (account list, blob list, blob upload, blob download)
//! - `azure_aks` - AKS operations (list, show, get-credentials, scale)
//! - `azure_network` - Network operations (vnet list, nsg list, public-ip list)
//! - `azure_resource` - Resource Manager (group list, group show, resource list)
//! - `azure_keyvault` - Key Vault (secret list, secret show, secret set)
//! - `azure_monitor` - Monitor (metrics list, activity-log list, alert list)
//! - `azure_acr` - Container Registry (list, show, repository list)
//!
//! ## Prerequisites
//!
//! - Azure CLI must be installed
//! - Logged in via `az login`
//!
//! ## MCP Alternative
//!
//! Use Azure-specific MCP servers for advanced operations.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all Azure tools
pub struct AzureTools;

impl AzureTools {
    /// Get all Azure tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(AzureVmTool::new()),
            Box::new(AzureStorageTool::new()),
            Box::new(AzureAksTool::new()),
            Box::new(AzureNetworkTool::new()),
            Box::new(AzureResourceTool::new()),
            Box::new(AzureKeyvaultTool::new()),
            Box::new(AzureMonitorTool::new()),
            Box::new(AzureAcrTool::new()),
        ]
    }

    /// Check if Azure CLI is available
    pub fn is_available() -> bool {
        which::which("az").is_ok()
    }
}

// ============================================================================
// Azure VM Tool
// ============================================================================

/// Azure VM operations
pub struct AzureVmTool {
    config: ToolConfig,
}

impl AzureVmTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "VM subcommand",
                    "enum": ["list", "show", "start", "stop", "restart", "delete"]
                },
                "name": {
                    "type": "string",
                    "description": "VM name (required for show, start, stop, restart, delete)"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_vm",
                "Azure VM operations: list, show, start, stop, restart, delete.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AzureVmTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureVmTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let name: Option<String> = input.get_arg("name").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["vm".to_string(), command.clone()];

        if let Some(ref n) = name {
            args.push("--name".to_string());
            args.push(n.clone());
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az vm");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az vm {} failed: {}",
                        command, output.stderr
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
// Azure Storage Tool
// ============================================================================

/// Azure Storage operations
pub struct AzureStorageTool {
    config: ToolConfig,
}

impl AzureStorageTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Storage subcommand",
                    "enum": ["account-list", "blob-list", "blob-upload", "blob-download"]
                },
                "account_name": {
                    "type": "string",
                    "description": "Storage account name (required for blob operations)"
                },
                "container_name": {
                    "type": "string",
                    "description": "Container name (required for blob operations)"
                },
                "blob_name": {
                    "type": "string",
                    "description": "Blob name (required for upload/download)"
                },
                "file_path": {
                    "type": "string",
                    "description": "Local file path (for upload/download)"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_storage",
                "Azure Storage operations: account list, blob list, blob upload, blob download.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for AzureStorageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureStorageTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let account_name: Option<String> = input.get_arg("account_name").ok();
        let container_name: Option<String> = input.get_arg("container_name").ok();
        let blob_name: Option<String> = input.get_arg("blob_name").ok();
        let file_path: Option<String> = input.get_arg("file_path").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["storage".to_string()];

        match command.as_str() {
            "account-list" => {
                args.push("account".to_string());
                args.push("list".to_string());
            }
            "blob-list" => {
                args.push("blob".to_string());
                args.push("list".to_string());
            }
            "blob-upload" => {
                args.push("blob".to_string());
                args.push("upload".to_string());
            }
            "blob-download" => {
                args.push("blob".to_string());
                args.push("download".to_string());
            }
            _ => {}
        }

        if let Some(ref acc) = account_name {
            args.push("--account-name".to_string());
            args.push(acc.clone());
        }

        if let Some(ref cont) = container_name {
            args.push("--container-name".to_string());
            args.push(cont.clone());
        }

        if let Some(ref blob) = blob_name {
            args.push("--name".to_string());
            args.push(blob.clone());
        }

        if let Some(ref fp) = file_path {
            if command == "blob-upload" {
                args.push("--file".to_string());
                args.push(fp.clone());
            } else if command == "blob-download" {
                args.push("--file".to_string());
                args.push(fp.clone());
            }
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az storage");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 300).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az storage {} failed: {}",
                        command, output.stderr
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
// Azure AKS Tool
// ============================================================================

/// Azure AKS operations
pub struct AzureAksTool {
    config: ToolConfig,
}

impl AzureAksTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "AKS subcommand",
                    "enum": ["list", "show", "get-credentials", "scale"]
                },
                "name": {
                    "type": "string",
                    "description": "AKS cluster name (required for show, get-credentials, scale)"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "node_count": {
                    "type": "integer",
                    "description": "Node count for scaling"
                },
                "nodepool_name": {
                    "type": "string",
                    "description": "Node pool name (for scale command)"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_aks",
                "Azure AKS operations: list, show, get-credentials, scale.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AzureAksTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureAksTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let name: Option<String> = input.get_arg("name").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let node_count: Option<i32> = input.get_arg("node_count").ok();
        let nodepool_name: Option<String> = input.get_arg("nodepool_name").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["aks".to_string(), command.clone()];

        if let Some(ref n) = name {
            args.push("--name".to_string());
            args.push(n.clone());
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if command == "scale" {
            if let Some(nc) = node_count {
                args.push("--node-count".to_string());
                args.push(nc.to_string());
            }
            if let Some(ref np) = nodepool_name {
                args.push("--nodepool-name".to_string());
                args.push(np.clone());
            }
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az aks");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az aks {} failed: {}",
                        command, output.stderr
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
// Azure Network Tool
// ============================================================================

/// Azure Network operations
pub struct AzureNetworkTool {
    config: ToolConfig,
}

impl AzureNetworkTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Network subcommand",
                    "enum": ["vnet-list", "nsg-list", "public-ip-list"]
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_network",
                "Azure Network operations: vnet list, nsg list, public-ip list.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AzureNetworkTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureNetworkTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["network".to_string()];

        match command.as_str() {
            "vnet-list" => {
                args.push("vnet".to_string());
                args.push("list".to_string());
            }
            "nsg-list" => {
                args.push("nsg".to_string());
                args.push("list".to_string());
            }
            "public-ip-list" => {
                args.push("public-ip".to_string());
                args.push("list".to_string());
            }
            _ => {}
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az network");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az network {} failed: {}",
                        command, output.stderr
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
// Azure Resource Tool
// ============================================================================

/// Azure Resource Manager operations
pub struct AzureResourceTool {
    config: ToolConfig,
}

impl AzureResourceTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Resource subcommand",
                    "enum": ["group-list", "group-show", "resource-list"]
                },
                "name": {
                    "type": "string",
                    "description": "Resource group name (required for group-show)"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name (for resource-list)"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_resource",
                "Azure Resource Manager: group list, group show, resource list.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AzureResourceTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureResourceTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let name: Option<String> = input.get_arg("name").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec![];

        match command.as_str() {
            "group-list" => {
                args.push("group".to_string());
                args.push("list".to_string());
            }
            "group-show" => {
                args.push("group".to_string());
                args.push("show".to_string());
            }
            "resource-list" => {
                args.push("resource".to_string());
                args.push("list".to_string());
            }
            _ => {}
        }

        if let Some(ref n) = name {
            args.push("--name".to_string());
            args.push(n.clone());
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az {} failed: {}",
                        command, output.stderr
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
// Azure Key Vault Tool
// ============================================================================

/// Azure Key Vault operations
pub struct AzureKeyvaultTool {
    config: ToolConfig,
}

impl AzureKeyvaultTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Key Vault subcommand",
                    "enum": ["secret-list", "secret-show", "secret-set"]
                },
                "vault_name": {
                    "type": "string",
                    "description": "Key Vault name"
                },
                "secret_name": {
                    "type": "string",
                    "description": "Secret name (required for secret-show, secret-set)"
                },
                "secret_value": {
                    "type": "string",
                    "description": "Secret value (required for secret-set)"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_keyvault",
                "Azure Key Vault: secret list, secret show, secret set.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AzureKeyvaultTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureKeyvaultTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let vault_name: Option<String> = input.get_arg("vault_name").ok();
        let secret_name: Option<String> = input.get_arg("secret_name").ok();
        let secret_value: Option<String> = input.get_arg("secret_value").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["keyvault".to_string(), "secret".to_string()];

        match command.as_str() {
            "secret-list" => {
                args.push("list".to_string());
            }
            "secret-show" => {
                args.push("show".to_string());
            }
            "secret-set" => {
                args.push("set".to_string());
            }
            _ => {}
        }

        if let Some(ref vn) = vault_name {
            args.push("--vault-name".to_string());
            args.push(vn.clone());
        }

        if let Some(ref sn) = secret_name {
            args.push("--name".to_string());
            args.push(sn.clone());
        }

        if command == "secret-set" {
            if let Some(ref sv) = secret_value {
                args.push("--value".to_string());
                args.push(sv.clone());
            }
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az keyvault");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az keyvault {} failed: {}",
                        command, output.stderr
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
// Azure Monitor Tool
// ============================================================================

/// Azure Monitor operations
pub struct AzureMonitorTool {
    config: ToolConfig,
}

impl AzureMonitorTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Monitor subcommand",
                    "enum": ["metrics-list", "activity-log-list", "alert-list"]
                },
                "resource": {
                    "type": "string",
                    "description": "Resource ID or name"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "start_time": {
                    "type": "string",
                    "description": "Start time (ISO 8601 format)"
                },
                "end_time": {
                    "type": "string",
                    "description": "End time (ISO 8601 format)"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_monitor",
                "Azure Monitor: metrics list, activity-log list, alert list.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AzureMonitorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureMonitorTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let resource: Option<String> = input.get_arg("resource").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let start_time: Option<String> = input.get_arg("start_time").ok();
        let end_time: Option<String> = input.get_arg("end_time").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["monitor".to_string()];

        match command.as_str() {
            "metrics-list" => {
                args.push("metrics".to_string());
                args.push("list".to_string());
            }
            "activity-log-list" => {
                args.push("activity-log".to_string());
                args.push("list".to_string());
            }
            "alert-list" => {
                args.push("alert".to_string());
                args.push("list".to_string());
            }
            _ => {}
        }

        if let Some(ref res) = resource {
            args.push("--resource".to_string());
            args.push(res.clone());
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref st) = start_time {
            args.push("--start-time".to_string());
            args.push(st.clone());
        }

        if let Some(ref et) = end_time {
            args.push("--end-time".to_string());
            args.push(et.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az monitor");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az monitor {} failed: {}",
                        command, output.stderr
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
// Azure Container Registry Tool
// ============================================================================

/// Azure Container Registry operations
pub struct AzureAcrTool {
    config: ToolConfig,
}

impl AzureAcrTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "ACR subcommand",
                    "enum": ["list", "show", "repository-list"]
                },
                "name": {
                    "type": "string",
                    "description": "Registry name (required for show, repository-list)"
                },
                "resource_group": {
                    "type": "string",
                    "description": "Resource group name"
                },
                "subscription": {
                    "type": "string",
                    "description": "Azure subscription ID or name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "tsv"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "azure_acr",
                "Azure Container Registry: list, show, repository list.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AzureAcrTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AzureAcrTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let name: Option<String> = input.get_arg("name").ok();
        let resource_group: Option<String> = input.get_arg("resource_group").ok();
        let subscription: Option<String> = input.get_arg("subscription").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["acr".to_string()];

        match command.as_str() {
            "list" => {
                args.push("list".to_string());
            }
            "show" => {
                args.push("show".to_string());
            }
            "repository-list" => {
                args.push("repository".to_string());
                args.push("list".to_string());
            }
            _ => {}
        }

        if let Some(ref n) = name {
            args.push("--name".to_string());
            args.push(n.clone());
        }

        if let Some(ref rg) = resource_group {
            args.push("--resource-group".to_string());
            args.push(rg.clone());
        }

        if let Some(ref sub) = subscription {
            args.push("--subscription".to_string());
            args.push(sub.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing az acr");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("az", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if output_format == "json" {
                        serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": output.stdout })
                        })
                    } else {
                        serde_json::json!({ "output": output.stdout })
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "az acr {} failed: {}",
                        command, output.stderr
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
