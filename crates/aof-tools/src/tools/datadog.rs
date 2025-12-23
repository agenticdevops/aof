//! Datadog Tools
//!
//! Tools for querying and interacting with Datadog's observability platform.
//!
//! ## Available Tools
//!
//! - `datadog_metric_query` - Query metrics using Datadog query language
//! - `datadog_log_query` - Search logs using Datadog log search syntax
//! - `datadog_monitor_list` - List monitors and their current states
//! - `datadog_monitor_mute` - Mute a specific monitor or monitor group
//! - `datadog_event_post` - Post custom events to the event stream
//! - `datadog_downtime_create` - Create scheduled downtime for maintenance
//!
//! ## Prerequisites
//!
//! - Requires `observability` feature flag
//! - Valid Datadog API key (DD-API-KEY header)
//! - Valid Datadog Application key (DD-APPLICATION-KEY header)
//! - Correct endpoint for your Datadog region
//!
//! ## Authentication
//!
//! All tools require dual authentication:
//! - `api_key`: Organization API key (can use env var DATADOG_API_KEY)
//! - `app_key`: Application key with user permissions (can use env var DATADOG_APP_KEY)
//!
//! ## Supported Regions
//!
//! - US1 (default): https://api.datadoghq.com
//! - US3: https://api.us3.datadoghq.com
//! - US5: https://api.us5.datadoghq.com
//! - EU1: https://api.datadoghq.eu
//! - AP1: https://api.ap1.datadoghq.com
//! - US1-FED: https://api.ddog-gov.com

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Datadog tools
pub struct DatadogTools;

impl DatadogTools {
    /// Get all Datadog tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(DatadogMetricQueryTool::new()),
            Box::new(DatadogLogQueryTool::new()),
            Box::new(DatadogMonitorListTool::new()),
            Box::new(DatadogMonitorMuteTool::new()),
            Box::new(DatadogEventPostTool::new()),
            Box::new(DatadogDowntimeCreateTool::new()),
        ]
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create authenticated Datadog HTTP client
async fn create_datadog_client(api_key: &str, app_key: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "DD-API-KEY",
        reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
    );

    headers.insert(
        "DD-APPLICATION-KEY",
        reqwest::header::HeaderValue::from_str(app_key)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid app key: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle Datadog API response
async fn handle_datadog_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();

    // Parse response body
    let body: serde_json::Value = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            return Ok(ToolResult::error(format!(
                "{} failed to parse response: {}",
                operation, e
            )));
        }
    };

    // Check for errors
    if status >= 400 {
        let error_msg = body
            .get("errors")
            .or_else(|| body.get("error"))
            .map(|e| format!("{:?}", e))
            .unwrap_or_else(|| "Unknown error".to_string());

        return Ok(ToolResult::error(format!(
            "{} returned status {}: {}",
            operation, status, error_msg
        )));
    }

    // Success
    Ok(ToolResult::success(body))
}

/// Parse time parameter (Unix timestamp, ISO 8601, or relative time)
fn parse_time_param(time: &str) -> Result<i64, String> {
    // Try parsing as Unix timestamp
    if let Ok(ts) = time.parse::<i64>() {
        return Ok(ts);
    }

    // Try parsing as ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time) {
        return Ok(dt.timestamp());
    }

    // Try parsing relative time (e.g., "-1h", "-30m")
    if time.starts_with('-') {
        if let Some(duration) = parse_relative_time(&time[1..]) {
            let now = chrono::Utc::now().timestamp();
            return Ok(now - duration);
        }
    }

    // Handle "now"
    if time == "now" {
        return Ok(chrono::Utc::now().timestamp());
    }

    Err(format!(
        "Invalid time format: {}. Expected Unix timestamp, ISO 8601, or relative time like '-1h'",
        time
    ))
}

/// Parse relative time string (e.g., "1h", "30m", "7d")
fn parse_relative_time(s: &str) -> Option<i64> {
    if s.len() < 2 {
        return None;
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    match unit {
        "s" => Some(num),
        "m" => Some(num * 60),
        "h" => Some(num * 3600),
        "d" => Some(num * 86400),
        _ => None,
    }
}

// ============================================================================
// Datadog Metric Query Tool
// ============================================================================

/// Query Datadog metrics using Datadog query language
pub struct DatadogMetricQueryTool {
    config: ToolConfig,
}

impl DatadogMetricQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header). Can use env var DATADOG_API_KEY"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY header). Can use env var DATADOG_APP_KEY"
                },
                "query": {
                    "type": "string",
                    "description": "Metric query in Datadog query language (e.g., 'avg:system.cpu.user{*}')"
                },
                "from": {
                    "type": "string",
                    "description": "Start time (Unix timestamp, ISO 8601, or relative like '-1h')"
                },
                "to": {
                    "type": "string",
                    "description": "End time (Unix timestamp, ISO 8601, or 'now')"
                }
            }),
            vec!["api_key", "app_key", "query", "from", "to"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_metric_query",
                "Query Datadog metrics using Datadog query language. Returns time-series data for analysis and monitoring.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for DatadogMetricQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogMetricQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let query: String = input.get_arg("query")?;
        let from: String = input.get_arg("from")?;
        let to: String = input.get_arg("to")?;

        debug!(
            endpoint = %endpoint,
            query = %query,
            "Querying Datadog metrics"
        );

        // Parse time parameters
        let from_ts = match parse_time_param(&from) {
            Ok(ts) => ts,
            Err(e) => return Ok(ToolResult::error(format!("Invalid 'from' time: {}", e))),
        };

        let to_ts = match parse_time_param(&to) {
            Ok(ts) => ts,
            Err(e) => return Ok(ToolResult::error(format!("Invalid 'to' time: {}", e))),
        };

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request
        let url = format!("{}/api/v1/query", endpoint.trim_end_matches('/'));
        let params = [
            ("query", query.as_str()),
            ("from", &from_ts.to_string()),
            ("to", &to_ts.to_string()),
        ];

        // Execute request
        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog metric query failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog metric query").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Datadog Log Query Tool
// ============================================================================

/// Search Datadog logs using Datadog log search syntax
pub struct DatadogLogQueryTool {
    config: ToolConfig,
}

impl DatadogLogQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header)"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY header)"
                },
                "query": {
                    "type": "string",
                    "description": "Log search query (e.g., 'service:api status:error')"
                },
                "from": {
                    "type": "string",
                    "description": "Start time (ISO 8601 format)"
                },
                "to": {
                    "type": "string",
                    "description": "End time (ISO 8601 format)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum logs to return (max: 1000)",
                    "default": 50
                },
                "sort": {
                    "type": "string",
                    "description": "Sort order: 'timestamp' or '-timestamp'",
                    "enum": ["timestamp", "-timestamp"],
                    "default": "-timestamp"
                }
            }),
            vec!["api_key", "app_key", "query", "from", "to"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_log_query",
                "Search Datadog logs using Datadog log search syntax. Returns log entries for debugging and analysis.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for DatadogLogQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogLogQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let query: String = input.get_arg("query")?;
        let from: String = input.get_arg("from")?;
        let to: String = input.get_arg("to")?;
        let limit: i32 = input.get_arg("limit").unwrap_or(50);
        let sort: String = input
            .get_arg("sort")
            .unwrap_or_else(|_| "-timestamp".to_string());

        debug!(endpoint = %endpoint, query = %query, "Querying Datadog logs");

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request body
        let url = format!(
            "{}/api/v2/logs/events/search",
            endpoint.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "filter": {
                "query": query,
                "from": from,
                "to": to
            },
            "sort": sort,
            "page": {
                "limit": limit
            }
        });

        // Execute request
        let response = match client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog log query failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog log query").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Datadog Monitor List Tool
// ============================================================================

/// List Datadog monitors and their current states
pub struct DatadogMonitorListTool {
    config: ToolConfig,
}

impl DatadogMonitorListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header)"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY header)"
                },
                "tags": {
                    "type": "string",
                    "description": "Filter by tags (comma-separated: 'env:prod,service:api')"
                },
                "name": {
                    "type": "string",
                    "description": "Filter by monitor name (substring match)"
                },
                "monitor_tags": {
                    "type": "string",
                    "description": "Filter by monitor-specific tags"
                }
            }),
            vec!["api_key", "app_key"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_monitor_list",
                "List Datadog monitors and their current states. Use filters to narrow results.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for DatadogMonitorListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogMonitorListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let tags: Option<String> = input.get_arg("tags").ok();
        let name: Option<String> = input.get_arg("name").ok();
        let monitor_tags: Option<String> = input.get_arg("monitor_tags").ok();

        debug!(endpoint = %endpoint, "Listing Datadog monitors");

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request URL with query parameters
        let url = format!("{}/api/v1/monitor", endpoint.trim_end_matches('/'));
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(t) = tags {
            params.push(("tags", t));
        }
        if let Some(n) = name {
            params.push(("name", n));
        }
        if let Some(mt) = monitor_tags {
            params.push(("monitor_tags", mt));
        }

        // Execute request
        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog monitor list failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog monitor list").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Datadog Monitor Mute Tool
// ============================================================================

/// Mute a Datadog monitor or monitor group
pub struct DatadogMonitorMuteTool {
    config: ToolConfig,
}

impl DatadogMonitorMuteTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header)"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY header)"
                },
                "monitor_id": {
                    "type": "integer",
                    "description": "Monitor ID to mute"
                },
                "scope": {
                    "type": "string",
                    "description": "Scope to mute (e.g., 'host:web-01', 'env:staging')"
                },
                "end": {
                    "type": "integer",
                    "description": "End time for mute (Unix timestamp). Omit for indefinite mute"
                }
            }),
            vec!["api_key", "app_key", "monitor_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_monitor_mute",
                "Mute a Datadog monitor or monitor group. Use during maintenance or to silence false positives.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for DatadogMonitorMuteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogMonitorMuteTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let monitor_id: i64 = input.get_arg("monitor_id")?;
        let scope: Option<String> = input.get_arg("scope").ok();
        let end: Option<i64> = input.get_arg("end").ok();

        debug!(
            endpoint = %endpoint,
            monitor_id = monitor_id,
            "Muting Datadog monitor"
        );

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request body
        let mut body = serde_json::json!({});
        if let Some(s) = scope {
            body["scope"] = serde_json::json!(s);
        }
        if let Some(e) = end {
            body["end"] = serde_json::json!(e);
        }

        // Build URL
        let url = format!(
            "{}/api/v1/monitor/{}/mute",
            endpoint.trim_end_matches('/'),
            monitor_id
        );

        // Execute request
        let response = match client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog monitor mute failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog monitor mute").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Datadog Event Post Tool
// ============================================================================

/// Post custom events to the Datadog event stream
pub struct DatadogEventPostTool {
    config: ToolConfig,
}

impl DatadogEventPostTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header)"
                },
                "title": {
                    "type": "string",
                    "description": "Event title (max 500 characters)"
                },
                "text": {
                    "type": "string",
                    "description": "Event description (supports Markdown, max 4000 characters)"
                },
                "alert_type": {
                    "type": "string",
                    "description": "Event severity",
                    "enum": ["error", "warning", "info", "success"],
                    "default": "info"
                },
                "tags": {
                    "type": "array",
                    "description": "Event tags (e.g., ['env:prod', 'version:1.2.3'])",
                    "items": { "type": "string" }
                }
            }),
            vec!["api_key", "title", "text"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_event_post",
                "Post custom events to the Datadog event stream. Track deployments, incidents, and milestones.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for DatadogEventPostTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogEventPostTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let title: String = input.get_arg("title")?;
        let text: String = input.get_arg("text")?;
        let alert_type: String = input
            .get_arg("alert_type")
            .unwrap_or_else(|_| "info".to_string());
        let tags: Option<Vec<String>> = input.get_arg("tags").ok();

        debug!(endpoint = %endpoint, title = %title, "Posting Datadog event");

        // Create authenticated client (event post only needs API key)
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "DD-API-KEY",
            reqwest::header::HeaderValue::from_str(&api_key)
                .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        // Build request body
        let mut body = serde_json::json!({
            "title": title,
            "text": text,
            "alert_type": alert_type
        });

        if let Some(t) = tags {
            body["tags"] = serde_json::json!(t);
        }

        // Build URL
        let url = format!("{}/api/v1/events", endpoint.trim_end_matches('/'));

        // Execute request
        let response = match client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog event post failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog event post").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Datadog Downtime Create Tool
// ============================================================================

/// Create scheduled downtime for maintenance windows
pub struct DatadogDowntimeCreateTool {
    config: ToolConfig,
}

impl DatadogDowntimeCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Datadog API endpoint (default: https://api.datadoghq.com)",
                    "default": "https://api.datadoghq.com"
                },
                "api_key": {
                    "type": "string",
                    "description": "Datadog API key (DD-API-KEY header)"
                },
                "app_key": {
                    "type": "string",
                    "description": "Datadog Application key (DD-APPLICATION-KEY header)"
                },
                "scope": {
                    "type": "array",
                    "description": "Scopes to apply downtime (e.g., ['host:web-*', 'env:prod'])",
                    "items": { "type": "string" }
                },
                "start": {
                    "type": "integer",
                    "description": "Start time (Unix timestamp, default: now)"
                },
                "end": {
                    "type": "integer",
                    "description": "End time (Unix timestamp). Required if no recurrence"
                },
                "message": {
                    "type": "string",
                    "description": "Message to include with notifications"
                }
            }),
            vec!["api_key", "app_key", "scope"],
        );

        Self {
            config: tool_config_with_timeout(
                "datadog_downtime_create",
                "Create scheduled downtime for maintenance windows. Suppress alerts during planned work.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for DatadogDowntimeCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DatadogDowntimeCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input
            .get_arg("endpoint")
            .unwrap_or_else(|_| "https://api.datadoghq.com".to_string());
        let api_key: String = input.get_arg("api_key")?;
        let app_key: String = input.get_arg("app_key")?;
        let scope: Vec<String> = input.get_arg("scope")?;
        let start: Option<i64> = input.get_arg("start").ok();
        let end: Option<i64> = input.get_arg("end").ok();
        let message: Option<String> = input.get_arg("message").ok();

        debug!(
            endpoint = %endpoint,
            scope = ?scope,
            "Creating Datadog downtime"
        );

        // Create authenticated client
        let client = match create_datadog_client(&api_key, &app_key).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::error(format!("Authentication failed: {}", e))),
        };

        // Build request body
        let mut body = serde_json::json!({
            "scope": scope
        });

        if let Some(s) = start {
            body["start"] = serde_json::json!(s);
        }
        if let Some(e) = end {
            body["end"] = serde_json::json!(e);
        }
        if let Some(m) = message {
            body["message"] = serde_json::json!(m);
        }

        // Build URL
        let url = format!("{}/api/v1/downtime", endpoint.trim_end_matches('/'));

        // Execute request
        let response = match client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Datadog downtime create failed: {}",
                    e
                )));
            }
        };

        // Handle response
        handle_datadog_response(response, "Datadog downtime create").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_unix() {
        let result = parse_time_param("1639065600");
        assert_eq!(result.unwrap(), 1639065600);
    }

    #[test]
    fn test_parse_time_iso8601() {
        let result = parse_time_param("2024-01-15T10:30:00Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_time_relative() {
        let result = parse_relative_time("1h");
        assert_eq!(result, Some(3600));

        let result = parse_relative_time("30m");
        assert_eq!(result, Some(1800));

        let result = parse_relative_time("7d");
        assert_eq!(result, Some(604800));
    }

    #[test]
    fn test_parse_time_now() {
        let result = parse_time_param("now");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_time_invalid() {
        let result = parse_time_param("invalid");
        assert!(result.is_err());
    }
}
