//! Observability Tools
//!
//! Tools for querying and interacting with observability platforms.
//!
//! ## Available Tools
//!
//! - `prometheus_query` - Query Prometheus metrics
//! - `loki_query` - Query Loki logs
//! - `elasticsearch_query` - Query Elasticsearch/OpenSearch
//! - `victoriametrics_query` - Query VictoriaMetrics
//!
//! ## Prerequisites
//!
//! - Requires `observability` feature flag
//! - Valid endpoint URLs and credentials
//!
//! ## MCP Alternative
//!
//! For MCP-based observability, use dedicated MCP servers for each platform.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all observability tools
pub struct ObservabilityTools;

impl ObservabilityTools {
    /// Get all observability tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(PrometheusQueryTool::new()),
            Box::new(LokiQueryTool::new()),
            Box::new(ElasticsearchQueryTool::new()),
            Box::new(VictoriaMetricsQueryTool::new()),
        ]
    }
}

// ============================================================================
// Prometheus Query Tool
// ============================================================================

/// Query Prometheus metrics
pub struct PrometheusQueryTool {
    config: ToolConfig,
}

impl PrometheusQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Prometheus server URL (e.g., http://prometheus:9090)"
                },
                "query": {
                    "type": "string",
                    "description": "PromQL query"
                },
                "time": {
                    "type": "string",
                    "description": "Evaluation timestamp (RFC3339 or Unix timestamp)"
                },
                "start": {
                    "type": "string",
                    "description": "Range query start time"
                },
                "end": {
                    "type": "string",
                    "description": "Range query end time"
                },
                "step": {
                    "type": "string",
                    "description": "Range query step (e.g., '15s', '1m')",
                    "default": "15s"
                },
                "timeout": {
                    "type": "string",
                    "description": "Query timeout",
                    "default": "30s"
                }
            }),
            vec!["endpoint", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "prometheus_query",
                "Query Prometheus metrics using PromQL. Supports instant and range queries.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for PrometheusQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for PrometheusQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let query: String = input.get_arg("query")?;
        let time: Option<String> = input.get_arg("time").ok();
        let start: Option<String> = input.get_arg("start").ok();
        let end: Option<String> = input.get_arg("end").ok();
        let step: String = input.get_arg("step").unwrap_or_else(|_| "15s".to_string());
        let timeout: String = input.get_arg("timeout").unwrap_or_else(|_| "30s".to_string());

        debug!(endpoint = %endpoint, query = %query, "Querying Prometheus");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        // Determine if range query or instant query
        let (url, params) = if start.is_some() && end.is_some() {
            let url = format!("{}/api/v1/query_range", endpoint.trim_end_matches('/'));
            let mut params = vec![
                ("query", query.clone()),
                ("start", start.unwrap()),
                ("end", end.unwrap()),
                ("step", step),
                ("timeout", timeout),
            ];
            (url, params)
        } else {
            let url = format!("{}/api/v1/query", endpoint.trim_end_matches('/'));
            let mut params = vec![
                ("query", query.clone()),
                ("timeout", timeout),
            ];
            if let Some(t) = time {
                params.push(("time", t));
            }
            (url, params)
        };

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Prometheus query failed: {}", e)));
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
                "Prometheus returned status {}: {:?}",
                status,
                body.get("error")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "status": body.get("status"),
            "data": body.get("data"),
            "query": query
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Loki Query Tool
// ============================================================================

/// Query Loki logs
pub struct LokiQueryTool {
    config: ToolConfig,
}

impl LokiQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Loki server URL (e.g., http://loki:3100)"
                },
                "query": {
                    "type": "string",
                    "description": "LogQL query"
                },
                "start": {
                    "type": "string",
                    "description": "Start time (RFC3339 or Unix nanoseconds)"
                },
                "end": {
                    "type": "string",
                    "description": "End time (RFC3339 or Unix nanoseconds)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of entries",
                    "default": 100
                },
                "direction": {
                    "type": "string",
                    "description": "Query direction: forward or backward",
                    "enum": ["forward", "backward"],
                    "default": "backward"
                }
            }),
            vec!["endpoint", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "loki_query",
                "Query Loki logs using LogQL. Returns log entries matching the query.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for LokiQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for LokiQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let query: String = input.get_arg("query")?;
        let start: Option<String> = input.get_arg("start").ok();
        let end: Option<String> = input.get_arg("end").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(100);
        let direction: String = input.get_arg("direction").unwrap_or_else(|_| "backward".to_string());

        debug!(endpoint = %endpoint, query = %query, "Querying Loki");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!("{}/loki/api/v1/query_range", endpoint.trim_end_matches('/'));

        let mut params = vec![
            ("query", query.clone()),
            ("limit", limit.to_string()),
            ("direction", direction),
        ];

        if let Some(s) = start {
            params.push(("start", s));
        }
        if let Some(e) = end {
            params.push(("end", e));
        }

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Loki query failed: {}", e)));
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
                "Loki returned status {}: {:?}",
                status,
                body.get("message")
            )));
        }

        // Extract log entries count
        let result_count = body
            .get("data")
            .and_then(|d| d.get("result"))
            .and_then(|r| r.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "status": body.get("status"),
            "data": body.get("data"),
            "query": query,
            "result_count": result_count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Elasticsearch Query Tool
// ============================================================================

/// Query Elasticsearch/OpenSearch
pub struct ElasticsearchQueryTool {
    config: ToolConfig,
}

impl ElasticsearchQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Elasticsearch/OpenSearch URL (e.g., http://elasticsearch:9200)"
                },
                "index": {
                    "type": "string",
                    "description": "Index name or pattern (e.g., 'logs-*')"
                },
                "query": {
                    "type": "object",
                    "description": "Elasticsearch query DSL"
                },
                "size": {
                    "type": "integer",
                    "description": "Number of results to return",
                    "default": 10
                },
                "from": {
                    "type": "integer",
                    "description": "Starting offset",
                    "default": 0
                },
                "sort": {
                    "type": "array",
                    "description": "Sort specification",
                    "items": { "type": "object" }
                },
                "source": {
                    "type": "array",
                    "description": "Fields to include in response",
                    "items": { "type": "string" }
                },
                "auth": {
                    "type": "object",
                    "description": "Authentication (username, password)",
                    "properties": {
                        "username": { "type": "string" },
                        "password": { "type": "string" }
                    }
                }
            }),
            vec!["endpoint", "index"],
        );

        Self {
            config: tool_config_with_timeout(
                "elasticsearch_query",
                "Query Elasticsearch or OpenSearch. Supports full Query DSL.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ElasticsearchQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ElasticsearchQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let index: String = input.get_arg("index")?;
        let query: Option<serde_json::Value> = input.get_arg("query").ok();
        let size: i32 = input.get_arg("size").unwrap_or(10);
        let from: i32 = input.get_arg("from").unwrap_or(0);
        let sort: Option<Vec<serde_json::Value>> = input.get_arg("sort").ok();
        let source: Option<Vec<String>> = input.get_arg("source").ok();
        let auth: Option<HashMap<String, String>> = input.get_arg("auth").ok();

        debug!(endpoint = %endpoint, index = %index, "Querying Elasticsearch");

        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60));

        let client = client_builder
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!("{}/{}/_search", endpoint.trim_end_matches('/'), index);

        // Build search body
        let mut body = serde_json::json!({
            "size": size,
            "from": from
        });

        if let Some(q) = query {
            body["query"] = q;
        } else {
            body["query"] = serde_json::json!({ "match_all": {} });
        }

        if let Some(s) = sort {
            body["sort"] = serde_json::json!(s);
        }

        if let Some(src) = source {
            body["_source"] = serde_json::json!(src);
        }

        let mut request = client.post(&url).json(&body);

        if let Some(auth_info) = auth {
            if let (Some(user), Some(pass)) = (auth_info.get("username"), auth_info.get("password")) {
                request = request.basic_auth(user, Some(pass));
            }
        }

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Elasticsearch query failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let resp_body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status >= 400 {
            return Ok(ToolResult::error(format!(
                "Elasticsearch returned status {}: {:?}",
                status,
                resp_body.get("error")
            )));
        }

        let hits = resp_body
            .get("hits")
            .and_then(|h| h.get("total"))
            .and_then(|t| t.get("value").or(Some(t)))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(ToolResult::success(serde_json::json!({
            "hits": resp_body.get("hits"),
            "took": resp_body.get("took"),
            "total_hits": hits,
            "index": index
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// VictoriaMetrics Query Tool
// ============================================================================

/// Query VictoriaMetrics
pub struct VictoriaMetricsQueryTool {
    config: ToolConfig,
}

impl VictoriaMetricsQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "VictoriaMetrics URL (e.g., http://victoriametrics:8428)"
                },
                "query": {
                    "type": "string",
                    "description": "MetricsQL query (PromQL compatible)"
                },
                "time": {
                    "type": "string",
                    "description": "Evaluation timestamp"
                },
                "start": {
                    "type": "string",
                    "description": "Range query start time"
                },
                "end": {
                    "type": "string",
                    "description": "Range query end time"
                },
                "step": {
                    "type": "string",
                    "description": "Range query step",
                    "default": "15s"
                }
            }),
            vec!["endpoint", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "victoriametrics_query",
                "Query VictoriaMetrics using MetricsQL. Compatible with PromQL.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for VictoriaMetricsQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VictoriaMetricsQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let query: String = input.get_arg("query")?;
        let time: Option<String> = input.get_arg("time").ok();
        let start: Option<String> = input.get_arg("start").ok();
        let end: Option<String> = input.get_arg("end").ok();
        let step: String = input.get_arg("step").unwrap_or_else(|_| "15s".to_string());

        debug!(endpoint = %endpoint, query = %query, "Querying VictoriaMetrics");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        // VictoriaMetrics uses same API as Prometheus
        let (url, params): (String, Vec<(&str, String)>) = if start.is_some() && end.is_some() {
            let url = format!("{}/api/v1/query_range", endpoint.trim_end_matches('/'));
            let params = vec![
                ("query", query.clone()),
                ("start", start.unwrap()),
                ("end", end.unwrap()),
                ("step", step),
            ];
            (url, params)
        } else {
            let url = format!("{}/api/v1/query", endpoint.trim_end_matches('/'));
            let mut params = vec![("query", query.clone())];
            if let Some(t) = time {
                params.push(("time", t));
            }
            (url, params)
        };

        let response = match client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("VictoriaMetrics query failed: {}", e)));
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
                "VictoriaMetrics returned status {}: {:?}",
                status,
                body.get("error")
            )));
        }

        Ok(ToolResult::success(serde_json::json!({
            "status": body.get("status"),
            "data": body.get("data"),
            "query": query
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
