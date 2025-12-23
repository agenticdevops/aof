//! Grafana Tools
//!
//! Tools for querying and interacting with Grafana's REST API.
//!
//! ## Available Tools
//!
//! - `grafana_query` - Query data sources through Grafana
//! - `grafana_dashboard_get` - Get dashboard by UID
//! - `grafana_dashboard_list` - Search dashboards
//! - `grafana_alert_list` - List alert rules
//! - `grafana_alert_silence` - Create alert silence
//! - `grafana_annotation_create` - Create annotation
//!
//! ## Prerequisites
//!
//! - Requires `observability` feature flag
//! - Valid Grafana endpoint and API key
//! - Service account token with appropriate permissions
//!
//! ## Authentication
//!
//! All tools use Bearer token authentication via API keys.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Grafana tools
pub struct GrafanaTools;

impl GrafanaTools {
    /// Get all Grafana tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GrafanaQueryTool::new()),
            Box::new(GrafanaDashboardGetTool::new()),
            Box::new(GrafanaDashboardListTool::new()),
            Box::new(GrafanaAlertListTool::new()),
            Box::new(GrafanaAlertSilenceTool::new()),
            Box::new(GrafanaAnnotationCreateTool::new()),
        ]
    }
}

/// Create Grafana HTTP client with authentication
fn create_grafana_client(
    api_key: &str,
    org_id: Option<u64>,
) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Bearer token authentication
    let auth_value = format!("Bearer {}", api_key);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
    );

    // Optional organization ID header
    if let Some(org) = org_id {
        headers.insert(
            "X-Grafana-Org-Id",
            reqwest::header::HeaderValue::from_str(&org.to_string())
                .map_err(|e| aof_core::AofError::tool(format!("Invalid org ID: {}", e)))?,
        );
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

// ============================================================================
// Grafana Query Tool
// ============================================================================

/// Query data sources through Grafana
pub struct GrafanaQueryTool {
    config: ToolConfig,
}

impl GrafanaQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL (e.g., https://grafana.example.com)"
                },
                "datasource_uid": {
                    "type": "string",
                    "description": "Data source UID from Grafana"
                },
                "query": {
                    "type": "string",
                    "description": "Query in data source's native language (PromQL, LogQL, etc.)"
                },
                "from": {
                    "type": "string",
                    "description": "Start time (RFC3339 or Unix timestamp in milliseconds)"
                },
                "to": {
                    "type": "string",
                    "description": "End time (RFC3339 or Unix timestamp in milliseconds)"
                },
                "max_data_points": {
                    "type": "integer",
                    "description": "Maximum number of data points to return",
                    "default": 1000
                },
                "interval_ms": {
                    "type": "integer",
                    "description": "Interval in milliseconds between data points"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key or service account token"
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID for multi-org setups"
                }
            }),
            vec!["endpoint", "datasource_uid", "query", "api_key"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_query",
                "Query data sources through Grafana's unified query API. Supports Prometheus, Loki, and other data sources.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let datasource_uid: String = input.get_arg("datasource_uid")?;
        let query: String = input.get_arg("query")?;
        let api_key: String = input.get_arg("api_key")?;

        let from: Option<String> = input.get_arg("from").ok();
        let to: Option<String> = input.get_arg("to").ok();
        let max_data_points: i32 = input.get_arg("max_data_points").unwrap_or(1000);
        let interval_ms: Option<i32> = input.get_arg("interval_ms").ok();
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, datasource_uid = %datasource_uid, "Querying Grafana");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/ds/query", endpoint.trim_end_matches('/'));

        // Build query payload
        let mut query_payload = serde_json::json!({
            "queries": [{
                "datasource": {
                    "uid": datasource_uid
                },
                "expr": query,
                "refId": "A",
                "maxDataPoints": max_data_points
            }]
        });

        if let Some(f) = from {
            query_payload["from"] = serde_json::json!(f);
        }
        if let Some(t) = to {
            query_payload["to"] = serde_json::json!(t);
        }
        if let Some(interval) = interval_ms {
            query_payload["queries"][0]["intervalMs"] = serde_json::json!(interval);
        }

        let response = match client.post(&url).json(&query_payload).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check endpoint URL.",
                        e
                    )));
                } else {
                    return Ok(ToolResult::error(format!("Grafana query failed: {}", e)));
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

        if status == 401 {
            return Ok(ToolResult::error(
                "Authentication failed. Check API key and permissions.".to_string()
            ));
        }

        if status == 404 {
            return Ok(ToolResult::error(format!(
                "Resource not found: {}",
                body.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
            )));
        }

        if status == 429 {
            let retry_after = "unknown"; // Response headers not easily accessible here
            return Ok(ToolResult::error(format!(
                "Rate limited. Retry after: {}",
                retry_after
            )));
        }

        if status >= 500 {
            return Ok(ToolResult::error(format!(
                "Grafana server error ({}): {:?}",
                status,
                body.get("message")
            )));
        }

        if status != 200 {
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "results": body.get("results"),
            "datasource_uid": datasource_uid,
            "query": query
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Grafana Dashboard Get Tool
// ============================================================================

/// Get dashboard by UID
pub struct GrafanaDashboardGetTool {
    config: ToolConfig,
}

impl GrafanaDashboardGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL"
                },
                "dashboard_uid": {
                    "type": "string",
                    "description": "Dashboard UID"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key"
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID"
                }
            }),
            vec!["endpoint", "dashboard_uid", "api_key"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_dashboard_get",
                "Retrieve a dashboard by UID. Returns complete dashboard JSON for inspection.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaDashboardGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaDashboardGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let dashboard_uid: String = input.get_arg("dashboard_uid")?;
        let api_key: String = input.get_arg("api_key")?;
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, dashboard_uid = %dashboard_uid, "Getting Grafana dashboard");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/dashboards/uid/{}", endpoint.trim_end_matches('/'), dashboard_uid);

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to get dashboard: {}", e)));
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
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "dashboard": body.get("dashboard"),
            "meta": body.get("meta")
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Grafana Dashboard List Tool
// ============================================================================

/// Search and list dashboards
pub struct GrafanaDashboardListTool {
    config: ToolConfig,
}

impl GrafanaDashboardListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key"
                },
                "query": {
                    "type": "string",
                    "description": "Search query string"
                },
                "tags": {
                    "type": "array",
                    "description": "Tags to filter by",
                    "items": {
                        "type": "string"
                    }
                },
                "folder_ids": {
                    "type": "array",
                    "description": "Folder IDs to filter by",
                    "items": {
                        "type": "integer"
                    }
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results",
                    "default": 100
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID"
                }
            }),
            vec!["endpoint", "api_key"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_dashboard_list",
                "Search and list dashboards. Filter by query, tags, or folders.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaDashboardListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaDashboardListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let api_key: String = input.get_arg("api_key")?;
        let query: Option<String> = input.get_arg("query").ok();
        let tags: Option<Vec<String>> = input.get_arg("tags").ok();
        let folder_ids: Option<Vec<i64>> = input.get_arg("folder_ids").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(100);
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, "Listing Grafana dashboards");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/search", endpoint.trim_end_matches('/'));

        let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];

        if let Some(q) = query {
            params.push(("query", q));
        }

        if let Some(tag_list) = tags {
            for tag in tag_list {
                params.push(("tag", tag));
            }
        }

        if let Some(folder_list) = folder_ids {
            for folder in folder_list {
                params.push(("folderIds", folder.to_string()));
            }
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to list dashboards: {}", e)));
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
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status, body
            )));
        }

        let count = body.as_array().map(|a| a.len()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "dashboards": body,
            "count": count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Grafana Alert List Tool
// ============================================================================

/// List alert rules
pub struct GrafanaAlertListTool {
    config: ToolConfig,
}

impl GrafanaAlertListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key"
                },
                "dashboard_uid": {
                    "type": "string",
                    "description": "Filter by dashboard UID"
                },
                "panel_id": {
                    "type": "integer",
                    "description": "Filter by panel ID"
                },
                "state": {
                    "type": "string",
                    "description": "Filter by state",
                    "enum": ["alerting", "ok", "no_data", "paused"]
                },
                "folder_id": {
                    "type": "integer",
                    "description": "Filter by folder ID"
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID"
                }
            }),
            vec!["endpoint", "api_key"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_alert_list",
                "List alert rules and their current states. Supports filtering by dashboard, state, and folder.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaAlertListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaAlertListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let api_key: String = input.get_arg("api_key")?;
        let dashboard_uid: Option<String> = input.get_arg("dashboard_uid").ok();
        let panel_id: Option<i64> = input.get_arg("panel_id").ok();
        let state: Option<String> = input.get_arg("state").ok();
        let folder_id: Option<i64> = input.get_arg("folder_id").ok();
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, "Listing Grafana alerts");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/alerts", endpoint.trim_end_matches('/'));

        let mut params: Vec<(&str, String)> = vec![];

        if let Some(uid) = dashboard_uid {
            params.push(("dashboardUid", uid));
        }

        if let Some(pid) = panel_id {
            params.push(("panelId", pid.to_string()));
        }

        if let Some(s) = state {
            params.push(("state", s));
        }

        if let Some(fid) = folder_id {
            params.push(("folderId", fid.to_string()));
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to list alerts: {}", e)));
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
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status, body
            )));
        }

        let count = body.as_array().map(|a| a.len()).unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "alerts": body,
            "count": count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Grafana Alert Silence Tool
// ============================================================================

/// Create alert silence
pub struct GrafanaAlertSilenceTool {
    config: ToolConfig,
}

impl GrafanaAlertSilenceTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key"
                },
                "matchers": {
                    "type": "array",
                    "description": "Label matchers for silence",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string"
                            },
                            "value": {
                                "type": "string"
                            },
                            "isRegex": {
                                "type": "boolean",
                                "default": false
                            }
                        },
                        "required": ["name", "value"]
                    }
                },
                "starts_at": {
                    "type": "string",
                    "description": "Start time (RFC3339), default: now"
                },
                "ends_at": {
                    "type": "string",
                    "description": "End time (RFC3339)"
                },
                "comment": {
                    "type": "string",
                    "description": "Reason for silence"
                },
                "created_by": {
                    "type": "string",
                    "description": "Username"
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID"
                }
            }),
            vec!["endpoint", "api_key", "matchers", "ends_at", "comment"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_alert_silence",
                "Create an alert silence to suppress notifications. Requires matchers, end time, and reason.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaAlertSilenceTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaAlertSilenceTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let api_key: String = input.get_arg("api_key")?;
        let matchers: Vec<serde_json::Value> = input.get_arg("matchers")?;
        let ends_at: String = input.get_arg("ends_at")?;
        let comment: String = input.get_arg("comment")?;
        let starts_at: Option<String> = input.get_arg("starts_at").ok();
        let created_by: Option<String> = input.get_arg("created_by").ok();
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, "Creating Grafana alert silence");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/alertmanager/grafana/api/v2/silences", endpoint.trim_end_matches('/'));

        let mut payload = serde_json::json!({
            "matchers": matchers,
            "endsAt": ends_at,
            "comment": comment
        });

        if let Some(start) = &starts_at {
            payload["startsAt"] = serde_json::json!(start);
        }

        if let Some(creator) = &created_by {
            payload["createdBy"] = serde_json::json!(creator);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to create silence: {}", e)));
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
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "silence_id": body.get("silenceID"),
            "starts_at": starts_at,
            "ends_at": ends_at
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Grafana Annotation Create Tool
// ============================================================================

/// Create annotation
pub struct GrafanaAnnotationCreateTool {
    config: ToolConfig,
}

impl GrafanaAnnotationCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Grafana server URL"
                },
                "api_key": {
                    "type": "string",
                    "description": "Grafana API key"
                },
                "time": {
                    "type": "integer",
                    "description": "Event time in Unix milliseconds, default: now"
                },
                "time_end": {
                    "type": "integer",
                    "description": "End time for region annotations"
                },
                "text": {
                    "type": "string",
                    "description": "Annotation text"
                },
                "tags": {
                    "type": "array",
                    "description": "Tags for annotation",
                    "items": {
                        "type": "string"
                    }
                },
                "dashboard_uid": {
                    "type": "string",
                    "description": "Associate with specific dashboard"
                },
                "panel_id": {
                    "type": "integer",
                    "description": "Associate with specific panel"
                },
                "org_id": {
                    "type": "integer",
                    "description": "Organization ID"
                }
            }),
            vec!["endpoint", "api_key", "text"],
        );

        Self {
            config: tool_config_with_timeout(
                "grafana_annotation_create",
                "Create an annotation on dashboards. Marks significant events like deployments or incidents.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GrafanaAnnotationCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrafanaAnnotationCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let api_key: String = input.get_arg("api_key")?;
        let text: String = input.get_arg("text")?;
        let time: Option<i64> = input.get_arg("time").ok();
        let time_end: Option<i64> = input.get_arg("time_end").ok();
        let tags: Option<Vec<String>> = input.get_arg("tags").ok();
        let dashboard_uid: Option<String> = input.get_arg("dashboard_uid").ok();
        let panel_id: Option<i64> = input.get_arg("panel_id").ok();
        let org_id: Option<u64> = input.get_arg("org_id").ok();

        debug!(endpoint = %endpoint, "Creating Grafana annotation");

        let client = create_grafana_client(&api_key, org_id)?;

        let url = format!("{}/api/annotations", endpoint.trim_end_matches('/'));

        let mut payload = serde_json::json!({
            "text": text
        });

        if let Some(t) = time {
            payload["time"] = serde_json::json!(t);
        }

        if let Some(te) = time_end {
            payload["timeEnd"] = serde_json::json!(te);
        }

        if let Some(tag_list) = tags {
            payload["tags"] = serde_json::json!(tag_list);
        }

        if let Some(uid) = dashboard_uid {
            payload["dashboardUID"] = serde_json::json!(uid);
        }

        if let Some(pid) = panel_id {
            payload["panelId"] = serde_json::json!(pid);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to create annotation: {}", e)));
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
            return Ok(ToolResult::error(format!(
                "Grafana returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "annotation_id": body.get("id"),
            "message": body.get("message").unwrap_or(&serde_json::json!("Annotation created"))
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
