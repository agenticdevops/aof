//! Splunk Tools
//!
//! Tools for querying and interacting with Splunk's observability and SIEM platform.
//!
//! ## Available Tools
//!
//! - `splunk_search` - Execute SPL (Search Processing Language) queries
//! - `splunk_alerts_list` - List fired/triggered alerts
//! - `splunk_saved_searches` - List configured saved searches
//! - `splunk_saved_search_run` - Execute a saved search
//! - `splunk_hec_send` - Send events via HTTP Event Collector
//! - `splunk_indexes_list` - List available indexes
//!
//! ## Prerequisites
//!
//! - Requires `siem` feature flag
//! - Valid Splunk authentication token or username/password
//! - Network access to Splunk REST API (port 8089) and HEC (port 8088)
//!
//! ## Authentication
//!
//! - Bearer token: `Authorization: Bearer <token>`
//! - Splunk token: `Authorization: Splunk <token>`
//! - HEC: `Authorization: Splunk <hec_token>`
//!
//! ## Important Notes
//!
//! Splunk searches are asynchronous. The search tool creates a job, polls for
//! completion, and retrieves results automatically.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Splunk tools
pub struct SplunkTools;

impl SplunkTools {
    /// Get all Splunk tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(SplunkSearchTool::new()),
            Box::new(SplunkAlertsListTool::new()),
            Box::new(SplunkSavedSearchesTool::new()),
            Box::new(SplunkSavedSearchRunTool::new()),
            Box::new(SplunkHecSendTool::new()),
            Box::new(SplunkIndexesListTool::new()),
        ]
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create authenticated Splunk HTTP client
async fn create_splunk_client(token: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    // Splunk may use self-signed certs in some deployments
    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle Splunk API response
async fn handle_splunk_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("{} parse error: {}", operation, e)))?;

    // Check for Splunk error messages
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        let errors: Vec<String> = messages
            .iter()
            .filter(|m| m.get("type").and_then(|t| t.as_str()) == Some("ERROR"))
            .filter_map(|m| m.get("text").and_then(|t| t.as_str()))
            .map(String::from)
            .collect();

        if !errors.is_empty() {
            return Ok(ToolResult::error(format!(
                "{} errors: {}",
                operation,
                errors.join("; ")
            )));
        }
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!(
            "{} HTTP {}: {:?}",
            operation, status, body
        )));
    }

    Ok(ToolResult::success(body))
}

// ============================================================================
// Search Tool
// ============================================================================

/// Execute SPL queries against Splunk data
pub struct SplunkSearchTool {
    config: ToolConfig,
}

impl SplunkSearchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "base_url": {
                    "type": "string",
                    "description": "Splunk REST API base URL (e.g., https://splunk.company.com:8089)"
                },
                "token": {
                    "type": "string",
                    "description": "Splunk authentication token. Can use env var SPLUNK_TOKEN"
                },
                "query": {
                    "type": "string",
                    "description": "SPL search query (e.g., 'index=web status>=500 | stats count by host')"
                },
                "earliest_time": {
                    "type": "string",
                    "description": "Start time (e.g., '-1h', '-1d@d', '2025-12-25T00:00:00')",
                    "default": "-1h"
                },
                "latest_time": {
                    "type": "string",
                    "description": "End time (e.g., 'now', '@d', '2025-12-25T12:00:00')",
                    "default": "now"
                },
                "max_count": {
                    "type": "integer",
                    "description": "Maximum results to return",
                    "default": 1000
                }
            }),
            vec!["base_url", "token", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_search",
                "Execute SPL (Search Processing Language) queries against Splunk data. Search logs, metrics, and events.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for SplunkSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkSearchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let base_url: String = input.get_arg("base_url")?;
        let token: String = input.get_arg("token")?;
        let query: String = input.get_arg("query")?;
        let earliest_time: String = input
            .get_arg("earliest_time")
            .unwrap_or_else(|_| "-1h".to_string());
        let latest_time: String = input
            .get_arg("latest_time")
            .unwrap_or_else(|_| "now".to_string());
        let max_count: i32 = input.get_arg("max_count").unwrap_or(1000);

        debug!(query = %query, "Executing Splunk search");

        let client = create_splunk_client(&token).await?;
        let base = base_url.trim_end_matches('/');

        // 1. Create search job
        let create_url = format!("{}/services/search/v2/jobs", base);
        let form_data = [
            ("search", query.as_str()),
            ("output_mode", "json"),
            ("exec_mode", "normal"),
            ("earliest_time", earliest_time.as_str()),
            ("latest_time", latest_time.as_str()),
        ];

        let create_response = client
            .post(&create_url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create search job: {}", e)))?;

        let job_response: serde_json::Value = create_response
            .json()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to parse job response: {}", e)))?;

        let sid = job_response["sid"]
            .as_str()
            .ok_or_else(|| aof_core::AofError::tool("No SID in response".to_string()))?;

        debug!(sid = %sid, "Search job created, polling for completion");

        // 2. Poll for completion
        let status_url = format!("{}/services/search/v2/jobs/{}", base, sid);
        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 60;

        loop {
            attempts += 1;
            if attempts > MAX_ATTEMPTS {
                return Ok(ToolResult::error(
                    "Search job timed out waiting for completion".to_string(),
                ));
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

            let status_response = client
                .get(&status_url)
                .query(&[("output_mode", "json")])
                .send()
                .await
                .map_err(|e| aof_core::AofError::tool(format!("Failed to check job status: {}", e)))?;

            let status: serde_json::Value = status_response.json().await.map_err(|e| {
                aof_core::AofError::tool(format!("Failed to parse status response: {}", e))
            })?;

            let dispatch_state = status["entry"][0]["content"]["dispatchState"]
                .as_str()
                .unwrap_or("");

            match dispatch_state {
                "DONE" => break,
                "FAILED" => {
                    return Ok(ToolResult::error("Search job failed".to_string()));
                }
                _ => continue,
            }
        }

        // 3. Retrieve results
        let results_url = format!("{}/services/search/v2/jobs/{}/results", base, sid);
        let results_response = client
            .get(&results_url)
            .query(&[
                ("output_mode", "json"),
                ("count", &max_count.to_string()),
            ])
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to retrieve results: {}", e)))?;

        handle_splunk_response(results_response, "Splunk search").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Alerts List Tool
// ============================================================================

/// List fired/triggered Splunk alerts
pub struct SplunkAlertsListTool {
    config: ToolConfig,
}

impl SplunkAlertsListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "base_url": {
                    "type": "string",
                    "description": "Splunk REST API base URL"
                },
                "token": {
                    "type": "string",
                    "description": "Splunk authentication token"
                },
                "count": {
                    "type": "integer",
                    "description": "Number of alerts to retrieve",
                    "default": 50
                }
            }),
            vec!["base_url", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_alerts_list",
                "List fired/triggered alerts from Splunk for incident response workflows.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for SplunkAlertsListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkAlertsListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let base_url: String = input.get_arg("base_url")?;
        let token: String = input.get_arg("token")?;
        let count: i32 = input.get_arg("count").unwrap_or(50);

        debug!("Listing Splunk fired alerts");

        let client = create_splunk_client(&token).await?;
        let url = format!(
            "{}/servicesNS/-/-/alerts/fired_alerts",
            base_url.trim_end_matches('/')
        );

        let response = client
            .get(&url)
            .query(&[("output_mode", "json"), ("count", &count.to_string())])
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to list alerts: {}", e)))?;

        handle_splunk_response(response, "Splunk alerts list").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Saved Searches Tool
// ============================================================================

/// List Splunk saved searches
pub struct SplunkSavedSearchesTool {
    config: ToolConfig,
}

impl SplunkSavedSearchesTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "base_url": {
                    "type": "string",
                    "description": "Splunk REST API base URL"
                },
                "token": {
                    "type": "string",
                    "description": "Splunk authentication token"
                },
                "search": {
                    "type": "string",
                    "description": "Filter by name pattern"
                },
                "count": {
                    "type": "integer",
                    "description": "Number of results",
                    "default": 50
                }
            }),
            vec!["base_url", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_saved_searches",
                "List configured saved searches in Splunk for management and execution.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for SplunkSavedSearchesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkSavedSearchesTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let base_url: String = input.get_arg("base_url")?;
        let token: String = input.get_arg("token")?;
        let search: Option<String> = input.get_arg("search").ok();
        let count: i32 = input.get_arg("count").unwrap_or(50);

        debug!("Listing Splunk saved searches");

        let client = create_splunk_client(&token).await?;
        let url = format!(
            "{}/servicesNS/-/-/saved/searches",
            base_url.trim_end_matches('/')
        );

        let mut params = vec![
            ("output_mode".to_string(), "json".to_string()),
            ("count".to_string(), count.to_string()),
        ];

        if let Some(s) = search {
            params.push(("search".to_string(), s));
        }

        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to list saved searches: {}", e)))?;

        handle_splunk_response(response, "Splunk saved searches").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Run Saved Search Tool
// ============================================================================

/// Execute a Splunk saved search
pub struct SplunkSavedSearchRunTool {
    config: ToolConfig,
}

impl SplunkSavedSearchRunTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "base_url": {
                    "type": "string",
                    "description": "Splunk REST API base URL"
                },
                "token": {
                    "type": "string",
                    "description": "Splunk authentication token"
                },
                "name": {
                    "type": "string",
                    "description": "Saved search name"
                },
                "trigger_actions": {
                    "type": "boolean",
                    "description": "Whether to trigger alert actions",
                    "default": false
                }
            }),
            vec!["base_url", "token", "name"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_saved_search_run",
                "Execute a pre-configured saved search in Splunk for scheduled or on-demand analysis.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for SplunkSavedSearchRunTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkSavedSearchRunTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let base_url: String = input.get_arg("base_url")?;
        let token: String = input.get_arg("token")?;
        let name: String = input.get_arg("name")?;
        let trigger_actions: bool = input.get_arg("trigger_actions").unwrap_or(false);

        debug!(name = %name, "Running Splunk saved search");

        let client = create_splunk_client(&token).await?;
        let url = format!(
            "{}/servicesNS/-/-/saved/searches/{}/dispatch",
            base_url.trim_end_matches('/'),
            urlencoding::encode(&name)
        );

        let form_data = [
            ("output_mode", "json"),
            ("trigger_actions", if trigger_actions { "1" } else { "0" }),
        ];

        let response = client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to run saved search: {}", e)))?;

        handle_splunk_response(response, "Splunk saved search run").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// HEC Send Tool
// ============================================================================

/// Send events to Splunk via HTTP Event Collector
pub struct SplunkHecSendTool {
    config: ToolConfig,
}

impl SplunkHecSendTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "hec_url": {
                    "type": "string",
                    "description": "Splunk HEC endpoint URL (e.g., https://splunk.company.com:8088)"
                },
                "hec_token": {
                    "type": "string",
                    "description": "HEC token (GUID format). Can use env var SPLUNK_HEC_TOKEN"
                },
                "event": {
                    "type": "object",
                    "description": "Event data to send"
                },
                "source": {
                    "type": "string",
                    "description": "Event source",
                    "default": "aof"
                },
                "sourcetype": {
                    "type": "string",
                    "description": "Event sourcetype",
                    "default": "aof:event"
                },
                "index": {
                    "type": "string",
                    "description": "Target index"
                },
                "host": {
                    "type": "string",
                    "description": "Host value for the event"
                }
            }),
            vec!["hec_url", "hec_token", "event"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_hec_send",
                "Send events to Splunk via HTTP Event Collector for ingestion and analysis.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for SplunkHecSendTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkHecSendTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let hec_url: String = input.get_arg("hec_url")?;
        let hec_token: String = input.get_arg("hec_token")?;
        let event: serde_json::Value = input.get_arg("event")?;
        let source: String = input
            .get_arg("source")
            .unwrap_or_else(|_| "aof".to_string());
        let sourcetype: String = input
            .get_arg("sourcetype")
            .unwrap_or_else(|_| "aof:event".to_string());
        let index: Option<String> = input.get_arg("index").ok();
        let host: Option<String> = input.get_arg("host").ok();

        debug!(source = %source, "Sending event to Splunk HEC");

        let url = format!(
            "{}/services/collector/event",
            hec_url.trim_end_matches('/')
        );

        let mut payload = json!({
            "event": event,
            "source": source,
            "sourcetype": sourcetype
        });

        if let Some(idx) = index {
            payload["index"] = json!(idx);
        }
        if let Some(h) = host {
            payload["host"] = json!(h);
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create client: {}", e)))?;

        let response = client
            .post(&url)
            .header("Authorization", format!("Splunk {}", hec_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("HEC request failed: {}", e)))?;

        let status = response.status().as_u16();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to parse response: {}", e)))?;

        if status >= 400 {
            return Ok(ToolResult::error(format!("HEC error {}: {:?}", status, body)));
        }

        Ok(ToolResult::success(body))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Indexes List Tool
// ============================================================================

/// List available Splunk indexes
pub struct SplunkIndexesListTool {
    config: ToolConfig,
}

impl SplunkIndexesListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "base_url": {
                    "type": "string",
                    "description": "Splunk REST API base URL"
                },
                "token": {
                    "type": "string",
                    "description": "Splunk authentication token"
                }
            }),
            vec!["base_url", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "splunk_indexes_list",
                "List available Splunk indexes to discover data sources for querying.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for SplunkIndexesListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SplunkIndexesListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let base_url: String = input.get_arg("base_url")?;
        let token: String = input.get_arg("token")?;

        debug!("Listing Splunk indexes");

        let client = create_splunk_client(&token).await?;
        let url = format!("{}/services/data/indexes", base_url.trim_end_matches('/'));

        let response = client
            .get(&url)
            .query(&[("output_mode", "json")])
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Failed to list indexes: {}", e)))?;

        handle_splunk_response(response, "Splunk indexes list").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_tool_config() {
        let tool = SplunkSearchTool::new();
        assert_eq!(tool.config().name, "splunk_search");
    }

    #[test]
    fn test_alerts_list_tool_config() {
        let tool = SplunkAlertsListTool::new();
        assert_eq!(tool.config().name, "splunk_alerts_list");
    }

    #[test]
    fn test_saved_searches_tool_config() {
        let tool = SplunkSavedSearchesTool::new();
        assert_eq!(tool.config().name, "splunk_saved_searches");
    }

    #[test]
    fn test_saved_search_run_tool_config() {
        let tool = SplunkSavedSearchRunTool::new();
        assert_eq!(tool.config().name, "splunk_saved_search_run");
    }

    #[test]
    fn test_hec_send_tool_config() {
        let tool = SplunkHecSendTool::new();
        assert_eq!(tool.config().name, "splunk_hec_send");
    }

    #[test]
    fn test_indexes_list_tool_config() {
        let tool = SplunkIndexesListTool::new();
        assert_eq!(tool.config().name, "splunk_indexes_list");
    }

    #[test]
    fn test_splunk_tools_all() {
        let tools = SplunkTools::all();
        assert_eq!(tools.len(), 6);
    }
}
