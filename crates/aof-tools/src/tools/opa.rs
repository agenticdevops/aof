//! Open Policy Agent (OPA) Tools
//!
//! Tools for policy evaluation, compliance checking, and data management via OPA's REST API.
//!
//! ## Available Tools
//!
//! - `opa_eval` - Evaluate a policy against input data
//! - `opa_query` - Execute an ad-hoc Rego query
//! - `opa_data_get` - Get data from OPA's document store
//! - `opa_data_put` - Store data in OPA's document store
//! - `opa_policy_list` - List loaded policies
//! - `opa_policy_put` - Upload a new policy
//! - `opa_health` - Check OPA server health and status
//!
//! ## Prerequisites
//!
//! - Requires `security` feature flag
//! - OPA server running (default: http://localhost:8181)
//! - No authentication required for basic OPA deployments
//!
//! ## Authentication
//!
//! OPA does not require authentication by default. For secured deployments,
//! authentication can be added via reverse proxy or custom authentication plugins.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all OPA tools
pub struct OpaTools;

impl OpaTools {
    /// Get all OPA tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(OpaEvalTool::new()),
            Box::new(OpaQueryTool::new()),
            Box::new(OpaDataGetTool::new()),
            Box::new(OpaDataPutTool::new()),
            Box::new(OpaPolicyListTool::new()),
            Box::new(OpaPolicyPutTool::new()),
            Box::new(OpaHealthTool::new()),
        ]
    }
}

/// Create OPA HTTP client
fn create_opa_client() -> Result<reqwest::Client, aof_core::AofError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle common OPA error responses
fn handle_opa_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let error_msg = body
        .get("message")
        .and_then(|m| m.as_str())
        .or_else(|| body.get("error").and_then(|e| e.as_str()))
        .unwrap_or("Unknown error");

    let error_code = body
        .get("code")
        .and_then(|c| c.as_str())
        .unwrap_or("OPA_ERROR");

    match status {
        400 => ToolResult::error(format!("Bad request: {}", error_msg)),
        404 => ToolResult::error("Policy path or data not found".to_string()),
        500 => {
            // Check for policy evaluation errors
            if let Some(errors) = body.get("errors") {
                ToolResult::success(serde_json::json!({
                    "success": false,
                    "error": "evaluation error",
                    "error_code": "OPA_EVAL_ERROR",
                    "details": errors
                }))
            } else {
                ToolResult::error(format!("OPA server error: {}", error_msg))
            }
        }
        _ => ToolResult::error(format!("OPA returned status {} ({}): {}", status, error_code, error_msg)),
    }
}

// ============================================================================
// OPA Eval Tool
// ============================================================================

/// Evaluate a policy against input data
pub struct OpaEvalTool {
    config: ToolConfig,
}

impl OpaEvalTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL (e.g., http://localhost:8181)"
                },
                "path": {
                    "type": "string",
                    "description": "Policy path (e.g., data/authz/allow)"
                },
                "input": {
                    "type": "object",
                    "description": "Input data as JSON object"
                },
                "pretty": {
                    "type": "boolean",
                    "description": "Pretty print result"
                }
            }),
            vec!["endpoint", "path", "input"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_eval",
                "Evaluate a policy against input data. Returns policy decision and metrics.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaEvalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaEvalTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let input_data: serde_json::Value = input.get_arg("input")?;
        let pretty: bool = input.get_arg("pretty").unwrap_or(false);

        debug!(endpoint = %endpoint, path = %path, "Evaluating OPA policy");

        let client = create_opa_client()?;

        // Remove 'data/' prefix if present to construct API path
        let api_path = path.trim_start_matches("data/");
        let mut url = format!(
            "{}/v1/data/{}",
            endpoint.trim_end_matches('/'),
            api_path
        );

        if pretty {
            url = format!("{}?pretty=true", url);
        }

        let payload = serde_json::json!({
            "input": input_data
        });

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check OPA endpoint.",
                        e
                    )));
                } else {
                    return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
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
            return Ok(handle_opa_error(status, &body));
        }

        let result = body.get("result").cloned().unwrap_or(serde_json::json!(null));
        let decision_id = body.get("decision_id").and_then(|d| d.as_str());
        let metrics = body.get("metrics").cloned();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "result": result,
            "decision_id": decision_id,
            "metrics": metrics
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Query Tool
// ============================================================================

/// Execute an ad-hoc Rego query
pub struct OpaQueryTool {
    config: ToolConfig,
}

impl OpaQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                },
                "query": {
                    "type": "string",
                    "description": "Rego query (e.g., data.users[_].admin == true)"
                },
                "input": {
                    "type": "object",
                    "description": "Optional input data for the query"
                }
            }),
            vec!["endpoint", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_query",
                "Execute an ad-hoc Rego query against OPA's document store.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let query: String = input.get_arg("query")?;
        let input_data: Option<serde_json::Value> = input.get_arg("input").ok();

        debug!(endpoint = %endpoint, query = %query, "Executing OPA query");

        let client = create_opa_client()?;

        let url = format!("{}/v1/query", endpoint.trim_end_matches('/'));

        let mut payload = serde_json::json!({
            "query": query
        });

        if let Some(data) = input_data {
            payload["input"] = data;
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
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
            return Ok(handle_opa_error(status, &body));
        }

        let result = body.get("result").cloned().unwrap_or(serde_json::json!([]));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "result": result
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Data Get Tool
// ============================================================================

/// Get data from OPA's document store
pub struct OpaDataGetTool {
    config: ToolConfig,
}

impl OpaDataGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                },
                "path": {
                    "type": "string",
                    "description": "Data path (e.g., roles, users/alice)"
                }
            }),
            vec!["endpoint", "path"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_data_get",
                "Get data from OPA's document store. Returns stored data.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaDataGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaDataGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;

        debug!(endpoint = %endpoint, path = %path, "Getting data from OPA");

        let client = create_opa_client()?;

        let url = format!(
            "{}/v1/data/{}",
            endpoint.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
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
            return Ok(handle_opa_error(status, &body));
        }

        let result = body.get("result").cloned().unwrap_or(serde_json::json!(null));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "result": result
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Data Put Tool
// ============================================================================

/// Store data in OPA's document store
pub struct OpaDataPutTool {
    config: ToolConfig,
}

impl OpaDataPutTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                },
                "path": {
                    "type": "string",
                    "description": "Data path"
                },
                "data": {
                    "type": "object",
                    "description": "Data to store as JSON"
                }
            }),
            vec!["endpoint", "path", "data"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_data_put",
                "Store data in OPA's document store. Updates reference data for policies.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaDataPutTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaDataPutTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let data: serde_json::Value = input.get_arg("data")?;

        debug!(endpoint = %endpoint, path = %path, "Storing data in OPA");

        let client = create_opa_client()?;

        let url = format!(
            "{}/v1/data/{}",
            endpoint.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let response = match client.put(&url).json(&data).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();

        if status != 200 && status != 204 {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
            return Ok(handle_opa_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "stored": true,
            "path": path
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Policy List Tool
// ============================================================================

/// List loaded policies
pub struct OpaPolicyListTool {
    config: ToolConfig,
}

impl OpaPolicyListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                }
            }),
            vec!["endpoint"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_policy_list",
                "List all loaded policies. Returns policy IDs, paths, and source code.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaPolicyListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaPolicyListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;

        debug!(endpoint = %endpoint, "Listing OPA policies");

        let client = create_opa_client()?;

        let url = format!("{}/v1/policies", endpoint.trim_end_matches('/'));

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
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
            return Ok(handle_opa_error(status, &body));
        }

        let result = body.get("result").cloned().unwrap_or(serde_json::json!([]));

        // Transform result into a list of policies
        let mut policies = Vec::new();
        if let Some(obj) = result.as_object() {
            for (id, policy) in obj {
                policies.push(serde_json::json!({
                    "id": id,
                    "path": policy.get("path").and_then(|p| p.as_str()),
                    "raw": policy.get("raw").and_then(|r| r.as_str())
                }));
            }
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "policies": policies
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Policy Put Tool
// ============================================================================

/// Upload a new policy
pub struct OpaPolicyPutTool {
    config: ToolConfig,
}

impl OpaPolicyPutTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                },
                "policy_id": {
                    "type": "string",
                    "description": "Policy identifier"
                },
                "policy": {
                    "type": "string",
                    "description": "Rego policy source code"
                }
            }),
            vec!["endpoint", "policy_id", "policy"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_policy_put",
                "Upload a new policy to OPA. Creates or updates a Rego policy.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaPolicyPutTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaPolicyPutTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let policy_id: String = input.get_arg("policy_id")?;
        let policy: String = input.get_arg("policy")?;

        debug!(endpoint = %endpoint, policy_id = %policy_id, "Uploading OPA policy");

        let client = create_opa_client()?;

        let url = format!(
            "{}/v1/policies/{}",
            endpoint.trim_end_matches('/'),
            policy_id
        );

        let response = match client
            .put(&url)
            .header("Content-Type", "text/plain")
            .body(policy)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();

        if status != 200 && status != 204 {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
            return Ok(handle_opa_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "uploaded": true,
            "policy_id": policy_id
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// OPA Health Tool
// ============================================================================

/// Check OPA server health and status
pub struct OpaHealthTool {
    config: ToolConfig,
}

impl OpaHealthTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "OPA server URL"
                },
                "bundles": {
                    "type": "boolean",
                    "description": "Include bundle status"
                },
                "plugins": {
                    "type": "boolean",
                    "description": "Include plugin status"
                }
            }),
            vec!["endpoint"],
        );

        Self {
            config: tool_config_with_timeout(
                "opa_health",
                "Check OPA server health and status. Returns health status and optional bundle/plugin info.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for OpaHealthTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for OpaHealthTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let bundles: bool = input.get_arg("bundles").unwrap_or(true);
        let plugins: bool = input.get_arg("plugins").unwrap_or(true);

        debug!(endpoint = %endpoint, "Checking OPA health");

        let client = create_opa_client()?;

        let mut url = format!("{}/health", endpoint.trim_end_matches('/'));

        let mut query_params = Vec::new();
        if bundles {
            query_params.push("bundles=true");
        }
        if plugins {
            query_params.push("plugins=true");
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("OPA request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(_) => serde_json::json!({}),
        };

        let healthy = status == 200;

        let bundles_status = body.get("bundles").cloned();
        let plugins_status = body.get("plugins").cloned();

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "healthy": healthy,
            "bundles": bundles_status,
            "plugins": plugins_status
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
    fn test_opa_tools_creation() {
        let tools = OpaTools::all();
        assert_eq!(tools.len(), 7);

        let names: Vec<&str> = tools.iter().map(|t| t.config().name.as_str()).collect();
        assert!(names.contains(&"opa_eval"));
        assert!(names.contains(&"opa_query"));
        assert!(names.contains(&"opa_data_get"));
        assert!(names.contains(&"opa_data_put"));
        assert!(names.contains(&"opa_policy_list"));
        assert!(names.contains(&"opa_policy_put"));
        assert!(names.contains(&"opa_health"));
    }

    #[test]
    fn test_opa_eval_config() {
        let tool = OpaEvalTool::new();
        let config = tool.config();

        assert_eq!(config.name, "opa_eval");
        assert!(config.description.contains("policy"));
    }

    #[test]
    fn test_opa_query_config() {
        let tool = OpaQueryTool::new();
        let config = tool.config();

        assert_eq!(config.name, "opa_query");
        assert!(config.description.contains("query"));
    }

    #[test]
    fn test_opa_health_config() {
        let tool = OpaHealthTool::new();
        let config = tool.config();

        assert_eq!(config.name, "opa_health");
        assert!(config.description.contains("health"));
    }
}
