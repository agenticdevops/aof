//! Snyk Security Tools
//!
//! Tools for vulnerability scanning, dependency analysis, and security monitoring via Snyk's API.
//!
//! ## Available Tools
//!
//! - `snyk_test` - Test a project for vulnerabilities
//! - `snyk_monitor` - Monitor a project for new vulnerabilities
//! - `snyk_container_test` - Scan a container image for vulnerabilities
//! - `snyk_issues_list` - List issues for a project or organization
//! - `snyk_issue_ignore` - Ignore a vulnerability with reason
//! - `snyk_fix_pr` - Create a fix pull request
//!
//! ## Prerequisites
//!
//! - Requires `security` feature flag
//! - Valid Snyk API token (https://snyk.io)
//! - Snyk CLI installed for some operations
//!
//! ## Authentication
//!
//! All tools use token authentication via the `api_token` parameter or `SNYK_TOKEN` environment variable.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, execute_command, tool_config_with_timeout};

const SNYK_API_BASE: &str = "https://api.snyk.io/v1";

/// Collection of all Snyk tools
pub struct SnykTools;

impl SnykTools {
    /// Get all Snyk tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(SnykTestTool::new()),
            Box::new(SnykMonitorTool::new()),
            Box::new(SnykContainerTestTool::new()),
            Box::new(SnykIssuesListTool::new()),
            Box::new(SnykIssueIgnoreTool::new()),
            Box::new(SnykFixPrTool::new()),
        ]
    }
}

/// Get Snyk API token from input or environment
fn get_api_token(input: &ToolInput) -> AofResult<String> {
    input
        .get_arg::<String>("api_token")
        .or_else(|_| {
            std::env::var("SNYK_TOKEN")
                .map_err(|_| aof_core::AofError::tool("Missing api_token parameter and SNYK_TOKEN environment variable not set"))
        })
}

/// Create Snyk HTTP client with authentication
fn create_snyk_client(token: &str) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Token authentication
    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("token {}", token))
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle common Snyk error responses
fn handle_snyk_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let error_msg = body
        .get("message")
        .and_then(|m| m.as_str())
        .or_else(|| body.get("error").and_then(|e| e.as_str()))
        .unwrap_or("Unknown error");

    match status {
        400 => ToolResult::error(format!("Bad request: {}", error_msg)),
        401 => ToolResult::error("Authentication failed: Invalid or expired API token".to_string()),
        403 => ToolResult::error(format!("Permission denied: {}", error_msg)),
        404 => ToolResult::error("Resource not found or path does not exist".to_string()),
        429 => ToolResult::error("Rate limited. Please retry after a delay.".to_string()),
        500..=599 => ToolResult::error(format!("Snyk server error ({}): {}", status, error_msg)),
        _ => ToolResult::error(format!("Snyk returned status {}: {}", status, error_msg)),
    }
}

// ============================================================================
// Snyk Test Tool
// ============================================================================

/// Test a project for vulnerabilities
pub struct SnykTestTool {
    config: ToolConfig,
}

impl SnykTestTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Project directory path to scan"
                },
                "project_type": {
                    "type": "string",
                    "description": "Package manager type",
                    "enum": ["npm", "pip", "maven", "gradle", "go", "cargo", "yarn", "pnpm"]
                },
                "severity_threshold": {
                    "type": "string",
                    "description": "Fail threshold for severity",
                    "enum": ["low", "medium", "high", "critical"]
                },
                "all_projects": {
                    "type": "boolean",
                    "description": "Scan all projects in directory",
                    "default": false
                },
                "json_output": {
                    "type": "boolean",
                    "description": "Return JSON output",
                    "default": true
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_test",
                "Test a project for vulnerabilities using Snyk CLI. Reports security issues in dependencies.",
                parameters,
                120, // Extended timeout for scanning
            ),
        }
    }
}

impl Default for SnykTestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykTestTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let project_type: Option<String> = input.get_arg("project_type").ok();
        let severity_threshold: Option<String> = input.get_arg("severity_threshold").ok();
        let all_projects: bool = input.get_arg("all_projects").unwrap_or(false);
        let json_output: bool = input.get_arg("json_output").unwrap_or(true);

        // Ensure token is set (via parameter or env var)
        let token = get_api_token(&input)?;
        std::env::set_var("SNYK_TOKEN", &token);

        debug!(path = %path, "Testing project with Snyk");

        let mut args = vec!["test"];

        if json_output {
            args.push("--json");
        }

        if all_projects {
            args.push("--all-projects");
        }

        if let Some(threshold) = &severity_threshold {
            args.push("--severity-threshold");
            args.push(threshold);
        }

        if let Some(ptype) = &project_type {
            match ptype.as_str() {
                "npm" => args.push("--package-manager=npm"),
                "pip" => args.push("--package-manager=pip"),
                "maven" => args.push("--package-manager=maven"),
                "gradle" => args.push("--package-manager=gradle"),
                "go" => args.push("--package-manager=golang"),
                "cargo" => args.push("--package-manager=cargo"),
                "yarn" => args.push("--package-manager=yarn"),
                "pnpm" => args.push("--package-manager=pnpm"),
                _ => {}
            }
        }

        args.push(&path);

        let result = execute_command("snyk", &args, None, 120).await;

        match result {
            Ok(output) => {
                if json_output && !output.stdout.is_empty() {
                    // Parse JSON output
                    match serde_json::from_str::<serde_json::Value>(&output.stdout) {
                        Ok(json) => {
                            let vulnerabilities = json.get("vulnerabilities")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);

                            let summary = json.get("summary").cloned().unwrap_or(serde_json::json!({}));

                            Ok(ToolResult::success(serde_json::json!({
                                "success": output.exit_code == 0,
                                "ok": output.exit_code == 0,
                                "vulnerabilities_found": vulnerabilities,
                                "summary": summary,
                                "vulnerabilities": json.get("vulnerabilities"),
                                "path": path
                            })))
                        }
                        Err(_) => {
                            // Fallback to raw output
                            Ok(ToolResult::success(serde_json::json!({
                                "success": output.exit_code == 0,
                                "output": output.stdout,
                                "path": path
                            })))
                        }
                    }
                } else {
                    Ok(ToolResult::success(serde_json::json!({
                        "success": output.exit_code == 0,
                        "output": output.stdout,
                        "path": path
                    })))
                }
            }
            Err(e) => {
                if e.contains("not found") || e.contains("No such file") {
                    Ok(ToolResult::error("Snyk CLI not found. Please install from https://snyk.io/install".to_string()))
                } else {
                    Ok(ToolResult::error(format!("Snyk test failed: {}", e)))
                }
            }
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Snyk Monitor Tool
// ============================================================================

/// Monitor a project for new vulnerabilities
pub struct SnykMonitorTool {
    config: ToolConfig,
}

impl SnykMonitorTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Project directory path to monitor"
                },
                "project_name": {
                    "type": "string",
                    "description": "Custom project name in Snyk"
                },
                "org_id": {
                    "type": "string",
                    "description": "Snyk organization ID"
                },
                "target_reference": {
                    "type": "string",
                    "description": "Branch or version reference"
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_monitor",
                "Monitor a project for new vulnerabilities. Creates a snapshot in Snyk for continuous monitoring.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for SnykMonitorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykMonitorTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let project_name: Option<String> = input.get_arg("project_name").ok();
        let org_id: Option<String> = input.get_arg("org_id").ok();
        let target_reference: Option<String> = input.get_arg("target_reference").ok();

        let token = get_api_token(&input)?;
        std::env::set_var("SNYK_TOKEN", &token);

        debug!(path = %path, "Monitoring project with Snyk");

        let mut args = vec!["monitor", "--json"];

        if let Some(name) = &project_name {
            args.push("--project-name");
            args.push(name);
        }

        if let Some(org) = &org_id {
            args.push("--org");
            args.push(org);
        }

        if let Some(target_ref) = &target_reference {
            args.push("--target-reference");
            args.push(target_ref);
        }

        args.push(&path);

        let result = execute_command("snyk", &args, None, 120).await;

        match result {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    match serde_json::from_str::<serde_json::Value>(&output.stdout) {
                        Ok(json) => {
                            Ok(ToolResult::success(serde_json::json!({
                                "success": true,
                                "project_id": json.get("id"),
                                "project_url": json.get("uri"),
                                "issues": json.get("issuesToNotify"),
                                "path": path
                            })))
                        }
                        Err(_) => {
                            Ok(ToolResult::success(serde_json::json!({
                                "success": true,
                                "output": output.stdout,
                                "path": path
                            })))
                        }
                    }
                } else {
                    Ok(ToolResult::error(format!("Monitor failed: {}", output.stderr)))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("Snyk monitor failed: {}", e))),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Snyk Container Test Tool
// ============================================================================

/// Scan a container image for vulnerabilities
pub struct SnykContainerTestTool {
    config: ToolConfig,
}

impl SnykContainerTestTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "image": {
                    "type": "string",
                    "description": "Image reference (e.g., nginx:1.25, myregistry.io/app:v1.0)"
                },
                "dockerfile": {
                    "type": "string",
                    "description": "Path to Dockerfile for better analysis"
                },
                "platform": {
                    "type": "string",
                    "description": "Platform override",
                    "enum": ["linux/amd64", "linux/arm64", "linux/arm/v7"]
                },
                "severity_threshold": {
                    "type": "string",
                    "description": "Fail threshold for severity",
                    "enum": ["low", "medium", "high", "critical"]
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["image"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_container_test",
                "Scan a container image for vulnerabilities including OS packages and application dependencies.",
                parameters,
                180, // Extended timeout for container scanning
            ),
        }
    }
}

impl Default for SnykContainerTestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykContainerTestTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let image: String = input.get_arg("image")?;
        let dockerfile: Option<String> = input.get_arg("dockerfile").ok();
        let platform: Option<String> = input.get_arg("platform").ok();
        let severity_threshold: Option<String> = input.get_arg("severity_threshold").ok();

        let token = get_api_token(&input)?;
        std::env::set_var("SNYK_TOKEN", &token);

        debug!(image = %image, "Scanning container image with Snyk");

        let mut args = vec!["container", "test", "--json"];

        if let Some(df) = &dockerfile {
            args.push("--file");
            args.push(df);
        }

        if let Some(plat) = &platform {
            args.push("--platform");
            args.push(plat);
        }

        if let Some(threshold) = &severity_threshold {
            args.push("--severity-threshold");
            args.push(threshold);
        }

        args.push(&image);

        let result = execute_command("snyk", &args, None, 180).await;

        match result {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    match serde_json::from_str::<serde_json::Value>(&output.stdout) {
                        Ok(json) => {
                            let vulnerabilities = json.get("vulnerabilities")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);

                            Ok(ToolResult::success(serde_json::json!({
                                "success": output.exit_code == 0,
                                "image": image,
                                "base_image": json.get("docker").and_then(|d| d.get("baseImage")),
                                "base_image_recommendation": json.get("docker").and_then(|d| d.get("baseImageRemediation")),
                                "vulnerabilities": json.get("vulnerabilities"),
                                "vulnerabilities_count": vulnerabilities,
                                "summary": json.get("summary")
                            })))
                        }
                        Err(_) => {
                            Ok(ToolResult::success(serde_json::json!({
                                "success": output.exit_code == 0,
                                "output": output.stdout,
                                "image": image
                            })))
                        }
                    }
                } else {
                    Ok(ToolResult::error(format!("Container test failed: {}", output.stderr)))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("Snyk container test failed: {}", e))),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Snyk Issues List Tool
// ============================================================================

/// List issues for a project or organization
pub struct SnykIssuesListTool {
    config: ToolConfig,
}

impl SnykIssuesListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "org_id": {
                    "type": "string",
                    "description": "Snyk organization ID"
                },
                "project_id": {
                    "type": "string",
                    "description": "Filter to specific project ID"
                },
                "severity": {
                    "type": "string",
                    "description": "Filter by severity",
                    "enum": ["low", "medium", "high", "critical"]
                },
                "type": {
                    "type": "string",
                    "description": "Issue type filter",
                    "enum": ["vuln", "license"]
                },
                "ignored": {
                    "type": "boolean",
                    "description": "Include ignored issues",
                    "default": false
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results",
                    "default": 100,
                    "minimum": 1,
                    "maximum": 1000
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["org_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_issues_list",
                "List security issues for a project or organization. Retrieve and filter vulnerabilities for reporting and triage.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for SnykIssuesListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykIssuesListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let org_id: String = input.get_arg("org_id")?;
        let project_id: Option<String> = input.get_arg("project_id").ok();
        let severity: Option<String> = input.get_arg("severity").ok();
        let issue_type: Option<String> = input.get_arg("type").ok();
        let ignored: bool = input.get_arg("ignored").unwrap_or(false);
        let limit: i32 = input.get_arg("limit").unwrap_or(100);

        let token = get_api_token(&input)?;

        debug!(org_id = %org_id, "Listing Snyk issues");

        let client = create_snyk_client(&token)?;

        let mut url = format!("{}/org/{}/issues", SNYK_API_BASE, org_id);
        let mut query_params = vec![];

        if let Some(pid) = &project_id {
            query_params.push(format!("projectId={}", pid));
        }

        if let Some(sev) = &severity {
            query_params.push(format!("severity={}", sev));
        }

        if let Some(itype) = &issue_type {
            query_params.push(format!("type={}", itype));
        }

        if ignored {
            query_params.push("ignored=true".to_string());
        }

        query_params.push(format!("perPage={}", limit));

        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check Snyk API availability.",
                        e
                    )));
                } else {
                    return Ok(ToolResult::error(format!("Snyk API request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status != 200 {
            return Ok(handle_snyk_error(status, &body));
        }

        let issues = body.get("results").cloned().unwrap_or(serde_json::json!([]));
        let total = body.get("total").and_then(|t| t.as_i64()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "issues": issues,
            "total": total,
            "org_id": org_id
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Snyk Issue Ignore Tool
// ============================================================================

/// Ignore a vulnerability with reason
pub struct SnykIssueIgnoreTool {
    config: ToolConfig,
}

impl SnykIssueIgnoreTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "org_id": {
                    "type": "string",
                    "description": "Snyk organization ID"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID"
                },
                "issue_id": {
                    "type": "string",
                    "description": "Issue ID to ignore"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for ignoring",
                    "enum": ["not-vulnerable", "wont-fix", "temporary-ignore"]
                },
                "reason_text": {
                    "type": "string",
                    "description": "Detailed explanation for ignoring"
                },
                "expires": {
                    "type": "string",
                    "description": "Expiration date for ignore rule (ISO 8601 format)"
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["org_id", "project_id", "issue_id", "reason", "reason_text"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_issue_ignore",
                "Ignore a vulnerability with a specific reason. Mark false positives or accepted risks.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SnykIssueIgnoreTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykIssueIgnoreTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let org_id: String = input.get_arg("org_id")?;
        let project_id: String = input.get_arg("project_id")?;
        let issue_id: String = input.get_arg("issue_id")?;
        let reason: String = input.get_arg("reason")?;
        let reason_text: String = input.get_arg("reason_text")?;
        let expires: Option<String> = input.get_arg("expires").ok();

        let token = get_api_token(&input)?;

        debug!(org_id = %org_id, project_id = %project_id, issue_id = %issue_id, "Ignoring Snyk issue");

        let client = create_snyk_client(&token)?;

        let url = format!(
            "{}/org/{}/project/{}/ignore/{}",
            SNYK_API_BASE, org_id, project_id, issue_id
        );

        let mut payload = serde_json::json!({
            "reason": reason,
            "reasonText": reason_text
        });

        if let Some(exp) = expires {
            payload["expires"] = serde_json::json!(exp);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Snyk API request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status != 200 && status != 201 {
            return Ok(handle_snyk_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "ignored": true,
            "issue_id": issue_id,
            "expires": payload.get("expires")
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Snyk Fix PR Tool
// ============================================================================

/// Create a fix pull request
pub struct SnykFixPrTool {
    config: ToolConfig,
}

impl SnykFixPrTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "org_id": {
                    "type": "string",
                    "description": "Snyk organization ID"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID"
                },
                "issue_id": {
                    "type": "string",
                    "description": "Specific issue to fix (optional, fixes all if not specified)"
                },
                "api_token": {
                    "type": "string",
                    "description": "Snyk API token (or set SNYK_TOKEN env var)"
                }
            }),
            vec!["org_id", "project_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "snyk_fix_pr",
                "Create an automated fix pull request for vulnerabilities. Generate PRs with dependency upgrades.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for SnykFixPrTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SnykFixPrTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let org_id: String = input.get_arg("org_id")?;
        let project_id: String = input.get_arg("project_id")?;
        let issue_id: Option<String> = input.get_arg("issue_id").ok();

        let token = get_api_token(&input)?;

        debug!(org_id = %org_id, project_id = %project_id, "Creating Snyk fix PR");

        let client = create_snyk_client(&token)?;

        let url = format!(
            "{}/org/{}/project/{}/pr",
            SNYK_API_BASE, org_id, project_id
        );

        let mut payload = serde_json::json!({});

        if let Some(iid) = issue_id {
            payload["issueId"] = serde_json::json!(iid);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Snyk API request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status != 200 && status != 201 {
            return Ok(handle_snyk_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "pull_request": {
                "url": body.get("data").and_then(|d| d.get("url")),
                "fixes": body.get("data").and_then(|d| d.get("fixes"))
            }
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
    fn test_snyk_tools_creation() {
        let tools = SnykTools::all();
        assert_eq!(tools.len(), 6);

        let names: Vec<&str> = tools.iter().map(|t| t.config().name.as_str()).collect();
        assert!(names.contains(&"snyk_test"));
        assert!(names.contains(&"snyk_monitor"));
        assert!(names.contains(&"snyk_container_test"));
        assert!(names.contains(&"snyk_issues_list"));
        assert!(names.contains(&"snyk_issue_ignore"));
        assert!(names.contains(&"snyk_fix_pr"));
    }

    #[test]
    fn test_snyk_test_config() {
        let tool = SnykTestTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_test");
        assert!(config.description.contains("vulnerabilities"));
        assert_eq!(config.timeout_secs, 120);
    }

    #[test]
    fn test_snyk_container_test_config() {
        let tool = SnykContainerTestTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_container_test");
        assert!(config.description.contains("container"));
        assert_eq!(config.timeout_secs, 180);
    }

    #[test]
    fn test_snyk_issues_list_config() {
        let tool = SnykIssuesListTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_issues_list");
        assert!(config.description.contains("issues"));
    }

    #[test]
    fn test_snyk_issue_ignore_config() {
        let tool = SnykIssueIgnoreTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_issue_ignore");
        assert!(config.description.contains("Ignore"));
    }

    #[test]
    fn test_snyk_fix_pr_config() {
        let tool = SnykFixPrTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_fix_pr");
        assert!(config.description.contains("pull request"));
    }

    #[test]
    fn test_snyk_monitor_config() {
        let tool = SnykMonitorTool::new();
        let config = tool.config();

        assert_eq!(config.name, "snyk_monitor");
        assert!(config.description.contains("Monitor"));
    }
}
