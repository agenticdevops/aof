//! SonarQube Tools
//!
//! Tools for code quality analysis, security vulnerability detection, and technical debt management.
//!
//! ## Available Tools
//!
//! - `sonar_project_status` - Get quality gate status for a project
//! - `sonar_issues_search` - Search for bugs, vulnerabilities, and code smells
//! - `sonar_hotspots_search` - Find security hotspots requiring review
//! - `sonar_measures_component` - Get metrics like coverage, bugs, and complexity
//! - `sonar_issue_transition` - Change issue status (resolve, confirm, reopen)
//! - `sonar_project_analyses` - Get project analysis history
//!
//! ## Prerequisites
//!
//! - Requires `security` feature flag
//! - Valid SonarQube server endpoint
//! - Authentication token (passed as parameter)
//!
//! ## Authentication
//!
//! All tools use token authentication via Bearer or Basic auth header.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all SonarQube tools
pub struct SonarQubeTools;

impl SonarQubeTools {
    /// Get all SonarQube tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(SonarProjectStatusTool::new()),
            Box::new(SonarIssuesSearchTool::new()),
            Box::new(SonarHotspotsSearchTool::new()),
            Box::new(SonarMeasuresComponentTool::new()),
            Box::new(SonarIssueTransitionTool::new()),
            Box::new(SonarProjectAnalysesTool::new()),
        ]
    }
}

/// Create SonarQube HTTP client with authentication
fn create_sonar_client(token: &str) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Token authentication - use as Bearer token
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle common SonarQube error responses
fn handle_sonar_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let errors = body
        .get("errors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("msg").and_then(|m| m.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .or_else(|| {
            body.get("error")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "Unknown error".to_string());

    match status {
        400 => ToolResult::error(format!("Bad request: {}", errors)),
        401 => ToolResult::error(format!("Authentication failed: {}", errors)),
        403 => ToolResult::error(format!("Permission denied: {}", errors)),
        404 => ToolResult::error("Project not found or path does not exist".to_string()),
        429 => ToolResult::error("Rate limited. Retry after a delay.".to_string()),
        500..=599 => ToolResult::error(format!("SonarQube server error ({}): {}", status, errors)),
        _ => ToolResult::error(format!("SonarQube returned status {}: {}", status, errors)),
    }
}

// ============================================================================
// Sonar Project Status Tool
// ============================================================================

/// Get project quality gate status
pub struct SonarProjectStatusTool {
    config: ToolConfig,
}

impl SonarProjectStatusTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL (e.g., https://sonarqube.example.com)"
                },
                "project_key": {
                    "type": "string",
                    "description": "Project key in SonarQube"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (optional, uses main branch if not specified)"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "project_key", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_project_status",
                "Get quality gate status for a SonarQube project. Returns pass/fail status and condition details.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarProjectStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarProjectStatusTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_key: String = input.get_arg("project_key")?;
        let token: String = input.get_arg("token")?;
        let branch: Option<String> = input.get_arg("branch").ok();

        debug!(endpoint = %endpoint, project_key = %project_key, "Getting SonarQube quality gate status");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/qualitygates/project_status?projectKey={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&project_key)
        );

        if let Some(b) = branch {
            url.push_str(&format!("&branch={}", urlencoding::encode(&b)));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check SonarQube endpoint.",
                        e
                    )));
                } else {
                    return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status, &body));
        }

        let project_status = body.get("projectStatus");
        let gate_status = project_status
            .and_then(|ps| ps.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");

        let conditions = project_status
            .and_then(|ps| ps.get("conditions"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|cond| {
                        serde_json::json!({
                            "metric": cond.get("metricKey").and_then(|v| v.as_str()),
                            "operator": cond.get("comparator").and_then(|v| v.as_str()),
                            "value": cond.get("actualValue").and_then(|v| v.as_str()),
                            "threshold": cond.get("errorThreshold").and_then(|v| v.as_str()),
                            "status": cond.get("status").and_then(|v| v.as_str())
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "project_key": project_key,
            "status": gate_status,
            "conditions": conditions
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Sonar Issues Search Tool
// ============================================================================

/// Search for issues in a project
pub struct SonarIssuesSearchTool {
    config: ToolConfig,
}

impl SonarIssuesSearchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL"
                },
                "project_key": {
                    "type": "string",
                    "description": "Project key"
                },
                "types": {
                    "type": "string",
                    "description": "Issue types (comma-separated): BUG, VULNERABILITY, CODE_SMELL"
                },
                "severities": {
                    "type": "string",
                    "description": "Severities (comma-separated): BLOCKER, CRITICAL, MAJOR, MINOR, INFO"
                },
                "statuses": {
                    "type": "string",
                    "description": "Statuses (comma-separated): OPEN, CONFIRMED, RESOLVED, CLOSED"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name"
                },
                "page": {
                    "type": "integer",
                    "description": "Page number (default: 1)"
                },
                "page_size": {
                    "type": "integer",
                    "description": "Results per page (default: 100)"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "project_key", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_issues_search",
                "Search for issues (bugs, vulnerabilities, code smells) in a SonarQube project.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarIssuesSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarIssuesSearchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_key: String = input.get_arg("project_key")?;
        let token: String = input.get_arg("token")?;
        let types: Option<String> = input.get_arg("types").ok();
        let severities: Option<String> = input.get_arg("severities").ok();
        let statuses: Option<String> = input.get_arg("statuses").ok();
        let branch: Option<String> = input.get_arg("branch").ok();
        let page: Option<i32> = input.get_arg("page").ok();
        let page_size: Option<i32> = input.get_arg("page_size").ok();

        debug!(endpoint = %endpoint, project_key = %project_key, "Searching SonarQube issues");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/issues/search?componentKeys={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&project_key)
        );

        if let Some(t) = types {
            url.push_str(&format!("&types={}", urlencoding::encode(&t)));
        }
        if let Some(s) = severities {
            url.push_str(&format!("&severities={}", urlencoding::encode(&s)));
        }
        if let Some(st) = statuses {
            url.push_str(&format!("&statuses={}", urlencoding::encode(&st)));
        }
        if let Some(b) = branch {
            url.push_str(&format!("&branch={}", urlencoding::encode(&b)));
        }
        if let Some(p) = page {
            url.push_str(&format!("&p={}", p));
        }
        if let Some(ps) = page_size {
            url.push_str(&format!("&ps={}", ps));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status, &body));
        }

        let total = body.get("total").and_then(|t| t.as_i64()).unwrap_or(0);

        let issues = body
            .get("issues")
            .and_then(|i| i.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|issue| {
                        serde_json::json!({
                            "key": issue.get("key").and_then(|v| v.as_str()),
                            "type": issue.get("type").and_then(|v| v.as_str()),
                            "severity": issue.get("severity").and_then(|v| v.as_str()),
                            "message": issue.get("message").and_then(|v| v.as_str()),
                            "component": issue.get("component").and_then(|v| v.as_str()),
                            "line": issue.get("line").and_then(|v| v.as_i64()),
                            "rule": issue.get("rule").and_then(|v| v.as_str()),
                            "status": issue.get("status").and_then(|v| v.as_str()),
                            "effort": issue.get("effort").and_then(|v| v.as_str()),
                            "tags": issue.get("tags")
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "total": total,
            "issues": issues
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Sonar Hotspots Search Tool
// ============================================================================

/// Search for security hotspots
pub struct SonarHotspotsSearchTool {
    config: ToolConfig,
}

impl SonarHotspotsSearchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL"
                },
                "project_key": {
                    "type": "string",
                    "description": "Project key"
                },
                "status": {
                    "type": "string",
                    "description": "Hotspot status: TO_REVIEW, REVIEWED"
                },
                "resolution": {
                    "type": "string",
                    "description": "Resolution: FIXED, SAFE, ACKNOWLEDGED"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "project_key", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_hotspots_search",
                "Search for security hotspots that need review in a SonarQube project.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarHotspotsSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarHotspotsSearchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_key: String = input.get_arg("project_key")?;
        let token: String = input.get_arg("token")?;
        let status: Option<String> = input.get_arg("status").ok();
        let resolution: Option<String> = input.get_arg("resolution").ok();
        let branch: Option<String> = input.get_arg("branch").ok();

        debug!(endpoint = %endpoint, project_key = %project_key, "Searching SonarQube security hotspots");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/hotspots/search?projectKey={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&project_key)
        );

        if let Some(s) = status {
            url.push_str(&format!("&status={}", urlencoding::encode(&s)));
        }
        if let Some(r) = resolution {
            url.push_str(&format!("&resolution={}", urlencoding::encode(&r)));
        }
        if let Some(b) = branch {
            url.push_str(&format!("&branch={}", urlencoding::encode(&b)));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status_code, &body));
        }

        let hotspots = body
            .get("hotspots")
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|hotspot| {
                        serde_json::json!({
                            "key": hotspot.get("key").and_then(|v| v.as_str()),
                            "message": hotspot.get("message").and_then(|v| v.as_str()),
                            "component": hotspot.get("component").and_then(|v| v.as_str()),
                            "line": hotspot.get("line").and_then(|v| v.as_i64()),
                            "status": hotspot.get("status").and_then(|v| v.as_str()),
                            "vulnerability_probability": hotspot.get("vulnerabilityProbability").and_then(|v| v.as_str()),
                            "security_category": hotspot.get("securityCategory").and_then(|v| v.as_str())
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "hotspots": hotspots
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Sonar Measures Component Tool
// ============================================================================

/// Get metrics for a component
pub struct SonarMeasuresComponentTool {
    config: ToolConfig,
}

impl SonarMeasuresComponentTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL"
                },
                "component": {
                    "type": "string",
                    "description": "Component key (project or file)"
                },
                "metrics": {
                    "type": "string",
                    "description": "Comma-separated metrics: coverage, bugs, vulnerabilities, code_smells, duplicated_lines_density, ncloc"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "component", "metrics", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_measures_component",
                "Get code metrics (coverage, complexity, duplication) for a SonarQube component.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarMeasuresComponentTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarMeasuresComponentTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let component: String = input.get_arg("component")?;
        let metrics: String = input.get_arg("metrics")?;
        let token: String = input.get_arg("token")?;
        let branch: Option<String> = input.get_arg("branch").ok();

        debug!(endpoint = %endpoint, component = %component, "Getting SonarQube component measures");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/measures/component?component={}&metricKeys={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&component),
            urlencoding::encode(&metrics)
        );

        if let Some(b) = branch {
            url.push_str(&format!("&branch={}", urlencoding::encode(&b)));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status, &body));
        }

        let measures = body
            .get("component")
            .and_then(|c| c.get("measures"))
            .and_then(|m| m.as_array())
            .map(|arr| {
                let mut result = serde_json::Map::new();
                for measure in arr {
                    if let (Some(metric), Some(value)) = (
                        measure.get("metric").and_then(|m| m.as_str()),
                        measure.get("value").and_then(|v| v.as_str()),
                    ) {
                        result.insert(metric.to_string(), serde_json::json!(value));
                    }
                }
                serde_json::Value::Object(result)
            })
            .unwrap_or(serde_json::json!({}));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "component": component,
            "measures": measures
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Sonar Issue Transition Tool
// ============================================================================

/// Change issue status
pub struct SonarIssueTransitionTool {
    config: ToolConfig,
}

impl SonarIssueTransitionTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL"
                },
                "issue_key": {
                    "type": "string",
                    "description": "Issue key"
                },
                "transition": {
                    "type": "string",
                    "description": "Transition: confirm, resolve, reopen, wontfix, falsepositive"
                },
                "comment": {
                    "type": "string",
                    "description": "Comment for the transition"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "issue_key", "transition", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_issue_transition",
                "Change SonarQube issue status (resolve, confirm, reopen, wontfix, falsepositive).",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarIssueTransitionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarIssueTransitionTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let issue_key: String = input.get_arg("issue_key")?;
        let transition: String = input.get_arg("transition")?;
        let token: String = input.get_arg("token")?;
        let comment: Option<String> = input.get_arg("comment").ok();

        debug!(endpoint = %endpoint, issue_key = %issue_key, transition = %transition, "Transitioning SonarQube issue");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/issues/do_transition?issue={}&transition={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&issue_key),
            urlencoding::encode(&transition)
        );

        if let Some(c) = comment {
            url.push_str(&format!("&comment={}", urlencoding::encode(&c)));
        }

        let response = match client.post(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status, &body));
        }

        let issue = body.get("issue");
        let issue_status = issue
            .and_then(|i| i.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");

        let resolution = issue
            .and_then(|i| i.get("resolution"))
            .and_then(|r| r.as_str());

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "issue": {
                "key": issue_key,
                "status": issue_status,
                "resolution": resolution
            }
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Sonar Project Analyses Tool
// ============================================================================

/// Get project analysis history
pub struct SonarProjectAnalysesTool {
    config: ToolConfig,
}

impl SonarProjectAnalysesTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "SonarQube server URL"
                },
                "project_key": {
                    "type": "string",
                    "description": "Project key"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name"
                },
                "from": {
                    "type": "string",
                    "description": "Start date (YYYY-MM-DD)"
                },
                "to": {
                    "type": "string",
                    "description": "End date (YYYY-MM-DD)"
                },
                "token": {
                    "type": "string",
                    "description": "SonarQube authentication token"
                }
            }),
            vec!["endpoint", "project_key", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "sonar_project_analyses",
                "Get SonarQube project analysis history to track trends over time.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SonarProjectAnalysesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SonarProjectAnalysesTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let project_key: String = input.get_arg("project_key")?;
        let token: String = input.get_arg("token")?;
        let branch: Option<String> = input.get_arg("branch").ok();
        let from: Option<String> = input.get_arg("from").ok();
        let to: Option<String> = input.get_arg("to").ok();

        debug!(endpoint = %endpoint, project_key = %project_key, "Getting SonarQube project analyses");

        let client = create_sonar_client(&token)?;

        let mut url = format!(
            "{}/api/project_analyses/search?project={}",
            endpoint.trim_end_matches('/'),
            urlencoding::encode(&project_key)
        );

        if let Some(b) = branch {
            url.push_str(&format!("&branch={}", urlencoding::encode(&b)));
        }
        if let Some(f) = from {
            url.push_str(&format!("&from={}", urlencoding::encode(&f)));
        }
        if let Some(t) = to {
            url.push_str(&format!("&to={}", urlencoding::encode(&t)));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("SonarQube request failed: {}", e)));
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
            return Ok(handle_sonar_error(status, &body));
        }

        let analyses = body
            .get("analyses")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|analysis| {
                        serde_json::json!({
                            "key": analysis.get("key").and_then(|v| v.as_str()),
                            "date": analysis.get("date").and_then(|v| v.as_str()),
                            "events": analysis.get("events")
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "analyses": analyses
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
    fn test_sonarqube_tools_creation() {
        let tools = SonarQubeTools::all();
        assert_eq!(tools.len(), 6);

        let names: Vec<&str> = tools.iter().map(|t| t.config().name.as_str()).collect();
        assert!(names.contains(&"sonar_project_status"));
        assert!(names.contains(&"sonar_issues_search"));
        assert!(names.contains(&"sonar_hotspots_search"));
        assert!(names.contains(&"sonar_measures_component"));
        assert!(names.contains(&"sonar_issue_transition"));
        assert!(names.contains(&"sonar_project_analyses"));
    }

    #[test]
    fn test_project_status_config() {
        let tool = SonarProjectStatusTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_project_status");
        assert!(config.description.contains("quality gate"));
    }

    #[test]
    fn test_issues_search_config() {
        let tool = SonarIssuesSearchTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_issues_search");
        assert!(config.description.contains("issues"));
    }

    #[test]
    fn test_hotspots_search_config() {
        let tool = SonarHotspotsSearchTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_hotspots_search");
        assert!(config.description.contains("security hotspots"));
    }

    #[test]
    fn test_measures_component_config() {
        let tool = SonarMeasuresComponentTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_measures_component");
        assert!(config.description.contains("metrics"));
    }

    #[test]
    fn test_issue_transition_config() {
        let tool = SonarIssueTransitionTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_issue_transition");
        assert!(config.description.contains("status"));
    }

    #[test]
    fn test_project_analyses_config() {
        let tool = SonarProjectAnalysesTool::new();
        let config = tool.config();

        assert_eq!(config.name, "sonar_project_analyses");
        assert!(config.description.contains("analysis history"));
    }
}
