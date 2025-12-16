//! AWS CLI Tools
//!
//! Tools for AWS operations via AWS CLI.
//!
//! ## Available Tools
//!
//! - `aws_s3` - S3 operations (ls, cp, sync, rm)
//! - `aws_ec2` - EC2 operations (describe-instances, start, stop)
//! - `aws_logs` - CloudWatch Logs queries
//! - `aws_iam` - IAM operations
//! - `aws_lambda` - Lambda operations
//! - `aws_ecs` - ECS operations
//!
//! ## Prerequisites
//!
//! - AWS CLI v2 must be installed
//! - Valid AWS credentials configured (aws configure or env vars)
//!
//! ## MCP Alternative
//!
//! Use AWS-specific MCP servers for advanced operations.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all AWS tools
pub struct AwsTools;

impl AwsTools {
    /// Get all AWS tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(AwsS3Tool::new()),
            Box::new(AwsEc2Tool::new()),
            Box::new(AwsLogsTool::new()),
            Box::new(AwsIamTool::new()),
            Box::new(AwsLambdaTool::new()),
            Box::new(AwsEcsTool::new()),
        ]
    }

    /// Check if AWS CLI is available
    pub fn is_available() -> bool {
        which::which("aws").is_ok()
    }
}

// ============================================================================
// AWS S3 Tool
// ============================================================================

/// S3 operations
pub struct AwsS3Tool {
    config: ToolConfig,
}

impl AwsS3Tool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "S3 subcommand: ls, cp, sync, rm, mb, rb",
                    "enum": ["ls", "cp", "sync", "rm", "mb", "rb", "mv"]
                },
                "source": {
                    "type": "string",
                    "description": "Source path (local or s3://bucket/key)"
                },
                "destination": {
                    "type": "string",
                    "description": "Destination path (for cp, sync, mv)"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Recursive operation",
                    "default": false
                },
                "region": {
                    "type": "string",
                    "description": "AWS region"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_s3",
                "AWS S3 operations: list, copy, sync, and manage objects/buckets.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for AwsS3Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsS3Tool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let source: Option<String> = input.get_arg("source").ok();
        let destination: Option<String> = input.get_arg("destination").ok();
        let recursive: bool = input.get_arg("recursive").unwrap_or(false);
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["s3".to_string(), command.clone()];

        if let Some(ref src) = source {
            args.push(src.clone());
        }

        if let Some(ref dest) = destination {
            args.push(dest.clone());
        }

        if recursive {
            args.push("--recursive".to_string());
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = profile {
            args.push("--profile".to_string());
            args.push(p.clone());
        }

        debug!(args = ?args, "Executing aws s3");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 300).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "output": output.stdout,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "aws s3 {} failed: {}",
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
// AWS EC2 Tool
// ============================================================================

/// EC2 operations
pub struct AwsEc2Tool {
    config: ToolConfig,
}

impl AwsEc2Tool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "EC2 subcommand",
                    "enum": [
                        "describe-instances", "start-instances", "stop-instances",
                        "reboot-instances", "terminate-instances", "describe-security-groups",
                        "describe-vpcs", "describe-subnets"
                    ]
                },
                "instance_ids": {
                    "type": "array",
                    "description": "Instance IDs for operations",
                    "items": { "type": "string" }
                },
                "filters": {
                    "type": "array",
                    "description": "Filters (e.g., Name=tag:Name,Values=prod-*)",
                    "items": { "type": "string" }
                },
                "region": {
                    "type": "string",
                    "description": "AWS region"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                },
                "output": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "text", "table"],
                    "default": "json"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_ec2",
                "AWS EC2 operations: describe, start, stop, and manage instances.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsEc2Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsEc2Tool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let instance_ids: Vec<String> = input.get_arg("instance_ids").unwrap_or_default();
        let filters: Vec<String> = input.get_arg("filters").unwrap_or_default();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();
        let output_format: String = input.get_arg("output").unwrap_or_else(|_| "json".to_string());

        let mut args = vec!["ec2".to_string(), command.clone()];

        if !instance_ids.is_empty() {
            args.push("--instance-ids".to_string());
            args.extend(instance_ids);
        }

        for filter in &filters {
            args.push("--filters".to_string());
            args.push(filter.clone());
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = profile {
            args.push("--profile".to_string());
            args.push(p.clone());
        }

        args.push("--output".to_string());
        args.push(output_format.clone());

        debug!(args = ?args, "Executing aws ec2");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 120).await;

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
                        "aws ec2 {} failed: {}",
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
// AWS CloudWatch Logs Tool
// ============================================================================

/// CloudWatch Logs operations
pub struct AwsLogsTool {
    config: ToolConfig,
}

impl AwsLogsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Logs subcommand",
                    "enum": [
                        "describe-log-groups", "describe-log-streams",
                        "filter-log-events", "get-log-events", "tail"
                    ]
                },
                "log_group_name": {
                    "type": "string",
                    "description": "Log group name"
                },
                "log_stream_name": {
                    "type": "string",
                    "description": "Log stream name"
                },
                "filter_pattern": {
                    "type": "string",
                    "description": "Filter pattern for log events"
                },
                "start_time": {
                    "type": "string",
                    "description": "Start time (Unix timestamp in ms)"
                },
                "end_time": {
                    "type": "string",
                    "description": "End time (Unix timestamp in ms)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of events",
                    "default": 100
                },
                "region": {
                    "type": "string",
                    "description": "AWS region"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_logs",
                "AWS CloudWatch Logs: query and retrieve log events.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsLogsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsLogsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let log_group_name: Option<String> = input.get_arg("log_group_name").ok();
        let log_stream_name: Option<String> = input.get_arg("log_stream_name").ok();
        let filter_pattern: Option<String> = input.get_arg("filter_pattern").ok();
        let start_time: Option<String> = input.get_arg("start_time").ok();
        let end_time: Option<String> = input.get_arg("end_time").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(100);
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["logs".to_string(), command.clone()];

        if let Some(ref lg) = log_group_name {
            args.push("--log-group-name".to_string());
            args.push(lg.clone());
        }

        if let Some(ref ls) = log_stream_name {
            args.push("--log-stream-name".to_string());
            args.push(ls.clone());
        }

        if let Some(ref fp) = filter_pattern {
            args.push("--filter-pattern".to_string());
            args.push(fp.clone());
        }

        if let Some(ref st) = start_time {
            args.push("--start-time".to_string());
            args.push(st.clone());
        }

        if let Some(ref et) = end_time {
            args.push("--end-time".to_string());
            args.push(et.clone());
        }

        if command == "filter-log-events" || command == "get-log-events" {
            args.push("--limit".to_string());
            args.push(limit.to_string());
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = profile {
            args.push("--profile".to_string());
            args.push(p.clone());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        debug!(args = ?args, "Executing aws logs");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                        serde_json::json!({ "raw": output.stdout })
                    });

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "aws logs {} failed: {}",
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
// AWS IAM Tool
// ============================================================================

/// IAM operations
pub struct AwsIamTool {
    config: ToolConfig,
}

impl AwsIamTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "IAM subcommand",
                    "enum": [
                        "list-users", "list-roles", "list-policies",
                        "get-user", "get-role", "get-policy",
                        "list-attached-role-policies", "list-attached-user-policies"
                    ]
                },
                "user_name": {
                    "type": "string",
                    "description": "IAM user name"
                },
                "role_name": {
                    "type": "string",
                    "description": "IAM role name"
                },
                "policy_arn": {
                    "type": "string",
                    "description": "Policy ARN"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_iam",
                "AWS IAM: list and describe users, roles, and policies.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AwsIamTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsIamTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let user_name: Option<String> = input.get_arg("user_name").ok();
        let role_name: Option<String> = input.get_arg("role_name").ok();
        let policy_arn: Option<String> = input.get_arg("policy_arn").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["iam".to_string(), command.clone()];

        if let Some(ref u) = user_name {
            args.push("--user-name".to_string());
            args.push(u.clone());
        }

        if let Some(ref r) = role_name {
            args.push("--role-name".to_string());
            args.push(r.clone());
        }

        if let Some(ref p) = policy_arn {
            args.push("--policy-arn".to_string());
            args.push(p.clone());
        }

        if let Some(ref pr) = profile {
            args.push("--profile".to_string());
            args.push(pr.clone());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        debug!(args = ?args, "Executing aws iam");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                        serde_json::json!({ "raw": output.stdout })
                    });

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "aws iam {} failed: {}",
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
// AWS Lambda Tool
// ============================================================================

/// Lambda operations
pub struct AwsLambdaTool {
    config: ToolConfig,
}

impl AwsLambdaTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Lambda subcommand",
                    "enum": [
                        "list-functions", "get-function", "invoke",
                        "list-versions-by-function", "get-function-configuration"
                    ]
                },
                "function_name": {
                    "type": "string",
                    "description": "Lambda function name or ARN"
                },
                "payload": {
                    "type": "string",
                    "description": "JSON payload for invoke"
                },
                "region": {
                    "type": "string",
                    "description": "AWS region"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_lambda",
                "AWS Lambda: list, describe, and invoke functions.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsLambdaTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsLambdaTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let function_name: Option<String> = input.get_arg("function_name").ok();
        let payload: Option<String> = input.get_arg("payload").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["lambda".to_string(), command.clone()];

        if let Some(ref fn_name) = function_name {
            args.push("--function-name".to_string());
            args.push(fn_name.clone());
        }

        if command == "invoke" {
            if let Some(ref p) = payload {
                args.push("--payload".to_string());
                args.push(p.clone());
            }
            args.push("/dev/stdout".to_string()); // Output file for invoke
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref pr) = profile {
            args.push("--profile".to_string());
            args.push(pr.clone());
        }

        if command != "invoke" {
            args.push("--output".to_string());
            args.push("json".to_string());
        }

        debug!(args = ?args, "Executing aws lambda");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                        serde_json::json!({ "raw": output.stdout })
                    });

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "aws lambda {} failed: {}",
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
// AWS ECS Tool
// ============================================================================

/// ECS operations
pub struct AwsEcsTool {
    config: ToolConfig,
}

impl AwsEcsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "ECS subcommand",
                    "enum": [
                        "list-clusters", "list-services", "list-tasks",
                        "describe-clusters", "describe-services", "describe-tasks",
                        "update-service", "stop-task"
                    ]
                },
                "cluster": {
                    "type": "string",
                    "description": "ECS cluster name or ARN"
                },
                "service": {
                    "type": "string",
                    "description": "ECS service name"
                },
                "tasks": {
                    "type": "array",
                    "description": "Task ARNs",
                    "items": { "type": "string" }
                },
                "desired_count": {
                    "type": "integer",
                    "description": "Desired count for update-service"
                },
                "region": {
                    "type": "string",
                    "description": "AWS region"
                },
                "profile": {
                    "type": "string",
                    "description": "AWS profile name"
                }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_ecs",
                "AWS ECS: manage clusters, services, and tasks.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsEcsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsEcsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let cluster: Option<String> = input.get_arg("cluster").ok();
        let service: Option<String> = input.get_arg("service").ok();
        let tasks: Vec<String> = input.get_arg("tasks").unwrap_or_default();
        let desired_count: Option<i32> = input.get_arg("desired_count").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["ecs".to_string(), command.clone()];

        if let Some(ref c) = cluster {
            args.push("--cluster".to_string());
            args.push(c.clone());
        }

        if let Some(ref s) = service {
            if command.contains("service") {
                args.push("--services".to_string());
            } else {
                args.push("--service".to_string());
            }
            args.push(s.clone());
        }

        if !tasks.is_empty() {
            args.push("--tasks".to_string());
            args.extend(tasks);
        }

        if let Some(dc) = desired_count {
            args.push("--desired-count".to_string());
            args.push(dc.to_string());
        }

        if let Some(ref r) = region {
            args.push("--region".to_string());
            args.push(r.clone());
        }

        if let Some(ref pr) = profile {
            args.push("--profile".to_string());
            args.push(pr.clone());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        debug!(args = ?args, "Executing aws ecs");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    let data = serde_json::from_str(&output.stdout).unwrap_or_else(|_| {
                        serde_json::json!({ "raw": output.stdout })
                    });

                    Ok(ToolResult::success(serde_json::json!({
                        "data": data,
                        "command": command
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "aws ecs {} failed: {}",
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
