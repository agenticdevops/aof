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
            Box::new(AwsCloudFormationTool::new()),
            Box::new(AwsRdsTool::new()),
            Box::new(AwsSqsTool::new()),
            Box::new(AwsSnsTool::new()),
            Box::new(AwsCostTool::new()),
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

// ============================================================================
// AWS CloudFormation Tool
// ============================================================================

/// CloudFormation operations
pub struct AwsCloudFormationTool {
    config: ToolConfig,
}

impl AwsCloudFormationTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "CloudFormation subcommand",
                    "enum": [
                        "describe-stacks", "create-stack", "delete-stack",
                        "list-stack-resources", "describe-stack-events"
                    ]
                },
                "stack_name": {
                    "type": "string",
                    "description": "CloudFormation stack name"
                },
                "template_body": {
                    "type": "string",
                    "description": "CloudFormation template as JSON/YAML string"
                },
                "template_url": {
                    "type": "string",
                    "description": "S3 URL to CloudFormation template"
                },
                "parameters": {
                    "type": "array",
                    "description": "Stack parameters (ParameterKey=key,ParameterValue=value)",
                    "items": { "type": "string" }
                },
                "capabilities": {
                    "type": "array",
                    "description": "IAM capabilities (CAPABILITY_IAM, CAPABILITY_NAMED_IAM)",
                    "items": { "type": "string" }
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
                "aws_cloudformation",
                "AWS CloudFormation: manage infrastructure stacks and resources.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for AwsCloudFormationTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsCloudFormationTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let stack_name: Option<String> = input.get_arg("stack_name").ok();
        let template_body: Option<String> = input.get_arg("template_body").ok();
        let template_url: Option<String> = input.get_arg("template_url").ok();
        let parameters: Vec<String> = input.get_arg("parameters").unwrap_or_default();
        let capabilities: Vec<String> = input.get_arg("capabilities").unwrap_or_default();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["cloudformation".to_string(), command.clone()];

        if let Some(ref sn) = stack_name {
            args.push("--stack-name".to_string());
            args.push(sn.clone());
        }

        if let Some(ref tb) = template_body {
            args.push("--template-body".to_string());
            args.push(tb.clone());
        }

        if let Some(ref tu) = template_url {
            args.push("--template-url".to_string());
            args.push(tu.clone());
        }

        if !parameters.is_empty() {
            args.push("--parameters".to_string());
            args.extend(parameters);
        }

        if !capabilities.is_empty() {
            args.push("--capabilities".to_string());
            args.extend(capabilities);
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

        debug!(args = ?args, "Executing aws cloudformation");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("aws", &args_str, None, 300).await;

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
                        "aws cloudformation {} failed: {}",
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
// AWS RDS Tool
// ============================================================================

/// RDS operations
pub struct AwsRdsTool {
    config: ToolConfig,
}

impl AwsRdsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "RDS subcommand",
                    "enum": [
                        "describe-db-instances", "describe-db-clusters",
                        "create-db-snapshot", "describe-db-snapshots",
                        "stop-db-instance", "start-db-instance"
                    ]
                },
                "db_instance_identifier": {
                    "type": "string",
                    "description": "DB instance identifier"
                },
                "db_cluster_identifier": {
                    "type": "string",
                    "description": "DB cluster identifier"
                },
                "snapshot_identifier": {
                    "type": "string",
                    "description": "Snapshot identifier"
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
                "aws_rds",
                "AWS RDS: manage database instances, clusters, and snapshots.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsRdsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsRdsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let db_instance_identifier: Option<String> = input.get_arg("db_instance_identifier").ok();
        let db_cluster_identifier: Option<String> = input.get_arg("db_cluster_identifier").ok();
        let snapshot_identifier: Option<String> = input.get_arg("snapshot_identifier").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["rds".to_string(), command.clone()];

        if let Some(ref dbi) = db_instance_identifier {
            args.push("--db-instance-identifier".to_string());
            args.push(dbi.clone());
        }

        if let Some(ref dbc) = db_cluster_identifier {
            args.push("--db-cluster-identifier".to_string());
            args.push(dbc.clone());
        }

        if let Some(ref si) = snapshot_identifier {
            if command == "create-db-snapshot" {
                args.push("--db-snapshot-identifier".to_string());
            } else {
                args.push("--db-snapshot-identifier".to_string());
            }
            args.push(si.clone());
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

        debug!(args = ?args, "Executing aws rds");

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
                        "aws rds {} failed: {}",
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
// AWS SQS Tool
// ============================================================================

/// SQS operations
pub struct AwsSqsTool {
    config: ToolConfig,
}

impl AwsSqsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "SQS subcommand",
                    "enum": [
                        "list-queues", "send-message", "receive-message",
                        "delete-message", "get-queue-attributes", "purge-queue"
                    ]
                },
                "queue_url": {
                    "type": "string",
                    "description": "SQS queue URL"
                },
                "message_body": {
                    "type": "string",
                    "description": "Message body for send-message"
                },
                "receipt_handle": {
                    "type": "string",
                    "description": "Receipt handle for delete-message"
                },
                "max_messages": {
                    "type": "integer",
                    "description": "Maximum messages to receive (1-10)",
                    "default": 1
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
                "aws_sqs",
                "AWS SQS: manage message queues and send/receive messages.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AwsSqsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsSqsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let queue_url: Option<String> = input.get_arg("queue_url").ok();
        let message_body: Option<String> = input.get_arg("message_body").ok();
        let receipt_handle: Option<String> = input.get_arg("receipt_handle").ok();
        let max_messages: i32 = input.get_arg("max_messages").unwrap_or(1);
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["sqs".to_string(), command.clone()];

        if let Some(ref qu) = queue_url {
            args.push("--queue-url".to_string());
            args.push(qu.clone());
        }

        if let Some(ref mb) = message_body {
            args.push("--message-body".to_string());
            args.push(mb.clone());
        }

        if let Some(ref rh) = receipt_handle {
            args.push("--receipt-handle".to_string());
            args.push(rh.clone());
        }

        if command == "receive-message" {
            args.push("--max-number-of-messages".to_string());
            args.push(max_messages.to_string());
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

        debug!(args = ?args, "Executing aws sqs");

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
                        "aws sqs {} failed: {}",
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
// AWS SNS Tool
// ============================================================================

/// SNS operations
pub struct AwsSnsTool {
    config: ToolConfig,
}

impl AwsSnsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "SNS subcommand",
                    "enum": [
                        "list-topics", "list-subscriptions", "publish",
                        "create-topic", "subscribe"
                    ]
                },
                "topic_arn": {
                    "type": "string",
                    "description": "SNS topic ARN"
                },
                "message": {
                    "type": "string",
                    "description": "Message to publish"
                },
                "subject": {
                    "type": "string",
                    "description": "Message subject"
                },
                "protocol": {
                    "type": "string",
                    "description": "Subscription protocol (email, sms, https, etc.)"
                },
                "endpoint": {
                    "type": "string",
                    "description": "Subscription endpoint"
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
                "aws_sns",
                "AWS SNS: manage topics, subscriptions, and publish messages.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for AwsSnsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsSnsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let topic_arn: Option<String> = input.get_arg("topic_arn").ok();
        let message: Option<String> = input.get_arg("message").ok();
        let subject: Option<String> = input.get_arg("subject").ok();
        let protocol: Option<String> = input.get_arg("protocol").ok();
        let endpoint: Option<String> = input.get_arg("endpoint").ok();
        let region: Option<String> = input.get_arg("region").ok();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["sns".to_string(), command.clone()];

        if let Some(ref ta) = topic_arn {
            args.push("--topic-arn".to_string());
            args.push(ta.clone());
        }

        if let Some(ref m) = message {
            args.push("--message".to_string());
            args.push(m.clone());
        }

        if let Some(ref s) = subject {
            args.push("--subject".to_string());
            args.push(s.clone());
        }

        if let Some(ref prot) = protocol {
            args.push("--protocol".to_string());
            args.push(prot.clone());
        }

        if let Some(ref ep) = endpoint {
            args.push("--endpoint".to_string());
            args.push(ep.clone());
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

        debug!(args = ?args, "Executing aws sns");

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
                        "aws sns {} failed: {}",
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
// AWS Cost Explorer Tool
// ============================================================================

/// Cost Explorer operations
pub struct AwsCostTool {
    config: ToolConfig,
}

impl AwsCostTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "Cost Explorer subcommand",
                    "enum": ["get-cost-and-usage", "get-cost-forecast"]
                },
                "time_period_start": {
                    "type": "string",
                    "description": "Start date (YYYY-MM-DD format)"
                },
                "time_period_end": {
                    "type": "string",
                    "description": "End date (YYYY-MM-DD format)"
                },
                "granularity": {
                    "type": "string",
                    "description": "Time granularity",
                    "enum": ["DAILY", "MONTHLY", "HOURLY"],
                    "default": "MONTHLY"
                },
                "metrics": {
                    "type": "array",
                    "description": "Cost metrics (UnblendedCost, BlendedCost, UsageQuantity)",
                    "items": { "type": "string" },
                    "default": ["UnblendedCost"]
                },
                "group_by": {
                    "type": "array",
                    "description": "Group by dimensions (SERVICE, REGION, etc.)",
                    "items": { "type": "string" }
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
                "aws_cost",
                "AWS Cost Explorer: analyze and forecast AWS spending.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for AwsCostTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AwsCostTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let command: String = input.get_arg("command")?;
        let time_period_start: Option<String> = input.get_arg("time_period_start").ok();
        let time_period_end: Option<String> = input.get_arg("time_period_end").ok();
        let granularity: String = input.get_arg("granularity").unwrap_or_else(|_| "MONTHLY".to_string());
        let metrics: Vec<String> = input.get_arg("metrics").unwrap_or_else(|_| vec!["UnblendedCost".to_string()]);
        let group_by: Vec<String> = input.get_arg("group_by").unwrap_or_default();
        let profile: Option<String> = input.get_arg("profile").ok();

        let mut args = vec!["ce".to_string(), command.clone()];

        // Build time period JSON
        if let (Some(ref start), Some(ref end)) = (&time_period_start, &time_period_end) {
            let time_period = format!(r#"{{"Start":"{}","End":"{}"}}"#, start, end);
            args.push("--time-period".to_string());
            args.push(time_period);
        }

        args.push("--granularity".to_string());
        args.push(granularity);

        // Build metrics JSON array
        let metrics_json = serde_json::to_string(&metrics).unwrap();
        args.push("--metrics".to_string());
        args.push(metrics_json);

        // Build group-by JSON if provided
        if !group_by.is_empty() {
            let group_by_json: Vec<_> = group_by
                .iter()
                .map(|dim| format!(r#"{{"Type":"DIMENSION","Key":"{}"}}"#, dim))
                .collect();
            let group_by_str = format!("[{}]", group_by_json.join(","));
            args.push("--group-by".to_string());
            args.push(group_by_str);
        }

        if let Some(ref p) = profile {
            args.push("--profile".to_string());
            args.push(p.clone());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        debug!(args = ?args, "Executing aws ce");

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
                        "aws ce {} failed: {}",
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
