//! GitLab CI/CD Tools
//!
//! Tools for interacting with GitLab's built-in CI/CD system via REST API.
//!
//! ## Available Tools
//!
//! - `gitlab_pipeline_list` - List pipelines for a project
//! - `gitlab_pipeline_get` - Get detailed pipeline information
//! - `gitlab_pipeline_create` - Trigger a new pipeline
//! - `gitlab_pipeline_cancel` - Cancel a running pipeline
//! - `gitlab_pipeline_retry` - Retry a failed pipeline
//! - `gitlab_job_list` - List jobs in a pipeline
//! - `gitlab_job_get` - Get job details
//! - `gitlab_job_log` - Retrieve job execution logs
//!
//! ## Prerequisites
//!
//! - Requires `cicd` feature flag
//! - Valid GitLab Personal Access Token or OAuth2 token
//! - Token scopes: `api` or `read_api` (for read operations)
//!
//! ## Authentication
//!
//! All tools use Bearer token authentication.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all GitLab CI tools
pub struct GitLabCITools;

impl GitLabCITools {
    /// Get all GitLab CI tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GitLabPipelineListTool::new()),
            Box::new(GitLabPipelineGetTool::new()),
            Box::new(GitLabPipelineCreateTool::new()),
            Box::new(GitLabPipelineCancelTool::new()),
            Box::new(GitLabPipelineRetryTool::new()),
            Box::new(GitLabJobListTool::new()),
            Box::new(GitLabJobGetTool::new()),
            Box::new(GitLabJobLogTool::new()),
        ]
    }
}

/// URL-encode project ID (handles paths like "group/project")
fn encode_project_id(project_id: &str) -> String {
    project_id.replace('/', "%2F")
}

/// Build GitLab API URL for a project resource
fn build_project_url(endpoint: &str, project_id: &str, path: &str) -> String {
    let encoded_id = encode_project_id(project_id);
    format!(
        "{}/api/v4/projects/{}/{}",
        endpoint.trim_end_matches('/'),
        encoded_id,
        path.trim_start_matches('/')
    )
}

/// Create GitLab API HTTP client with authentication
fn create_gitlab_client(token: &str) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Bearer token authentication (works for both PAT and OAuth2)
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    // Content type for JSON requests
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    // Accept JSON responses
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    // User agent
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("aof-gitlab-ci-tool"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle GitLab API error responses
fn handle_gitlab_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let message = body
        .get("message")
        .and_then(|m| m.as_str())
        .or_else(|| body.get("error").and_then(|e| e.as_str()))
        .unwrap_or("Unknown error");

    match status {
        401 => ToolResult::error(
            "Authentication failed. Check token validity and scopes (api, read_api required)."
                .to_string(),
        ),
        403 => ToolResult::error(format!(
            "Permission denied: {}. Check token scopes and project permissions.",
            message
        )),
        404 => ToolResult::error(
            "Resource not found. Check project_id (use numeric ID or URL-encoded path) and resource ID."
                .to_string(),
        ),
        409 => ToolResult::error(format!(
            "Conflict: {}. Resource may not be in valid state for this operation.",
            message
        )),
        422 => ToolResult::error(format!(
            "Validation error: {}. Check that branch/tag/commit exists.",
            message
        )),
        429 => {
            ToolResult::error("Rate limit exceeded. Retry after cooldown period.".to_string())
        }
        500..=599 => ToolResult::error(format!("GitLab server error ({}): {}", status, message)),
        _ => ToolResult::error(format!("GitLab API error ({}): {}", status, message)),
    }
}

/// Validate endpoint URL
fn validate_endpoint(endpoint: &str) -> Result<(), aof_core::AofError> {
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(aof_core::AofError::tool(
            "Endpoint must start with http:// or https://".to_string(),
        ));
    }
    Ok(())
}

// ============================================================================
// GitLab Pipeline List Tool
// ============================================================================

/// List pipelines for a project with optional filtering
pub struct GitLabPipelineListTool {
    config: ToolConfig,
}

impl GitLabPipelineListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL (e.g., https://gitlab.com or https://gitlab.example.com)"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path (e.g., '123' or 'group/project')"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                },
                "scope": {
                    "type": "string",
                    "description": "Filter by scope",
                    "enum": ["running", "pending", "finished", "branches", "tags"]
                },
                "status": {
                    "type": "string",
                    "description": "Filter by status",
                    "enum": ["created", "waiting_for_resource", "preparing", "pending", "running", "success", "failed", "canceled", "skipped", "manual", "scheduled"]
                },
                "ref": {
                    "type": "string",
                    "description": "Filter by branch or tag name"
                },
                "sha": {
                    "type": "string",
                    "description": "Filter by commit SHA"
                },
                "username": {
                    "type": "string",
                    "description": "Filter by user who triggered pipeline"
                },
                "updated_after": {
                    "type": "string",
                    "description": "Return pipelines updated after this time (ISO 8601 format)"
                },
                "updated_before": {
                    "type": "string",
                    "description": "Return pipelines updated before this time (ISO 8601 format)"
                },
                "order_by": {
                    "type": "string",
                    "description": "Order results by field",
                    "enum": ["id", "status", "ref", "updated_at", "user_id"],
                    "default": "id"
                },
                "sort": {
                    "type": "string",
                    "description": "Sort direction",
                    "enum": ["asc", "desc"],
                    "default": "desc"
                },
                "per_page": {
                    "type": "integer",
                    "description": "Results per page (max 100)",
                    "default": 20,
                    "minimum": 1,
                    "maximum": 100
                },
                "page": {
                    "type": "integer",
                    "description": "Page number",
                    "default": 1,
                    "minimum": 1
                }
            }),
            vec!["endpoint", "project_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_pipeline_list",
                "List pipelines for a GitLab project with optional filtering by status, branch, or user.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabPipelineListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabPipelineListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        let scope: Option<String> = input.get_arg("scope").ok();
        let status: Option<String> = input.get_arg("status").ok();
        let ref_name: Option<String> = input.get_arg("ref").ok();
        let sha: Option<String> = input.get_arg("sha").ok();
        let username: Option<String> = input.get_arg("username").ok();
        let updated_after: Option<String> = input.get_arg("updated_after").ok();
        let updated_before: Option<String> = input.get_arg("updated_before").ok();
        let order_by: String = input.get_arg("order_by").unwrap_or_else(|_| "id".to_string());
        let sort: String = input.get_arg("sort").unwrap_or_else(|_| "desc".to_string());
        let per_page: i32 = input.get_arg("per_page").unwrap_or(20);
        let page: i32 = input.get_arg("page").unwrap_or(1);

        debug!(
            project_id = %project_id,
            "Listing GitLab pipelines"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(&endpoint, &project_id, "pipelines");

        // Build query parameters
        let mut params: Vec<(&str, String)> = vec![
            ("order_by", order_by),
            ("sort", sort),
            ("per_page", per_page.to_string()),
            ("page", page.to_string()),
        ];

        if let Some(s) = scope {
            params.push(("scope", s));
        }
        if let Some(s) = status {
            params.push(("status", s));
        }
        if let Some(r) = ref_name {
            params.push(("ref", r));
        }
        if let Some(s) = sha {
            params.push(("sha", s));
        }
        if let Some(u) = username {
            params.push(("username", u));
        }
        if let Some(after) = updated_after {
            params.push(("updated_after", after));
        }
        if let Some(before) = updated_before {
            params.push(("updated_before", before));
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error(
                        "Request timeout. GitLab may be slow or unreachable.".to_string(),
                    ));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check endpoint URL and network connectivity.",
                        e
                    )));
                }
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 200 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        // Parse pipeline list
        let pipelines = body.as_array().cloned().unwrap_or_default();
        let count = pipelines.len();

        // Calculate total duration for successful pipelines
        let total_duration: i64 = pipelines
            .iter()
            .filter_map(|p| p.get("duration").and_then(|d| d.as_i64()))
            .sum();

        Ok(ToolResult::success(serde_json::json!({
            "pipelines": pipelines,
            "count": count,
            "total_duration_seconds": total_duration
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Pipeline Get Tool
// ============================================================================

/// Get detailed information about a specific pipeline
pub struct GitLabPipelineGetTool {
    config: ToolConfig,
}

impl GitLabPipelineGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "pipeline_id": {
                    "type": "string",
                    "description": "Pipeline ID"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                }
            }),
            vec!["endpoint", "project_id", "pipeline_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_pipeline_get",
                "Get detailed information about a specific GitLab pipeline including status, duration, and user info.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabPipelineGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabPipelineGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let pipeline_id: String = input.get_arg("pipeline_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            pipeline_id = %pipeline_id,
            "Getting GitLab pipeline details"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(&endpoint, &project_id, &format!("pipelines/{}", pipeline_id));

        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout.".to_string()));
                }
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 200 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        Ok(ToolResult::success(body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Pipeline Create Tool
// ============================================================================

/// Trigger a new pipeline for a project
pub struct GitLabPipelineCreateTool {
    config: ToolConfig,
}

impl GitLabPipelineCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                },
                "ref": {
                    "type": "string",
                    "description": "Branch, tag, or commit SHA to run pipeline for"
                },
                "variables": {
                    "type": "array",
                    "description": "CI/CD variables to pass to pipeline",
                    "items": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Variable name"
                            },
                            "value": {
                                "type": "string",
                                "description": "Variable value"
                            },
                            "variable_type": {
                                "type": "string",
                                "description": "Variable type",
                                "enum": ["env_var", "file"],
                                "default": "env_var"
                            }
                        },
                        "required": ["key", "value"]
                    }
                }
            }),
            vec!["endpoint", "project_id", "token", "ref"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_pipeline_create",
                "Trigger a new pipeline for a GitLab project. Optionally pass CI/CD variables.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabPipelineCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabPipelineCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let token: String = input.get_arg("token")?;
        let ref_name: String = input.get_arg("ref")?;
        let variables: Option<Vec<serde_json::Value>> = input.get_arg("variables").ok();

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            ref_name = %ref_name,
            "Creating GitLab pipeline"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(&endpoint, &project_id, "pipeline");

        // Build request body
        let mut payload = serde_json::json!({
            "ref": ref_name
        });

        if let Some(vars) = variables {
            payload["variables"] = serde_json::Value::Array(vars);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout.".to_string()));
                }
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 201 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        let pipeline_id = body.get("id").and_then(|id| id.as_i64()).unwrap_or(0);
        let status = body
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        let web_url = body
            .get("web_url")
            .and_then(|u| u.as_str())
            .unwrap_or("");

        Ok(ToolResult::success(serde_json::json!({
            "pipeline_id": pipeline_id,
            "status": status,
            "ref": ref_name,
            "sha": body.get("sha"),
            "web_url": web_url,
            "created_at": body.get("created_at")
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Pipeline Cancel Tool
// ============================================================================

/// Cancel a running or pending pipeline
pub struct GitLabPipelineCancelTool {
    config: ToolConfig,
}

impl GitLabPipelineCancelTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "pipeline_id": {
                    "type": "string",
                    "description": "Pipeline ID to cancel"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                }
            }),
            vec!["endpoint", "project_id", "pipeline_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_pipeline_cancel",
                "Cancel a running or pending GitLab pipeline.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabPipelineCancelTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabPipelineCancelTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let pipeline_id: String = input.get_arg("pipeline_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            pipeline_id = %pipeline_id,
            "Canceling GitLab pipeline"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(
            &endpoint,
            &project_id,
            &format!("pipelines/{}/cancel", pipeline_id),
        );

        let response = match client.post(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 200 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "pipeline_id": body.get("id"),
            "status": body.get("status"),
            "ref": body.get("ref"),
            "sha": body.get("sha"),
            "web_url": body.get("web_url"),
            "canceled_at": chrono::Utc::now().to_rfc3339()
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Pipeline Retry Tool
// ============================================================================

/// Retry a failed pipeline
pub struct GitLabPipelineRetryTool {
    config: ToolConfig,
}

impl GitLabPipelineRetryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "pipeline_id": {
                    "type": "string",
                    "description": "Pipeline ID to retry"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                }
            }),
            vec!["endpoint", "project_id", "pipeline_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_pipeline_retry",
                "Retry a failed GitLab pipeline. Creates a new pipeline with the same parameters.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabPipelineRetryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabPipelineRetryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let pipeline_id: String = input.get_arg("pipeline_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            pipeline_id = %pipeline_id,
            "Retrying GitLab pipeline"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(
            &endpoint,
            &project_id,
            &format!("pipelines/{}/retry", pipeline_id),
        );

        let response = match client.post(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 201 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "pipeline_id": body.get("id"),
            "status": body.get("status"),
            "ref": body.get("ref"),
            "sha": body.get("sha"),
            "web_url": body.get("web_url"),
            "created_at": body.get("created_at"),
            "retried_from": pipeline_id
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Job List Tool
// ============================================================================

/// List jobs in a pipeline
pub struct GitLabJobListTool {
    config: ToolConfig,
}

impl GitLabJobListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "pipeline_id": {
                    "type": "string",
                    "description": "Pipeline ID"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                },
                "scope": {
                    "type": "array",
                    "description": "Filter by job status",
                    "items": {
                        "type": "string",
                        "enum": ["created", "pending", "running", "failed", "success", "canceled", "skipped", "manual"]
                    }
                },
                "include_retried": {
                    "type": "boolean",
                    "description": "Include retried jobs",
                    "default": false
                },
                "per_page": {
                    "type": "integer",
                    "description": "Results per page (max 100)",
                    "default": 20
                },
                "page": {
                    "type": "integer",
                    "description": "Page number",
                    "default": 1
                }
            }),
            vec!["endpoint", "project_id", "pipeline_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_job_list",
                "List all jobs in a GitLab pipeline with optional status filtering.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabJobListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabJobListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let pipeline_id: String = input.get_arg("pipeline_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        let scope: Option<Vec<String>> = input.get_arg("scope").ok();
        let include_retried: bool = input.get_arg("include_retried").unwrap_or(false);
        let per_page: i32 = input.get_arg("per_page").unwrap_or(20);
        let page: i32 = input.get_arg("page").unwrap_or(1);

        debug!(
            project_id = %project_id,
            pipeline_id = %pipeline_id,
            "Listing GitLab pipeline jobs"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(
            &endpoint,
            &project_id,
            &format!("pipelines/{}/jobs", pipeline_id),
        );

        let mut params: Vec<(&str, String)> = vec![
            ("per_page", per_page.to_string()),
            ("page", page.to_string()),
        ];

        if include_retried {
            params.push(("include_retried", "true".to_string()));
        }

        // Build scope query parameters
        let scope_params: Vec<String>;
        if let Some(scopes) = &scope {
            scope_params = scopes.clone();
            for s in &scope_params {
                params.push(("scope[]", s.clone()));
            }
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 200 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        let jobs = body.as_array().cloned().unwrap_or_default();
        let count = jobs.len();

        Ok(ToolResult::success(serde_json::json!({
            "jobs": jobs,
            "count": count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Job Get Tool
// ============================================================================

/// Get detailed information about a specific job
pub struct GitLabJobGetTool {
    config: ToolConfig,
}

impl GitLabJobGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "job_id": {
                    "type": "string",
                    "description": "Job ID"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                }
            }),
            vec!["endpoint", "project_id", "job_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_job_get",
                "Get detailed information about a specific GitLab CI job.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitLabJobGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabJobGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let job_id: String = input.get_arg("job_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            job_id = %job_id,
            "Getting GitLab job details"
        );

        let client = create_gitlab_client(&token)?;
        let url = build_project_url(&endpoint, &project_id, &format!("jobs/{}", job_id));

        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"message": "Failed to parse response"}));

        if status_code != 200 {
            return Ok(handle_gitlab_error(status_code, &body));
        }

        Ok(ToolResult::success(body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitLab Job Log Tool
// ============================================================================

/// Retrieve job execution logs
pub struct GitLabJobLogTool {
    config: ToolConfig,
}

impl GitLabJobLogTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "GitLab instance URL"
                },
                "project_id": {
                    "type": "string",
                    "description": "Project ID or path"
                },
                "job_id": {
                    "type": "string",
                    "description": "Job ID"
                },
                "token": {
                    "type": "string",
                    "description": "GitLab private token or OAuth2 token"
                }
            }),
            vec!["endpoint", "project_id", "job_id", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "gitlab_job_log",
                "Retrieve execution logs for a GitLab CI job. Useful for debugging failures.",
                parameters,
                120, // Longer timeout for potentially large logs
            ),
        }
    }
}

impl Default for GitLabJobLogTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLabJobLogTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_id: String = input.get_arg("project_id")?;
        let job_id: String = input.get_arg("job_id")?;
        let token: String = input.get_arg("token")?;

        validate_endpoint(&endpoint)?;

        debug!(
            project_id = %project_id,
            job_id = %job_id,
            "Getting GitLab job logs"
        );

        let client = create_gitlab_client(&token)?;

        // First get job details for name and status
        let job_url = build_project_url(&endpoint, &project_id, &format!("jobs/{}", job_id));
        let job_response = client.get(&job_url).send().await;

        let (job_name, job_status) = if let Ok(resp) = job_response {
            if let Ok(job_body) = resp.json::<serde_json::Value>().await {
                (
                    job_body
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    job_body
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                )
            } else {
                ("unknown".to_string(), "unknown".to_string())
            }
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };

        // Get job trace (logs)
        let log_url = build_project_url(&endpoint, &project_id, &format!("jobs/{}/trace", job_id));

        // Override accept header for text response
        let response = match client
            .get(&log_url)
            .header(reqwest::header::ACCEPT, "text/plain")
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(ToolResult::error(format!("Request failed: {}", e)));
            }
        };

        let status_code = response.status().as_u16();

        if status_code != 200 {
            let body: serde_json::Value = response
                .json()
                .await
                .unwrap_or_else(|_| serde_json::json!({"message": "Failed to get logs"}));
            return Ok(handle_gitlab_error(status_code, &body));
        }

        let log_text = response.text().await.unwrap_or_default();
        let log_lines = log_text.lines().count();
        let truncated = log_lines > 10000;

        Ok(ToolResult::success(serde_json::json!({
            "job_id": job_id,
            "job_name": job_name,
            "status": job_status,
            "log": if truncated {
                log_text.lines().take(10000).collect::<Vec<_>>().join("\n")
            } else {
                log_text
            },
            "log_lines": log_lines,
            "truncated": truncated
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
    fn test_encode_project_id() {
        assert_eq!(encode_project_id("123"), "123");
        assert_eq!(encode_project_id("group/project"), "group%2Fproject");
        assert_eq!(
            encode_project_id("group/subgroup/project"),
            "group%2Fsubgroup%2Fproject"
        );
    }

    #[test]
    fn test_build_project_url() {
        let url = build_project_url("https://gitlab.com", "group/project", "pipelines");
        assert_eq!(
            url,
            "https://gitlab.com/api/v4/projects/group%2Fproject/pipelines"
        );

        let url2 = build_project_url("https://gitlab.com/", "123", "/pipelines/456");
        assert_eq!(url2, "https://gitlab.com/api/v4/projects/123/pipelines/456");
    }

    #[test]
    fn test_validate_endpoint() {
        assert!(validate_endpoint("https://gitlab.com").is_ok());
        assert!(validate_endpoint("http://gitlab.local").is_ok());
        assert!(validate_endpoint("gitlab.com").is_err());
        assert!(validate_endpoint("ftp://gitlab.com").is_err());
    }

    #[test]
    fn test_tool_configs() {
        let list_tool = GitLabPipelineListTool::new();
        assert_eq!(list_tool.config().name, "gitlab_pipeline_list");

        let get_tool = GitLabPipelineGetTool::new();
        assert_eq!(get_tool.config().name, "gitlab_pipeline_get");

        let create_tool = GitLabPipelineCreateTool::new();
        assert_eq!(create_tool.config().name, "gitlab_pipeline_create");

        let cancel_tool = GitLabPipelineCancelTool::new();
        assert_eq!(cancel_tool.config().name, "gitlab_pipeline_cancel");

        let retry_tool = GitLabPipelineRetryTool::new();
        assert_eq!(retry_tool.config().name, "gitlab_pipeline_retry");

        let job_list_tool = GitLabJobListTool::new();
        assert_eq!(job_list_tool.config().name, "gitlab_job_list");

        let job_get_tool = GitLabJobGetTool::new();
        assert_eq!(job_get_tool.config().name, "gitlab_job_get");

        let job_log_tool = GitLabJobLogTool::new();
        assert_eq!(job_log_tool.config().name, "gitlab_job_log");
    }

    #[test]
    fn test_all_tools() {
        let tools = GitLabCITools::all();
        assert_eq!(tools.len(), 8);
    }
}
