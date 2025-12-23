//! GitHub Actions Tools
//!
//! Tools for interacting with GitHub's CI/CD platform via REST API.
//!
//! ## Available Tools
//!
//! - `github_workflow_list` - List workflows in a repository
//! - `github_workflow_dispatch` - Trigger a workflow run
//! - `github_run_list` - List workflow runs
//! - `github_run_get` - Get workflow run details
//! - `github_run_cancel` - Cancel a running workflow
//! - `github_run_rerun` - Rerun a failed workflow
//! - `github_run_force_cancel` - Force cancel unresponsive workflow
//! - `github_artifacts_list` - List workflow artifacts
//! - `github_artifacts_download` - Download artifact
//! - `github_run_logs` - Download workflow logs
//!
//! ## Prerequisites
//!
//! - Requires `cicd` feature flag
//! - Valid GitHub Personal Access Token (PAT) with `repo` scope
//! - Repository access permissions
//!
//! ## Authentication
//!
//! All tools use Bearer token authentication via GitHub PAT.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all GitHub Actions tools
pub struct GitHubActionsTools;

impl GitHubActionsTools {
    /// Get all GitHub Actions tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GitHubWorkflowListTool::new()),
            Box::new(GitHubWorkflowDispatchTool::new()),
            Box::new(GitHubRunListTool::new()),
            Box::new(GitHubRunGetTool::new()),
            Box::new(GitHubRunCancelTool::new()),
            Box::new(GitHubRunRerunTool::new()),
            Box::new(GitHubRunForceCancelTool::new()),
            Box::new(GitHubArtifactsListTool::new()),
            Box::new(GitHubArtifactsDownloadTool::new()),
            Box::new(GitHubRunLogsTool::new()),
        ]
    }
}

/// Create GitHub API HTTP client with authentication
fn create_github_client(token: &str) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Bearer token authentication
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    // GitHub API version
    headers.insert(
        "X-GitHub-Api-Version",
        reqwest::header::HeaderValue::from_static("2022-11-28"),
    );

    // User agent (required by GitHub)
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("aof-github-actions-tool"),
    );

    // Accept header for JSON
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle GitHub API error responses
fn handle_github_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let message = body
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown error");

    match status {
        401 => ToolResult::error(
            "Authentication failed. Check token permissions.".to_string()
        ),
        403 => ToolResult::error(format!(
            "Permission denied: {}. Check token scopes or rate limits.",
            message
        )),
        404 => ToolResult::error(
            "Resource not found. Check owner, repo, and resource ID.".to_string()
        ),
        409 => ToolResult::error(
            "Conflict. Workflow may not be in valid state for this operation.".to_string()
        ),
        410 => ToolResult::error("Resource expired or deleted.".to_string()),
        422 => ToolResult::error(format!("Validation error: {}", message)),
        429 => ToolResult::error("Rate limited. Retry after cooldown period.".to_string()),
        500..=599 => ToolResult::error(format!("GitHub server error: {}", message)),
        _ => ToolResult::error(format!("GitHub API error ({}): {}", status, message)),
    }
}

// ============================================================================
// GitHub Workflow List Tool
// ============================================================================

/// List workflows in a repository
pub struct GitHubWorkflowListTool {
    config: ToolConfig,
}

impl GitHubWorkflowListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner (username or organization)"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "per_page": {
                    "type": "integer",
                    "description": "Results per page (max 100)",
                    "default": 30
                },
                "page": {
                    "type": "integer",
                    "description": "Page number",
                    "default": 1
                }
            }),
            vec!["token", "owner", "repo"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_workflow_list",
                "List all workflows in a GitHub repository. Returns workflow definitions and status.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubWorkflowListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubWorkflowListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let per_page: i32 = input.get_arg("per_page").unwrap_or(30);
        let page: i32 = input.get_arg("page").unwrap_or(1);

        debug!(owner = %owner, repo = %repo, "Listing GitHub workflows");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/workflows",
            owner, repo
        );

        let response = match client
            .get(&url)
            .query(&[("per_page", per_page), ("page", page)])
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!("Connection failed: {}", e)));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
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
            return Ok(handle_github_error(status, &body));
        }

        let workflows = body.get("workflows").cloned().unwrap_or(serde_json::json!([]));
        let total_count = body.get("total_count").and_then(|c| c.as_i64()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "workflows": workflows,
            "total_count": total_count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Workflow Dispatch Tool
// ============================================================================

/// Trigger a workflow run via dispatch event
pub struct GitHubWorkflowDispatchTool {
    config: ToolConfig,
}

impl GitHubWorkflowDispatchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token with repo scope"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "workflow_id": {
                    "type": "string",
                    "description": "Workflow ID or filename (e.g., 'deploy.yml')"
                },
                "ref": {
                    "type": "string",
                    "description": "Git branch, tag, or commit SHA"
                },
                "inputs": {
                    "type": "object",
                    "description": "Workflow input parameters (max 25)"
                }
            }),
            vec!["token", "owner", "repo", "workflow_id", "ref"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_workflow_dispatch",
                "Trigger a workflow run via dispatch event. Requires workflow_dispatch event in workflow.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubWorkflowDispatchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubWorkflowDispatchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let workflow_id: String = input.get_arg("workflow_id")?;
        let ref_name: String = input.get_arg("ref")?;
        let inputs: Option<serde_json::Value> = input.get_arg("inputs").ok();

        debug!(owner = %owner, repo = %repo, workflow_id = %workflow_id, "Dispatching GitHub workflow");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
            owner, repo, workflow_id
        );

        let mut payload = serde_json::json!({
            "ref": ref_name
        });

        if let Some(inp) = inputs {
            payload["inputs"] = inp;
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 204 No Content indicates success for dispatch
        if status == 204 {
            return Ok(ToolResult::success(serde_json::json!({
                "status": "dispatched",
                "workflow_id": workflow_id,
                "ref": ref_name,
                "message": "Workflow dispatch event created successfully"
            })));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run List Tool
// ============================================================================

/// List workflow runs for a repository
pub struct GitHubRunListTool {
    config: ToolConfig,
}

impl GitHubRunListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "workflow_id": {
                    "type": "string",
                    "description": "Filter by workflow ID or filename"
                },
                "actor": {
                    "type": "string",
                    "description": "Filter by user who triggered the run"
                },
                "branch": {
                    "type": "string",
                    "description": "Filter by branch name"
                },
                "event": {
                    "type": "string",
                    "description": "Filter by event type",
                    "enum": ["push", "pull_request", "workflow_dispatch", "schedule", "release"]
                },
                "status": {
                    "type": "string",
                    "description": "Filter by run status",
                    "enum": ["queued", "in_progress", "completed"]
                },
                "created": {
                    "type": "string",
                    "description": "Filter by creation date (ISO 8601 or range)"
                },
                "head_sha": {
                    "type": "string",
                    "description": "Filter by commit SHA"
                },
                "exclude_pull_requests": {
                    "type": "boolean",
                    "description": "Exclude pull request triggered runs",
                    "default": false
                },
                "per_page": {
                    "type": "integer",
                    "description": "Results per page (max 100)",
                    "default": 30
                },
                "page": {
                    "type": "integer",
                    "description": "Page number",
                    "default": 1
                }
            }),
            vec!["token", "owner", "repo"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_list",
                "List workflow runs for a repository. Supports filtering by workflow, status, branch, and more.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubRunListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let workflow_id: Option<String> = input.get_arg("workflow_id").ok();
        let actor: Option<String> = input.get_arg("actor").ok();
        let branch: Option<String> = input.get_arg("branch").ok();
        let event: Option<String> = input.get_arg("event").ok();
        let status: Option<String> = input.get_arg("status").ok();
        let created: Option<String> = input.get_arg("created").ok();
        let head_sha: Option<String> = input.get_arg("head_sha").ok();
        let exclude_pull_requests: bool = input.get_arg("exclude_pull_requests").unwrap_or(false);
        let per_page: i32 = input.get_arg("per_page").unwrap_or(30);
        let page: i32 = input.get_arg("page").unwrap_or(1);

        debug!(owner = %owner, repo = %repo, "Listing GitHub workflow runs");

        let client = create_github_client(&token)?;

        // Build URL based on whether workflow_id is specified
        let url = if let Some(wf_id) = &workflow_id {
            format!(
                "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                owner, repo, wf_id
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/actions/runs",
                owner, repo
            )
        };

        let mut params: Vec<(&str, String)> = vec![
            ("per_page", per_page.to_string()),
            ("page", page.to_string()),
        ];

        if let Some(a) = actor {
            params.push(("actor", a));
        }
        if let Some(b) = branch {
            params.push(("branch", b));
        }
        if let Some(e) = event {
            params.push(("event", e));
        }
        if let Some(s) = status {
            params.push(("status", s));
        }
        if let Some(c) = created {
            params.push(("created", c));
        }
        if let Some(sha) = head_sha {
            params.push(("head_sha", sha));
        }
        if exclude_pull_requests {
            params.push(("exclude_pull_requests", "true".to_string()));
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status_code = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status_code != 200 {
            return Ok(handle_github_error(status_code, &body));
        }

        let runs = body.get("workflow_runs").cloned().unwrap_or(serde_json::json!([]));
        let total_count = body.get("total_count").and_then(|c| c.as_i64()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "workflow_runs": runs,
            "total_count": total_count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run Get Tool
// ============================================================================

/// Get details of a specific workflow run
pub struct GitHubRunGetTool {
    config: ToolConfig,
}

impl GitHubRunGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Workflow run ID"
                }
            }),
            vec!["token", "owner", "repo", "run_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_get",
                "Get detailed information about a specific workflow run including status and timing.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubRunGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: i64 = input.get_arg("run_id")?;

        debug!(owner = %owner, repo = %repo, run_id = %run_id, "Getting GitHub workflow run");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}",
            owner, repo, run_id
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
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
            return Ok(handle_github_error(status, &body));
        }

        Ok(ToolResult::success(body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run Cancel Tool
// ============================================================================

/// Cancel a running workflow
pub struct GitHubRunCancelTool {
    config: ToolConfig,
}

impl GitHubRunCancelTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token with repo scope"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Workflow run ID to cancel"
                }
            }),
            vec!["token", "owner", "repo", "run_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_cancel",
                "Cancel a running workflow. Cancellation is asynchronous.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubRunCancelTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunCancelTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: i64 = input.get_arg("run_id")?;

        debug!(owner = %owner, repo = %repo, run_id = %run_id, "Cancelling GitHub workflow run");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}/cancel",
            owner, repo, run_id
        );

        let response = match client.post(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 202 Accepted indicates success for cancel
        if status == 202 {
            return Ok(ToolResult::success(serde_json::json!({
                "status": "cancelled",
                "run_id": run_id,
                "message": "Workflow run cancelled successfully"
            })));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run Rerun Tool
// ============================================================================

/// Rerun a failed or cancelled workflow
pub struct GitHubRunRerunTool {
    config: ToolConfig,
}

impl GitHubRunRerunTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token with repo scope"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Workflow run ID to rerun"
                },
                "enable_debug_logging": {
                    "type": "boolean",
                    "description": "Enable debug logging for rerun",
                    "default": false
                },
                "rerun_failed_jobs": {
                    "type": "boolean",
                    "description": "Only rerun failed jobs instead of entire workflow",
                    "default": false
                }
            }),
            vec!["token", "owner", "repo", "run_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_rerun",
                "Rerun a failed or cancelled workflow. Optionally with debug logging.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubRunRerunTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunRerunTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: i64 = input.get_arg("run_id")?;
        let enable_debug_logging: bool = input.get_arg("enable_debug_logging").unwrap_or(false);
        let rerun_failed_jobs: bool = input.get_arg("rerun_failed_jobs").unwrap_or(false);

        debug!(owner = %owner, repo = %repo, run_id = %run_id, "Rerunning GitHub workflow");

        let client = create_github_client(&token)?;

        // Choose endpoint based on whether we're rerunning failed jobs only
        let url = if rerun_failed_jobs {
            format!(
                "https://api.github.com/repos/{}/{}/actions/runs/{}/rerun-failed-jobs",
                owner, repo, run_id
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/actions/runs/{}/rerun",
                owner, repo, run_id
            )
        };

        let payload = if enable_debug_logging {
            serde_json::json!({"enable_debug_logging": true})
        } else {
            serde_json::json!({})
        };

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 201 Created indicates success for rerun
        if status == 201 {
            return Ok(ToolResult::success(serde_json::json!({
                "status": "rerun_requested",
                "run_id": run_id,
                "debug_logging": enable_debug_logging,
                "failed_jobs_only": rerun_failed_jobs,
                "message": "Workflow rerun initiated"
            })));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run Force Cancel Tool
// ============================================================================

/// Force cancel an unresponsive workflow
pub struct GitHubRunForceCancelTool {
    config: ToolConfig,
}

impl GitHubRunForceCancelTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token with repo scope"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Workflow run ID to force cancel"
                }
            }),
            vec!["token", "owner", "repo", "run_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_force_cancel",
                "Force cancel an unresponsive workflow that isn't responding to normal cancellation.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubRunForceCancelTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunForceCancelTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: i64 = input.get_arg("run_id")?;

        debug!(owner = %owner, repo = %repo, run_id = %run_id, "Force cancelling GitHub workflow run");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}/force-cancel",
            owner, repo, run_id
        );

        let response = match client.post(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 202 Accepted indicates success for force cancel
        if status == 202 {
            return Ok(ToolResult::success(serde_json::json!({
                "status": "force_cancelled",
                "run_id": run_id,
                "message": "Workflow force cancelled"
            })));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Artifacts List Tool
// ============================================================================

/// List artifacts for a workflow run or repository
pub struct GitHubArtifactsListTool {
    config: ToolConfig,
}

impl GitHubArtifactsListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Filter by specific workflow run"
                },
                "name": {
                    "type": "string",
                    "description": "Filter by artifact name"
                },
                "per_page": {
                    "type": "integer",
                    "description": "Results per page (max 100)",
                    "default": 30
                },
                "page": {
                    "type": "integer",
                    "description": "Page number",
                    "default": 1
                }
            }),
            vec!["token", "owner", "repo"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_artifacts_list",
                "List artifacts for a workflow run or repository. Artifacts expire after 90 days by default.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitHubArtifactsListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubArtifactsListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: Option<i64> = input.get_arg("run_id").ok();
        let name: Option<String> = input.get_arg("name").ok();
        let per_page: i32 = input.get_arg("per_page").unwrap_or(30);
        let page: i32 = input.get_arg("page").unwrap_or(1);

        debug!(owner = %owner, repo = %repo, "Listing GitHub artifacts");

        let client = create_github_client(&token)?;

        // Build URL based on whether run_id is specified
        let url = if let Some(rid) = run_id {
            format!(
                "https://api.github.com/repos/{}/{}/actions/runs/{}/artifacts",
                owner, repo, rid
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/actions/artifacts",
                owner, repo
            )
        };

        let mut params: Vec<(&str, String)> = vec![
            ("per_page", per_page.to_string()),
            ("page", page.to_string()),
        ];

        if let Some(n) = name {
            params.push(("name", n));
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
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
            return Ok(handle_github_error(status, &body));
        }

        let artifacts = body.get("artifacts").cloned().unwrap_or(serde_json::json!([]));
        let total_count = body.get("total_count").and_then(|c| c.as_i64()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "artifacts": artifacts,
            "total_count": total_count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Artifacts Download Tool
// ============================================================================

/// Download a workflow artifact
pub struct GitHubArtifactsDownloadTool {
    config: ToolConfig,
}

impl GitHubArtifactsDownloadTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "artifact_id": {
                    "type": "integer",
                    "description": "Artifact ID to download"
                },
                "output_path": {
                    "type": "string",
                    "description": "Local file path to save artifact"
                }
            }),
            vec!["token", "owner", "repo", "artifact_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_artifacts_download",
                "Download a workflow artifact. Returns download URL for the ZIP archive.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GitHubArtifactsDownloadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubArtifactsDownloadTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let artifact_id: i64 = input.get_arg("artifact_id")?;
        let output_path: Option<String> = input.get_arg("output_path").ok();

        debug!(owner = %owner, repo = %repo, artifact_id = %artifact_id, "Downloading GitHub artifact");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/artifacts/{}/zip",
            owner, repo, artifact_id
        );

        // Just get the redirect URL, don't actually download
        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 302 redirect to actual download URL
        if status == 200 || status == 302 {
            let download_url = response
                .headers()
                .get("location")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            return Ok(ToolResult::success(serde_json::json!({
                "artifact_id": artifact_id,
                "download_url": download_url,
                "output_path": output_path.unwrap_or_else(|| format!("artifact-{}.zip", artifact_id)),
                "message": "Artifact download URL retrieved. Use the URL to download the ZIP archive."
            })));
        }

        if status == 410 {
            return Ok(ToolResult::error(
                "Artifact expired or deleted. Artifacts expire after 90 days.".to_string()
            ));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// GitHub Run Logs Tool
// ============================================================================

/// Download workflow run logs
pub struct GitHubRunLogsTool {
    config: ToolConfig,
}

impl GitHubRunLogsTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "token": {
                    "type": "string",
                    "description": "GitHub Personal Access Token"
                },
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "run_id": {
                    "type": "integer",
                    "description": "Workflow run ID"
                },
                "output_path": {
                    "type": "string",
                    "description": "Local file path to save logs"
                }
            }),
            vec!["token", "owner", "repo", "run_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "github_run_logs",
                "Download workflow run logs as a ZIP archive containing separate log files for each job.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GitHubRunLogsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitHubRunLogsTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let token: String = input.get_arg("token")?;
        let owner: String = input.get_arg("owner")?;
        let repo: String = input.get_arg("repo")?;
        let run_id: i64 = input.get_arg("run_id")?;
        let output_path: Option<String> = input.get_arg("output_path").ok();

        debug!(owner = %owner, repo = %repo, run_id = %run_id, "Downloading GitHub workflow logs");

        let client = create_github_client(&token)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}/logs",
            owner, repo, run_id
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!("GitHub request failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();

        // 302 redirect to actual download URL
        if status == 200 || status == 302 {
            let download_url = response
                .headers()
                .get("location")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            return Ok(ToolResult::success(serde_json::json!({
                "run_id": run_id,
                "download_url": download_url,
                "output_path": output_path.unwrap_or_else(|| format!("logs-{}.zip", run_id)),
                "message": "Log download URL retrieved. Use the URL to download the ZIP archive."
            })));
        }

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        Ok(handle_github_error(status, &body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_workflow_list_tool_creation() {
        let tool = GitHubWorkflowListTool::new();
        assert_eq!(tool.config().name, "github_workflow_list");
    }

    #[test]
    fn test_github_workflow_dispatch_tool_creation() {
        let tool = GitHubWorkflowDispatchTool::new();
        assert_eq!(tool.config().name, "github_workflow_dispatch");
    }

    #[test]
    fn test_github_run_list_tool_creation() {
        let tool = GitHubRunListTool::new();
        assert_eq!(tool.config().name, "github_run_list");
    }

    #[test]
    fn test_github_run_get_tool_creation() {
        let tool = GitHubRunGetTool::new();
        assert_eq!(tool.config().name, "github_run_get");
    }

    #[test]
    fn test_github_run_cancel_tool_creation() {
        let tool = GitHubRunCancelTool::new();
        assert_eq!(tool.config().name, "github_run_cancel");
    }

    #[test]
    fn test_github_run_rerun_tool_creation() {
        let tool = GitHubRunRerunTool::new();
        assert_eq!(tool.config().name, "github_run_rerun");
    }

    #[test]
    fn test_github_artifacts_list_tool_creation() {
        let tool = GitHubArtifactsListTool::new();
        assert_eq!(tool.config().name, "github_artifacts_list");
    }

    #[test]
    fn test_github_tools_all() {
        let tools = GitHubActionsTools::all();
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn test_handle_github_error_401() {
        let body = serde_json::json!({"message": "Bad credentials"});
        let result = handle_github_error(401, &body);
        assert!(!result.success);
    }

    #[test]
    fn test_handle_github_error_404() {
        let body = serde_json::json!({"message": "Not Found"});
        let result = handle_github_error(404, &body);
        assert!(!result.success);
    }
}
