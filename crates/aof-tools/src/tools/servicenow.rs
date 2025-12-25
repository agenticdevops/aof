//! ServiceNow Tools
//!
//! Tools for interacting with ServiceNow's IT Service Management (ITSM) platform.
//!
//! ## Available Tools
//!
//! - `servicenow_incident_create` - Create a new incident
//! - `servicenow_incident_query` - Query incidents with filters
//! - `servicenow_incident_update` - Update an existing incident
//! - `servicenow_incident_get` - Get incident details by sys_id or number
//! - `servicenow_cmdb_query` - Query CMDB Configuration Items
//! - `servicenow_change_create` - Create a change request
//!
//! ## Prerequisites
//!
//! - Requires `itsm` feature flag
//! - Valid ServiceNow credentials (Basic Auth or OAuth)
//! - Network access to ServiceNow instance
//!
//! ## Authentication
//!
//! - Basic Auth: `Authorization: Basic <base64(username:password)>`
//! - OAuth 2.0: `Authorization: Bearer <access_token>`
//!
//! ## Encoded Query Syntax
//!
//! ServiceNow uses encoded queries for filtering:
//! - `priority=1^state!=6` - High priority, not closed
//! - `assignment_group.name=Database Team^state=2` - Assigned to group, in progress
//! - `sys_created_on>javascript:gs.daysAgo(1)` - Created in last 24 hours

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use serde_json::json;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all ServiceNow tools
pub struct ServiceNowTools;

impl ServiceNowTools {
    /// Get all ServiceNow tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(ServiceNowIncidentCreateTool::new()),
            Box::new(ServiceNowIncidentQueryTool::new()),
            Box::new(ServiceNowIncidentUpdateTool::new()),
            Box::new(ServiceNowIncidentGetTool::new()),
            Box::new(ServiceNowCmdbQueryTool::new()),
            Box::new(ServiceNowChangeCreateTool::new()),
        ]
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create authenticated ServiceNow HTTP client
async fn create_servicenow_client(username: &str, password: &str) -> AofResult<Client> {
    let credentials = format!("{}:{}", username, password);
    let auth_header = format!("Basic {}", BASE64.encode(credentials.as_bytes()));

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&auth_header)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid auth: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle ServiceNow API response
async fn handle_servicenow_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("{} parse error: {}", operation, e)))?;

    // Check for ServiceNow error response
    if let Some(error) = body.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        let detail = error.get("detail").and_then(|d| d.as_str()).unwrap_or("");

        return Ok(ToolResult::error(format!(
            "{} failed: {} - {}",
            operation, message, detail
        )));
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!(
            "{} HTTP {}: {:?}",
            operation, status, body
        )));
    }

    // Extract result from response
    let result = body.get("result").cloned().unwrap_or(body);
    Ok(ToolResult::success(result))
}

// ============================================================================
// Incident Create Tool
// ============================================================================

/// Create a new incident in ServiceNow
pub struct ServiceNowIncidentCreateTool {
    config: ToolConfig,
}

impl ServiceNowIncidentCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL (e.g., https://company.service-now.com)"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username. Can use env var SERVICENOW_USERNAME"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password. Can use env var SERVICENOW_PASSWORD"
                },
                "short_description": {
                    "type": "string",
                    "description": "Brief incident summary (max 160 chars)"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed incident description"
                },
                "urgency": {
                    "type": "string",
                    "description": "Urgency: 1 (High), 2 (Medium), 3 (Low)",
                    "enum": ["1", "2", "3"],
                    "default": "2"
                },
                "impact": {
                    "type": "string",
                    "description": "Impact: 1 (High), 2 (Medium), 3 (Low)",
                    "enum": ["1", "2", "3"],
                    "default": "2"
                },
                "category": {
                    "type": "string",
                    "description": "Incident category"
                },
                "subcategory": {
                    "type": "string",
                    "description": "Incident subcategory"
                },
                "assignment_group": {
                    "type": "string",
                    "description": "Assignment group name or sys_id"
                },
                "assigned_to": {
                    "type": "string",
                    "description": "Assigned user name or sys_id"
                },
                "cmdb_ci": {
                    "type": "string",
                    "description": "Configuration Item sys_id"
                },
                "caller_id": {
                    "type": "string",
                    "description": "User who reported the incident"
                }
            }),
            vec!["instance_url", "username", "password", "short_description"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_create",
                "Create a new incident in ServiceNow for tracking and resolution.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowIncidentCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let short_description: String = input.get_arg("short_description")?;
        let description: Option<String> = input.get_arg("description").ok();
        let urgency: String = input.get_arg("urgency").unwrap_or_else(|_| "2".to_string());
        let impact: String = input.get_arg("impact").unwrap_or_else(|_| "2".to_string());
        let category: Option<String> = input.get_arg("category").ok();
        let subcategory: Option<String> = input.get_arg("subcategory").ok();
        let assignment_group: Option<String> = input.get_arg("assignment_group").ok();
        let assigned_to: Option<String> = input.get_arg("assigned_to").ok();
        let cmdb_ci: Option<String> = input.get_arg("cmdb_ci").ok();
        let caller_id: Option<String> = input.get_arg("caller_id").ok();

        debug!(short_description = %short_description, "Creating ServiceNow incident");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!(
            "{}/api/now/table/incident",
            instance_url.trim_end_matches('/')
        );

        let mut body = json!({
            "short_description": short_description,
            "urgency": urgency,
            "impact": impact
        });

        if let Some(desc) = description {
            body["description"] = json!(desc);
        }
        if let Some(cat) = category {
            body["category"] = json!(cat);
        }
        if let Some(subcat) = subcategory {
            body["subcategory"] = json!(subcat);
        }
        if let Some(group) = assignment_group {
            body["assignment_group"] = json!(group);
        }
        if let Some(user) = assigned_to {
            body["assigned_to"] = json!(user);
        }
        if let Some(ci) = cmdb_ci {
            body["cmdb_ci"] = json!(ci);
        }
        if let Some(caller) = caller_id {
            body["caller_id"] = json!(caller);
        }

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Request failed: {}", e)))?;

        handle_servicenow_response(response, "Create incident").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Incident Query Tool
// ============================================================================

/// Query incidents from ServiceNow
pub struct ServiceNowIncidentQueryTool {
    config: ToolConfig,
}

impl ServiceNowIncidentQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "query": {
                    "type": "string",
                    "description": "Encoded query (e.g., 'priority=1^state!=6')"
                },
                "fields": {
                    "type": "string",
                    "description": "Comma-separated fields to return (e.g., 'number,short_description,state')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results",
                    "default": 50
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset",
                    "default": 0
                }
            }),
            vec!["instance_url", "username", "password"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_query",
                "Query incidents from ServiceNow with filters and pagination.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowIncidentQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let query: Option<String> = input.get_arg("query").ok();
        let fields: Option<String> = input.get_arg("fields").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(50);
        let offset: i32 = input.get_arg("offset").unwrap_or(0);

        debug!("Querying ServiceNow incidents");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!(
            "{}/api/now/table/incident",
            instance_url.trim_end_matches('/')
        );

        let mut params: Vec<(&str, String)> = vec![
            ("sysparm_limit", limit.to_string()),
            ("sysparm_offset", offset.to_string()),
        ];

        if let Some(ref q) = query {
            params.push(("sysparm_query", q.clone()));
        }
        if let Some(ref f) = fields {
            params.push(("sysparm_fields", f.clone()));
        }

        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Query failed: {}", e)))?;

        handle_servicenow_response(response, "Query incidents").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Incident Update Tool
// ============================================================================

/// Update an existing incident in ServiceNow
pub struct ServiceNowIncidentUpdateTool {
    config: ToolConfig,
}

impl ServiceNowIncidentUpdateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "sys_id": {
                    "type": "string",
                    "description": "Incident sys_id"
                },
                "fields": {
                    "type": "object",
                    "description": "Fields to update (e.g., {\"state\": \"2\", \"work_notes\": \"Investigating\"})"
                }
            }),
            vec!["instance_url", "username", "password", "sys_id", "fields"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_update",
                "Update an existing incident in ServiceNow. Modify fields, add comments, or change status.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowIncidentUpdateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentUpdateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let sys_id: String = input.get_arg("sys_id")?;
        let fields: serde_json::Value = input.get_arg("fields")?;

        debug!(sys_id = %sys_id, "Updating ServiceNow incident");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!(
            "{}/api/now/table/incident/{}",
            instance_url.trim_end_matches('/'),
            sys_id
        );

        let response = client
            .patch(&url)
            .json(&fields)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Update failed: {}", e)))?;

        handle_servicenow_response(response, "Update incident").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Incident Get Tool
// ============================================================================

/// Get a single incident from ServiceNow
pub struct ServiceNowIncidentGetTool {
    config: ToolConfig,
}

impl ServiceNowIncidentGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "identifier": {
                    "type": "string",
                    "description": "Incident sys_id or number (e.g., INC0012345)"
                }
            }),
            vec!["instance_url", "username", "password", "identifier"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_get",
                "Get detailed information about a specific incident by sys_id or incident number.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowIncidentGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let identifier: String = input.get_arg("identifier")?;

        debug!(identifier = %identifier, "Getting ServiceNow incident");

        let client = create_servicenow_client(&username, &password).await?;

        // Determine if identifier is sys_id or incident number
        let url = if identifier.starts_with("INC") {
            // Query by number
            format!(
                "{}/api/now/table/incident?sysparm_query=number={}",
                instance_url.trim_end_matches('/'),
                identifier
            )
        } else {
            // Direct lookup by sys_id
            format!(
                "{}/api/now/table/incident/{}",
                instance_url.trim_end_matches('/'),
                identifier
            )
        };

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Get failed: {}", e)))?;

        handle_servicenow_response(response, "Get incident").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// CMDB Query Tool
// ============================================================================

/// Query CMDB Configuration Items
pub struct ServiceNowCmdbQueryTool {
    config: ToolConfig,
}

impl ServiceNowCmdbQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "class": {
                    "type": "string",
                    "description": "CI class (e.g., 'cmdb_ci_server', 'cmdb_ci_database', 'cmdb_ci_app_server')"
                },
                "query": {
                    "type": "string",
                    "description": "Encoded query string"
                },
                "fields": {
                    "type": "string",
                    "description": "Comma-separated fields to return"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results",
                    "default": 50
                }
            }),
            vec!["instance_url", "username", "password", "class"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_cmdb_query",
                "Query CMDB Configuration Items for incident context and impact analysis.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowCmdbQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowCmdbQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let class: String = input.get_arg("class")?;
        let query: Option<String> = input.get_arg("query").ok();
        let fields: Option<String> = input.get_arg("fields").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(50);

        debug!(class = %class, "Querying ServiceNow CMDB");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!(
            "{}/api/now/table/{}",
            instance_url.trim_end_matches('/'),
            class
        );

        let mut params: Vec<(&str, String)> = vec![("sysparm_limit", limit.to_string())];

        if let Some(ref q) = query {
            params.push(("sysparm_query", q.clone()));
        }
        if let Some(ref f) = fields {
            params.push(("sysparm_fields", f.clone()));
        }

        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("CMDB query failed: {}", e)))?;

        handle_servicenow_response(response, "CMDB query").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Change Create Tool
// ============================================================================

/// Create a change request in ServiceNow
pub struct ServiceNowChangeCreateTool {
    config: ToolConfig,
}

impl ServiceNowChangeCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "short_description": {
                    "type": "string",
                    "description": "Brief change summary"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed change description"
                },
                "type": {
                    "type": "string",
                    "description": "Change type: Standard, Normal, Emergency",
                    "enum": ["Standard", "Normal", "Emergency"],
                    "default": "Normal"
                },
                "risk": {
                    "type": "string",
                    "description": "Risk level: High, Moderate, Low",
                    "enum": ["High", "Moderate", "Low"],
                    "default": "Moderate"
                },
                "impact": {
                    "type": "string",
                    "description": "Impact: 1 (High), 2 (Medium), 3 (Low)",
                    "enum": ["1", "2", "3"],
                    "default": "2"
                },
                "start_date": {
                    "type": "string",
                    "description": "Planned start datetime (ISO 8601)"
                },
                "end_date": {
                    "type": "string",
                    "description": "Planned end datetime (ISO 8601)"
                },
                "cmdb_ci": {
                    "type": "string",
                    "description": "Affected CI sys_id"
                },
                "assignment_group": {
                    "type": "string",
                    "description": "Assignment group name or sys_id"
                }
            }),
            vec!["instance_url", "username", "password", "short_description"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_change_create",
                "Create a change request in ServiceNow for infrastructure or application changes.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ServiceNowChangeCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ServiceNowChangeCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let short_description: String = input.get_arg("short_description")?;
        let description: Option<String> = input.get_arg("description").ok();
        let change_type: String = input
            .get_arg("type")
            .unwrap_or_else(|_| "Normal".to_string());
        let risk: String = input
            .get_arg("risk")
            .unwrap_or_else(|_| "Moderate".to_string());
        let impact: String = input.get_arg("impact").unwrap_or_else(|_| "2".to_string());
        let start_date: Option<String> = input.get_arg("start_date").ok();
        let end_date: Option<String> = input.get_arg("end_date").ok();
        let cmdb_ci: Option<String> = input.get_arg("cmdb_ci").ok();
        let assignment_group: Option<String> = input.get_arg("assignment_group").ok();

        debug!(short_description = %short_description, "Creating ServiceNow change request");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!(
            "{}/api/now/table/change_request",
            instance_url.trim_end_matches('/')
        );

        let mut body = json!({
            "short_description": short_description,
            "type": change_type,
            "risk": risk,
            "impact": impact
        });

        if let Some(desc) = description {
            body["description"] = json!(desc);
        }
        if let Some(start) = start_date {
            body["start_date"] = json!(start);
        }
        if let Some(end) = end_date {
            body["end_date"] = json!(end);
        }
        if let Some(ci) = cmdb_ci {
            body["cmdb_ci"] = json!(ci);
        }
        if let Some(group) = assignment_group {
            body["assignment_group"] = json!(group);
        }

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Request failed: {}", e)))?;

        handle_servicenow_response(response, "Create change request").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_create_tool_config() {
        let tool = ServiceNowIncidentCreateTool::new();
        assert_eq!(tool.config().name, "servicenow_incident_create");
    }

    #[test]
    fn test_incident_query_tool_config() {
        let tool = ServiceNowIncidentQueryTool::new();
        assert_eq!(tool.config().name, "servicenow_incident_query");
    }

    #[test]
    fn test_incident_update_tool_config() {
        let tool = ServiceNowIncidentUpdateTool::new();
        assert_eq!(tool.config().name, "servicenow_incident_update");
    }

    #[test]
    fn test_incident_get_tool_config() {
        let tool = ServiceNowIncidentGetTool::new();
        assert_eq!(tool.config().name, "servicenow_incident_get");
    }

    #[test]
    fn test_cmdb_query_tool_config() {
        let tool = ServiceNowCmdbQueryTool::new();
        assert_eq!(tool.config().name, "servicenow_cmdb_query");
    }

    #[test]
    fn test_change_create_tool_config() {
        let tool = ServiceNowChangeCreateTool::new();
        assert_eq!(tool.config().name, "servicenow_change_create");
    }

    #[test]
    fn test_servicenow_tools_all() {
        let tools = ServiceNowTools::all();
        assert_eq!(tools.len(), 6);
    }
}
