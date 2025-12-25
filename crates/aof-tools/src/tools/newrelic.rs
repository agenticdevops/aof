//! New Relic Tools
//!
//! Tools for querying and interacting with New Relic's observability platform via NerdGraph (GraphQL).
//!
//! ## Available Tools
//!
//! - `newrelic_nrql_query` - Execute NRQL queries against New Relic data
//! - `newrelic_alerts_list` - List alert policies and conditions
//! - `newrelic_incidents_list` - List active and recent incidents
//! - `newrelic_entity_search` - Search for monitored entities
//! - `newrelic_metrics_query` - Query metric timeslice data
//! - `newrelic_incident_ack` - Acknowledge an incident
//!
//! ## Prerequisites
//!
//! - Requires `observability` feature flag
//! - Valid New Relic User API Key (NRAK-...)
//! - Account ID for account-scoped operations
//!
//! ## Authentication
//!
//! All tools use User API Key authentication via the `API-Key` header.
//! REST API keys are deprecated as of March 1, 2025.
//!
//! ## Supported Regions
//!
//! - US (default): https://api.newrelic.com/graphql
//! - EU: https://api.eu.newrelic.com/graphql

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all New Relic tools
pub struct NewRelicTools;

impl NewRelicTools {
    /// Get all New Relic tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(NewRelicNrqlQueryTool::new()),
            Box::new(NewRelicAlertsListTool::new()),
            Box::new(NewRelicIncidentsListTool::new()),
            Box::new(NewRelicEntitySearchTool::new()),
            Box::new(NewRelicMetricsQueryTool::new()),
            Box::new(NewRelicIncidentAckTool::new()),
        ]
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get New Relic NerdGraph endpoint based on region
fn get_endpoint(region: &str) -> &'static str {
    match region.to_lowercase().as_str() {
        "eu" | "eu1" => "https://api.eu.newrelic.com/graphql",
        _ => "https://api.newrelic.com/graphql",
    }
}

/// Create authenticated New Relic HTTP client
async fn create_newrelic_client(api_key: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "API-Key",
        reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
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

/// Execute a GraphQL query against NerdGraph
async fn execute_graphql(
    client: &Client,
    endpoint: &str,
    query: &str,
    variables: serde_json::Value,
) -> AofResult<ToolResult> {
    let body = json!({
        "query": query,
        "variables": variables
    });

    let response = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("GraphQL request failed: {}", e)))?;

    let status = response.status().as_u16();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to parse response: {}", e)))?;

    // Check for GraphQL errors
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
            .map(String::from)
            .collect();

        if !error_messages.is_empty() {
            return Ok(ToolResult::error(format!(
                "GraphQL errors: {}",
                error_messages.join("; ")
            )));
        }
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!("HTTP {}: {:?}", status, body)));
    }

    Ok(ToolResult::success(
        body.get("data").cloned().unwrap_or(body),
    ))
}

// ============================================================================
// NRQL Query Tool
// ============================================================================

/// Execute NRQL queries against New Relic data
pub struct NewRelicNrqlQueryTool {
    config: ToolConfig,
}

impl NewRelicNrqlQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...). Can use env var NEWRELIC_API_KEY"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID. Can use env var NEWRELIC_ACCOUNT_ID"
                },
                "query": {
                    "type": "string",
                    "description": "NRQL query (e.g., 'SELECT count(*) FROM Transaction SINCE 1 hour ago')"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                }
            }),
            vec!["api_key", "account_id", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_nrql_query",
                "Execute NRQL queries against New Relic data. Query metrics, logs, traces, and events using New Relic Query Language.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for NewRelicNrqlQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicNrqlQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let account_id: String = input.get_arg("account_id")?;
        let query: String = input.get_arg("query")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(account_id = %account_id, query = %query, "Executing NRQL query");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            query NrqlQuery($accountId: Int!, $nrql: Nrql!) {
                actor {
                    account(id: $accountId) {
                        nrql(query: $nrql) {
                            results
                            totalResult
                            metadata {
                                timeWindow {
                                    begin
                                    end
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "accountId": account_id.parse::<i64>().unwrap_or(0),
            "nrql": query
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Alerts List Tool
// ============================================================================

/// List New Relic alert policies
pub struct NewRelicAlertsListTool {
    config: ToolConfig,
}

impl NewRelicAlertsListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum policies to return",
                    "default": 50
                }
            }),
            vec!["api_key", "account_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_alerts_list",
                "List New Relic alert policies and their configurations for analysis and management.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for NewRelicAlertsListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicAlertsListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let account_id: String = input.get_arg("account_id")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(account_id = %account_id, "Listing New Relic alert policies");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            query ListAlertPolicies($accountId: Int!, $cursor: String) {
                actor {
                    account(id: $accountId) {
                        alerts {
                            policiesSearch(cursor: $cursor) {
                                policies {
                                    id
                                    name
                                    incidentPreference
                                }
                                nextCursor
                                totalCount
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "accountId": account_id.parse::<i64>().unwrap_or(0),
            "cursor": null
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Incidents List Tool
// ============================================================================

/// List New Relic active and recent incidents
pub struct NewRelicIncidentsListTool {
    config: ToolConfig,
}

impl NewRelicIncidentsListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                },
                "states": {
                    "type": "array",
                    "description": "Filter by state: ACTIVATED, CREATED, CLOSED",
                    "items": { "type": "string" },
                    "default": ["ACTIVATED", "CREATED"]
                }
            }),
            vec!["api_key", "account_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_incidents_list",
                "List active and recent incidents from New Relic for incident response workflows.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for NewRelicIncidentsListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicIncidentsListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let account_id: String = input.get_arg("account_id")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(account_id = %account_id, "Listing New Relic incidents");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            query ListIncidents($accountId: Int!) {
                actor {
                    account(id: $accountId) {
                        aiIssues {
                            issues(filter: {states: [ACTIVATED, CREATED]}) {
                                issues {
                                    issueId
                                    title
                                    priority
                                    state
                                    activatedAt
                                    closedAt
                                    origins
                                    entityGuids
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "accountId": account_id.parse::<i64>().unwrap_or(0)
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Entity Search Tool
// ============================================================================

/// Search for New Relic monitored entities
pub struct NewRelicEntitySearchTool {
    config: ToolConfig,
}

impl NewRelicEntitySearchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "query": {
                    "type": "string",
                    "description": "Entity search query (e.g., \"type = 'APPLICATION'\" or \"name LIKE 'prod%'\")"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum entities to return",
                    "default": 50
                }
            }),
            vec!["api_key", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_entity_search",
                "Search for monitored entities (applications, hosts, services) across the New Relic platform.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for NewRelicEntitySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicEntitySearchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let query: String = input.get_arg("query")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());
        let limit: i32 = input.get_arg("limit").unwrap_or(50);

        debug!(query = %query, "Searching New Relic entities");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            query EntitySearch($query: String!, $limit: Int) {
                actor {
                    entitySearch(query: $query) {
                        results(limit: $limit) {
                            entities {
                                guid
                                name
                                type
                                domain
                                entityType
                                reporting
                                tags {
                                    key
                                    values
                                }
                                alertSeverity
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "query": query,
            "limit": limit
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Metrics Query Tool
// ============================================================================

/// Query New Relic metric timeslice data
pub struct NewRelicMetricsQueryTool {
    config: ToolConfig,
}

impl NewRelicMetricsQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID"
                },
                "entity_guid": {
                    "type": "string",
                    "description": "Entity GUID to query metrics for"
                },
                "metric_names": {
                    "type": "array",
                    "description": "Array of metric names to query",
                    "items": { "type": "string" }
                },
                "from": {
                    "type": "string",
                    "description": "Start time (epoch milliseconds or ISO 8601)"
                },
                "to": {
                    "type": "string",
                    "description": "End time (epoch milliseconds or ISO 8601)"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                }
            }),
            vec!["api_key", "account_id", "entity_guid", "metric_names", "from", "to"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_metrics_query",
                "Query detailed metric timeslice data for specific entities for performance analysis.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for NewRelicMetricsQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicMetricsQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let entity_guid: String = input.get_arg("entity_guid")?;
        let metric_names: Vec<String> = input.get_arg("metric_names")?;
        let from: String = input.get_arg("from")?;
        let to: String = input.get_arg("to")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(entity_guid = %entity_guid, "Querying New Relic metrics");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        // Parse time values - try as milliseconds first, then as timestamp
        let from_ms: i64 = from.parse().unwrap_or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&from)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0)
        });
        let to_ms: i64 = to.parse().unwrap_or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&to)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0)
        });

        let graphql_query = r#"
            query MetricsQuery($entityGuid: EntityGuid!, $metricNames: [String!]!, $from: EpochMilliseconds!, $to: EpochMilliseconds!) {
                actor {
                    entity(guid: $entityGuid) {
                        ... on ApmApplicationEntity {
                            metricTimesliceData(
                                metrics: { names: $metricNames, values: ["average_response_time", "call_count"] }
                                from: $from
                                to: $to
                            ) {
                                results {
                                    name
                                    values {
                                        average
                                        total
                                        count
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "entityGuid": entity_guid,
            "metricNames": metric_names,
            "from": from_ms,
            "to": to_ms
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Incident Acknowledge Tool
// ============================================================================

/// Acknowledge a New Relic incident
pub struct NewRelicIncidentAckTool {
    config: ToolConfig,
}

impl NewRelicIncidentAckTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID"
                },
                "issue_id": {
                    "type": "string",
                    "description": "Issue/Incident ID to acknowledge"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                }
            }),
            vec!["api_key", "account_id", "issue_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_incident_ack",
                "Acknowledge an active incident in New Relic to mark it as being investigated.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for NewRelicIncidentAckTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NewRelicIncidentAckTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let account_id: String = input.get_arg("account_id")?;
        let issue_id: String = input.get_arg("issue_id")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(issue_id = %issue_id, "Acknowledging New Relic incident");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            mutation AcknowledgeIssue($accountId: Int!, $issueId: ID!) {
                aiIssuesAcknowledgeIssue(accountId: $accountId, issueId: $issueId) {
                    issue {
                        issueId
                        state
                        acknowledgedAt
                    }
                }
            }
        "#;

        let variables = json!({
            "accountId": account_id.parse::<i64>().unwrap_or(0),
            "issueId": issue_id
        });

        execute_graphql(&client, endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_endpoint_us() {
        assert_eq!(get_endpoint("us"), "https://api.newrelic.com/graphql");
    }

    #[test]
    fn test_get_endpoint_eu() {
        assert_eq!(get_endpoint("eu"), "https://api.eu.newrelic.com/graphql");
    }

    #[test]
    fn test_get_endpoint_default() {
        assert_eq!(get_endpoint("invalid"), "https://api.newrelic.com/graphql");
    }

    #[test]
    fn test_nrql_query_tool_config() {
        let tool = NewRelicNrqlQueryTool::new();
        assert_eq!(tool.config().name, "newrelic_nrql_query");
    }

    #[test]
    fn test_alerts_list_tool_config() {
        let tool = NewRelicAlertsListTool::new();
        assert_eq!(tool.config().name, "newrelic_alerts_list");
    }

    #[test]
    fn test_incidents_list_tool_config() {
        let tool = NewRelicIncidentsListTool::new();
        assert_eq!(tool.config().name, "newrelic_incidents_list");
    }

    #[test]
    fn test_entity_search_tool_config() {
        let tool = NewRelicEntitySearchTool::new();
        assert_eq!(tool.config().name, "newrelic_entity_search");
    }

    #[test]
    fn test_metrics_query_tool_config() {
        let tool = NewRelicMetricsQueryTool::new();
        assert_eq!(tool.config().name, "newrelic_metrics_query");
    }

    #[test]
    fn test_incident_ack_tool_config() {
        let tool = NewRelicIncidentAckTool::new();
        assert_eq!(tool.config().name, "newrelic_incident_ack");
    }

    #[test]
    fn test_newrelic_tools_all() {
        let tools = NewRelicTools::all();
        assert_eq!(tools.len(), 6);
    }
}
