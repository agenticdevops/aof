//! GCP gcloud CLI Tools
//!
//! Tools for Google Cloud operations via gcloud CLI.
//!
//! ## Available Tools
//!
//! - `gcp_compute` - Compute Engine operations (instances list, describe, start, stop)
//! - `gcp_storage` - Cloud Storage operations (ls, cp, rm, mb)
//! - `gcp_gke` - GKE operations (clusters list, describe, get-credentials)
//! - `gcp_iam` - IAM operations (roles list, service-accounts list, keys list)
//! - `gcp_logging` - Cloud Logging operations (read, logs list)
//! - `gcp_pubsub` - Pub/Sub operations (topics list, subscriptions list, publish)
//! - `gcp_sql` - Cloud SQL operations (instances list, describe, backups list)
//! - `gcp_functions` - Cloud Functions operations (list, describe, call)
//!
//! ## Prerequisites
//!
//! - Google Cloud SDK must be installed
//! - Authenticated via `gcloud auth login`
//! - Project set via `gcloud config set project`
//!
//! ## MCP Alternative
//!
//! Use GCP-specific MCP servers for advanced operations.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all GCP tools
pub struct GcpTools;

impl GcpTools {
    /// Get all GCP tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GcpComputeTool::new()),
            Box::new(GcpStorageTool::new()),
            Box::new(GcpGkeTool::new()),
            Box::new(GcpIamTool::new()),
            Box::new(GcpLoggingTool::new()),
            Box::new(GcpPubsubTool::new()),
            Box::new(GcpSqlTool::new()),
            Box::new(GcpFunctionsTool::new()),
        ]
    }

    /// Check if gcloud CLI is available
    pub fn is_available() -> bool {
        which::which("gcloud").is_ok()
    }
}

// ============================================================================
// GCP Compute Engine Tool
// ============================================================================

/// Compute Engine operations
pub struct GcpComputeTool {
    config: ToolConfig,
}

impl GcpComputeTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Compute subcommand",
                    "enum": [
                        "instances list", "instances describe", "instances start",
                        "instances stop", "instances reset", "instances delete",
                        "disks list", "disks describe", "machine-types list"
                    ]
                },
                "instance_name": {
                    "type": "string",
                    "description": "Instance name (for describe, start, stop, etc.)"
                },
                "zone": {
                    "type": "string",
                    "description": "Zone (e.g., us-central1-a)"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "filter": {
                    "type": "string",
                    "description": "Filter expression (e.g., name=prod-*)"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml", "table"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_compute",
                "GCP Compute Engine operations: list, describe, start, stop instances.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GcpComputeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpComputeTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let instance_name: Option<String> = input.get_arg("instance_name").ok();
        let zone: Option<String> = input.get_arg("zone").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let filter: Option<String> = input.get_arg("filter").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["compute".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref name) = instance_name {
            args.push(name.clone());
        }

        if let Some(ref z) = zone {
            args.push("--zone".to_string());
            args.push(z.clone());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        if let Some(ref f) = filter {
            args.push("--filter".to_string());
            args.push(f.clone());
        }

        args.push("--format".to_string());
        args.push(format.clone());

        debug!(args = ?args, "Executing gcloud compute");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" {
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
                        "gcloud compute {} failed: {}",
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
// GCP Cloud Storage Tool
// ============================================================================

/// Cloud Storage operations
pub struct GcpStorageTool {
    config: ToolConfig,
}

impl GcpStorageTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Storage subcommand",
                    "enum": ["ls", "cp", "rm", "mb", "rb", "mv", "rsync"]
                },
                "source": {
                    "type": "string",
                    "description": "Source path (local or gs://bucket/object)"
                },
                "destination": {
                    "type": "string",
                    "description": "Destination path (for cp, mv, rsync)"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Recursive operation",
                    "default": false
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_storage",
                "GCP Cloud Storage operations: list, copy, move, and manage objects/buckets.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for GcpStorageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpStorageTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let source: Option<String> = input.get_arg("source").ok();
        let destination: Option<String> = input.get_arg("destination").ok();
        let recursive: bool = input.get_arg("recursive").unwrap_or(false);
        let project: Option<String> = input.get_arg("project").ok();

        let mut args = vec!["storage".to_string(), command.clone()];

        if let Some(ref src) = source {
            args.push(src.clone());
        }

        if let Some(ref dest) = destination {
            args.push(dest.clone());
        }

        if recursive && (command == "cp" || command == "rm" || command == "rsync") {
            args.push("-r".to_string());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        debug!(args = ?args, "Executing gcloud storage");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 300).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "output": output.stdout,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "gcloud storage {} failed: {}",
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
// GCP GKE Tool
// ============================================================================

/// GKE operations
pub struct GcpGkeTool {
    config: ToolConfig,
}

impl GcpGkeTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "GKE subcommand",
                    "enum": [
                        "clusters list", "clusters describe", "clusters get-credentials",
                        "clusters create", "clusters delete", "clusters upgrade",
                        "node-pools list", "node-pools describe"
                    ]
                },
                "cluster_name": {
                    "type": "string",
                    "description": "Cluster name (for describe, get-credentials, etc.)"
                },
                "zone": {
                    "type": "string",
                    "description": "Zone (e.g., us-central1-a)"
                },
                "region": {
                    "type": "string",
                    "description": "Region (e.g., us-central1)"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml", "table"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_gke",
                "GCP GKE operations: list, describe, and manage Kubernetes clusters.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GcpGkeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpGkeTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let cluster_name: Option<String> = input.get_arg("cluster_name").ok();
        let zone: Option<String> = input.get_arg("zone").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["container".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref name) = cluster_name {
            args.push(name.clone());
        }

        if let Some(ref z) = zone {
            args.push("--zone".to_string());
            args.push(z.clone());
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        // Don't add format for get-credentials
        if !command.contains("get-credentials") {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        debug!(args = ?args, "Executing gcloud container");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" && !command.contains("get-credentials") {
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
                        "gcloud container {} failed: {}",
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
// GCP IAM Tool
// ============================================================================

/// IAM operations
pub struct GcpIamTool {
    config: ToolConfig,
}

impl GcpIamTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "IAM subcommand",
                    "enum": [
                        "roles list", "roles describe",
                        "service-accounts list", "service-accounts describe",
                        "service-accounts keys list", "service-accounts keys create"
                    ]
                },
                "role_name": {
                    "type": "string",
                    "description": "Role name or ID (for describe)"
                },
                "service_account": {
                    "type": "string",
                    "description": "Service account email"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml", "table"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_iam",
                "GCP IAM operations: list and describe roles, service accounts, and keys.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GcpIamTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpIamTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let role_name: Option<String> = input.get_arg("role_name").ok();
        let service_account: Option<String> = input.get_arg("service_account").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["iam".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref role) = role_name {
            args.push(role.clone());
        }

        if let Some(ref sa) = service_account {
            if command.contains("service-accounts") {
                args.push("--iam-account".to_string());
                args.push(sa.clone());
            }
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        args.push("--format".to_string());
        args.push(format.clone());

        debug!(args = ?args, "Executing gcloud iam");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" {
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
                        "gcloud iam {} failed: {}",
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
// GCP Cloud Logging Tool
// ============================================================================

/// Cloud Logging operations
pub struct GcpLoggingTool {
    config: ToolConfig,
}

impl GcpLoggingTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Logging subcommand",
                    "enum": ["read", "logs list", "logs delete", "sinks list"]
                },
                "filter": {
                    "type": "string",
                    "description": "Log filter expression"
                },
                "log_name": {
                    "type": "string",
                    "description": "Log name (for logs commands)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of entries",
                    "default": 100
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_logging",
                "GCP Cloud Logging operations: read and query log entries.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GcpLoggingTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpLoggingTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let filter: Option<String> = input.get_arg("filter").ok();
        let log_name: Option<String> = input.get_arg("log_name").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(100);
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["logging".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref name) = log_name {
            args.push(name.clone());
        }

        if let Some(ref f) = filter {
            args.push("--filter".to_string());
            args.push(f.clone());
        }

        if command == "read" {
            args.push("--limit".to_string());
            args.push(limit.to_string());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        args.push("--format".to_string());
        args.push(format.clone());

        debug!(args = ?args, "Executing gcloud logging");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" {
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
                        "gcloud logging {} failed: {}",
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
// GCP Pub/Sub Tool
// ============================================================================

/// Pub/Sub operations
pub struct GcpPubsubTool {
    config: ToolConfig,
}

impl GcpPubsubTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Pub/Sub subcommand",
                    "enum": [
                        "topics list", "topics describe", "topics create", "topics delete",
                        "topics publish",
                        "subscriptions list", "subscriptions describe",
                        "subscriptions create", "subscriptions delete"
                    ]
                },
                "topic_name": {
                    "type": "string",
                    "description": "Topic name (for topic operations)"
                },
                "subscription_name": {
                    "type": "string",
                    "description": "Subscription name (for subscription operations)"
                },
                "message": {
                    "type": "string",
                    "description": "Message to publish"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_pubsub",
                "GCP Pub/Sub operations: manage topics, subscriptions, and publish messages.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GcpPubsubTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpPubsubTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let topic_name: Option<String> = input.get_arg("topic_name").ok();
        let subscription_name: Option<String> = input.get_arg("subscription_name").ok();
        let message: Option<String> = input.get_arg("message").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["pubsub".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref topic) = topic_name {
            args.push(topic.clone());
        }

        if let Some(ref sub) = subscription_name {
            args.push(sub.clone());
        }

        if command == "topics publish" {
            if let Some(ref msg) = message {
                args.push("--message".to_string());
                args.push(msg.clone());
            }
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        // Don't add format for publish
        if !command.contains("publish") {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        debug!(args = ?args, "Executing gcloud pubsub");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" && !command.contains("publish") {
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
                        "gcloud pubsub {} failed: {}",
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
// GCP Cloud SQL Tool
// ============================================================================

/// Cloud SQL operations
pub struct GcpSqlTool {
    config: ToolConfig,
}

impl GcpSqlTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Cloud SQL subcommand",
                    "enum": [
                        "instances list", "instances describe", "instances create",
                        "instances delete", "instances restart", "instances patch",
                        "backups list", "backups describe", "backups create"
                    ]
                },
                "instance_name": {
                    "type": "string",
                    "description": "Cloud SQL instance name"
                },
                "backup_id": {
                    "type": "string",
                    "description": "Backup ID (for backup operations)"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml", "table"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_sql",
                "GCP Cloud SQL operations: manage instances and backups.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GcpSqlTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpSqlTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let instance_name: Option<String> = input.get_arg("instance_name").ok();
        let backup_id: Option<String> = input.get_arg("backup_id").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["sql".to_string()];
        args.extend(command.split_whitespace().map(|s| s.to_string()));

        if let Some(ref name) = instance_name {
            args.push(name.clone());
        }

        if let Some(ref backup) = backup_id {
            args.push(backup.clone());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        args.push("--format".to_string());
        args.push(format.clone());

        debug!(args = ?args, "Executing gcloud sql");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" {
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
                        "gcloud sql {} failed: {}",
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
// GCP Cloud Functions Tool
// ============================================================================

/// Cloud Functions operations
pub struct GcpFunctionsTool {
    config: ToolConfig,
}

impl GcpFunctionsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Cloud Functions subcommand",
                    "enum": [
                        "list", "describe", "deploy", "delete",
                        "call", "logs read"
                    ]
                },
                "function_name": {
                    "type": "string",
                    "description": "Function name"
                },
                "data": {
                    "type": "string",
                    "description": "JSON data for function call"
                },
                "region": {
                    "type": "string",
                    "description": "Region (e.g., us-central1)"
                },
                "project": {
                    "type": "string",
                    "description": "GCP project ID"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "yaml"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "gcp_functions",
                "GCP Cloud Functions operations: list, describe, and call functions.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GcpFunctionsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GcpFunctionsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let function_name: Option<String> = input.get_arg("function_name").ok();
        let data: Option<String> = input.get_arg("data").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let project: Option<String> = input.get_arg("project").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["functions".to_string(), command.clone()];

        if let Some(ref name) = function_name {
            args.push(name.clone());
        }

        if command == "call" {
            if let Some(ref d) = data {
                args.push("--data".to_string());
                args.push(d.clone());
            }
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = project {
            args.push("--project".to_string());
            args.push(p.clone());
        }

        // Don't add format for call
        if command != "call" {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        debug!(args = ?args, "Executing gcloud functions");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("gcloud", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = if format == "json" && command != "call" {
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
                        "gcloud functions {} failed: {}",
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
