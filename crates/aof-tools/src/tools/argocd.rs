//! ArgoCD Tools
//!
//! Tools for interacting with ArgoCD's REST API for GitOps continuous delivery.
//!
//! ## Available Tools
//!
//! - `argocd_app_list` - List all applications across projects
//! - `argocd_app_get` - Get detailed application status
//! - `argocd_app_sync` - Trigger application synchronization
//! - `argocd_app_rollback` - Rollback to previous deployment
//! - `argocd_app_history` - Get sync history
//! - `argocd_app_diff` - Show diff between Git and live state
//!
//! ## Prerequisites
//!
//! - Requires `cicd` feature flag
//! - Valid ArgoCD endpoint and JWT token
//! - Appropriate RBAC permissions
//!
//! ## Authentication
//!
//! All tools use JWT Bearer token authentication.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all ArgoCD tools
pub struct ArgoCDTools;

impl ArgoCDTools {
    /// Get all ArgoCD tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(ArgoCDAppListTool::new()),
            Box::new(ArgoCDAppGetTool::new()),
            Box::new(ArgoCDAppSyncTool::new()),
            Box::new(ArgoCDAppRollbackTool::new()),
            Box::new(ArgoCDAppHistoryTool::new()),
            Box::new(ArgoCDAppDiffTool::new()),
        ]
    }
}

/// Create ArgoCD HTTP client with JWT authentication
fn create_argocd_client(
    token: &str,
    verify_tls: bool,
) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Bearer token authentication
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(!verify_tls)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle ArgoCD API error responses
fn handle_argocd_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let message = body
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown error");

    match status {
        401 => ToolResult::error(
            "Authentication failed. Token may be invalid or expired. \
             Generate a new token using: argocd account generate-token"
                .to_string(),
        ),
        403 => ToolResult::error(format!(
            "Authorization failed: {}. \
             Check that the token has appropriate project permissions.",
            message
        )),
        404 => ToolResult::error(format!(
            "Application not found. Verify the application name and project. \
             List available apps: argocd app list. Details: {}",
            message
        )),
        409 => ToolResult::error(
            "Another operation is in progress for this application. \
             Wait for the current operation to complete and retry."
                .to_string(),
        ),
        429 => ToolResult::error("Rate limited. Retry after cooldown period.".to_string()),
        500..=599 => ToolResult::error(format!("ArgoCD server error: {}", message)),
        _ => ToolResult::error(format!("ArgoCD returned status {}: {}", status, message)),
    }
}

// ============================================================================
// ArgoCD App List Tool
// ============================================================================

/// List all ArgoCD applications
pub struct ArgoCDAppListTool {
    config: ToolConfig,
}

impl ArgoCDAppListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL (e.g., https://argocd.example.com)"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "project": {
                    "type": "string",
                    "description": "Filter by project name"
                },
                "selector": {
                    "type": "string",
                    "description": "Label selector (e.g., 'app=nginx,env=prod')"
                },
                "repo": {
                    "type": "string",
                    "description": "Filter by repository URL"
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_list",
                "List all ArgoCD applications across projects. Supports filtering by project, labels, and repository.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for ArgoCDAppListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let project: Option<String> = input.get_arg("project").ok();
        let selector: Option<String> = input.get_arg("selector").ok();
        let repo: Option<String> = input.get_arg("repo").ok();
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, "Listing ArgoCD applications");

        let client = create_argocd_client(&token, verify_tls)?;

        let url = format!("{}/api/v1/applications", endpoint.trim_end_matches('/'));

        let mut params: Vec<(&str, String)> = vec![];

        if let Some(p) = project {
            params.push(("project", p));
        }
        if let Some(s) = selector {
            params.push(("selector", s));
        }
        if let Some(r) = repo {
            params.push(("repo", r));
        }

        let response = match client.get(&url).query(&params).send().await {
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
                    return Ok(ToolResult::error(format!(
                        "ArgoCD request failed: {}",
                        e
                    )));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if status != 200 {
            return Ok(handle_argocd_error(status, &body));
        }

        // Extract applications from response
        let items = body.get("items").cloned().unwrap_or(serde_json::json!([]));
        let count = items.as_array().map(|a| a.len()).unwrap_or(0);

        // Simplify response for LLM consumption
        let applications: Vec<serde_json::Value> = items
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|app| {
                serde_json::json!({
                    "name": app.get("metadata").and_then(|m| m.get("name")),
                    "project": app.get("spec").and_then(|s| s.get("project")),
                    "namespace": app.get("spec").and_then(|s| s.get("destination")).and_then(|d| d.get("namespace")),
                    "server": app.get("spec").and_then(|s| s.get("destination")).and_then(|d| d.get("server")),
                    "source": {
                        "repoURL": app.get("spec").and_then(|s| s.get("source")).and_then(|src| src.get("repoURL")),
                        "path": app.get("spec").and_then(|s| s.get("source")).and_then(|src| src.get("path")),
                        "targetRevision": app.get("spec").and_then(|s| s.get("source")).and_then(|src| src.get("targetRevision"))
                    },
                    "status": {
                        "sync": app.get("status").and_then(|s| s.get("sync")).and_then(|sync| sync.get("status")),
                        "health": app.get("status").and_then(|s| s.get("health")).and_then(|h| h.get("status")),
                        "revision": app.get("status").and_then(|s| s.get("sync")).and_then(|sync| sync.get("revision"))
                    }
                })
            })
            .collect();

        Ok(ToolResult::success(serde_json::json!({
            "applications": applications,
            "count": count
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// ArgoCD App Get Tool
// ============================================================================

/// Get detailed status of an ArgoCD application
pub struct ArgoCDAppGetTool {
    config: ToolConfig,
}

impl ArgoCDAppGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for validation"
                },
                "refresh": {
                    "type": "string",
                    "description": "Refresh mode: 'normal' (from K8s) or 'hard' (from Git)",
                    "enum": ["normal", "hard"]
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token", "app_name"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_get",
                "Get detailed information about an ArgoCD application including health, sync status, and resources.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for ArgoCDAppGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let app_name: String = input.get_arg("app_name")?;
        let refresh: Option<String> = input.get_arg("refresh").ok();
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, app_name = %app_name, "Getting ArgoCD application");

        let client = create_argocd_client(&token, verify_tls)?;

        let mut url = format!(
            "{}/api/v1/applications/{}",
            endpoint.trim_end_matches('/'),
            app_name
        );

        // Add refresh parameter if specified
        if let Some(ref_mode) = refresh {
            url = format!("{}?refresh={}", url, ref_mode);
        }

        let response = match client.get(&url).send().await {
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
                    return Ok(ToolResult::error(format!(
                        "ArgoCD request failed: {}",
                        e
                    )));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if status != 200 {
            return Ok(handle_argocd_error(status, &body));
        }

        // Extract key information
        let metadata = body.get("metadata").cloned().unwrap_or_default();
        let spec = body.get("spec").cloned().unwrap_or_default();
        let app_status = body.get("status").cloned().unwrap_or_default();

        // Get resources list
        let resources: Vec<serde_json::Value> = app_status
            .get("resources")
            .and_then(|r| r.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|res| {
                serde_json::json!({
                    "kind": res.get("kind"),
                    "name": res.get("name"),
                    "namespace": res.get("namespace"),
                    "status": res.get("status"),
                    "health": res.get("health").and_then(|h| h.get("status"))
                })
            })
            .collect();

        Ok(ToolResult::success(serde_json::json!({
            "metadata": {
                "name": metadata.get("name"),
                "namespace": metadata.get("namespace"),
                "creationTimestamp": metadata.get("creationTimestamp")
            },
            "spec": {
                "project": spec.get("project"),
                "source": spec.get("source"),
                "destination": spec.get("destination"),
                "syncPolicy": spec.get("syncPolicy")
            },
            "status": {
                "sync": app_status.get("sync"),
                "health": app_status.get("health"),
                "operationState": app_status.get("operationState"),
                "conditions": app_status.get("conditions")
            },
            "resources": resources
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// ArgoCD App Sync Tool
// ============================================================================

/// Trigger ArgoCD application synchronization
pub struct ArgoCDAppSyncTool {
    config: ToolConfig,
}

impl ArgoCDAppSyncTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name"
                },
                "revision": {
                    "type": "string",
                    "description": "Git revision to sync (commit SHA, branch, tag)"
                },
                "prune": {
                    "type": "boolean",
                    "description": "Remove resources not in Git",
                    "default": false
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview sync without applying changes",
                    "default": false
                },
                "resources": {
                    "type": "array",
                    "description": "Specific resources to sync (selective sync)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "group": { "type": "string" },
                            "kind": { "type": "string" },
                            "name": { "type": "string" },
                            "namespace": { "type": "string" }
                        }
                    }
                },
                "sync_options": {
                    "type": "array",
                    "description": "Sync options (e.g., ['Validate=false', 'CreateNamespace=true'])",
                    "items": { "type": "string" }
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token", "app_name"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_sync",
                "Trigger manual synchronization of an ArgoCD application. Supports selective sync and dry run mode.",
                parameters,
                120, // Longer timeout for sync operations
            ),
        }
    }
}

impl Default for ArgoCDAppSyncTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppSyncTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let app_name: String = input.get_arg("app_name")?;
        let revision: Option<String> = input.get_arg("revision").ok();
        let prune: bool = input.get_arg("prune").unwrap_or(false);
        let dry_run: bool = input.get_arg("dry_run").unwrap_or(false);
        let resources: Option<Vec<serde_json::Value>> = input.get_arg("resources").ok();
        let sync_options: Option<Vec<String>> = input.get_arg("sync_options").ok();
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, app_name = %app_name, "Syncing ArgoCD application");

        let client = create_argocd_client(&token, verify_tls)?;

        let url = format!(
            "{}/api/v1/applications/{}/sync",
            endpoint.trim_end_matches('/'),
            app_name
        );

        // Build sync request payload
        let mut payload = serde_json::json!({
            "prune": prune,
            "dryRun": dry_run
        });

        if let Some(rev) = revision {
            payload["revision"] = serde_json::json!(rev);
        }

        if let Some(res) = resources {
            payload["resources"] = serde_json::json!(res);
        }

        if let Some(opts) = sync_options {
            payload["syncOptions"] = serde_json::json!({"items": opts});
        }

        let response = match client.post(&url).json(&payload).send().await {
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
                    return Ok(ToolResult::error(format!("ArgoCD sync failed: {}", e)));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if status != 200 {
            return Ok(handle_argocd_error(status, &body));
        }

        let operation = body.get("status").and_then(|s| s.get("operationState"));

        Ok(ToolResult::success(serde_json::json!({
            "app_name": app_name,
            "status": "sync_initiated",
            "dry_run": dry_run,
            "prune": prune,
            "operation": operation
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// ArgoCD App Rollback Tool
// ============================================================================

/// Rollback ArgoCD application to previous deployment
pub struct ArgoCDAppRollbackTool {
    config: ToolConfig,
}

impl ArgoCDAppRollbackTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name"
                },
                "revision": {
                    "type": "string",
                    "description": "Deployment ID to rollback to (from sync history)"
                },
                "prune": {
                    "type": "boolean",
                    "description": "Remove resources not in target revision",
                    "default": false
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview rollback without applying",
                    "default": false
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token", "app_name", "revision"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_rollback",
                "Rollback an ArgoCD application to a previous deployment. Requires auto-sync to be disabled.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for ArgoCDAppRollbackTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppRollbackTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let app_name: String = input.get_arg("app_name")?;
        let revision: String = input.get_arg("revision")?;
        let prune: bool = input.get_arg("prune").unwrap_or(false);
        let dry_run: bool = input.get_arg("dry_run").unwrap_or(false);
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, app_name = %app_name, revision = %revision, "Rolling back ArgoCD application");

        let client = create_argocd_client(&token, verify_tls)?;

        // First, check if auto-sync is enabled
        let get_url = format!(
            "{}/api/v1/applications/{}",
            endpoint.trim_end_matches('/'),
            app_name
        );

        let get_response = match client.get(&get_url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to get application: {}",
                    e
                )));
            }
        };

        let get_status = get_response.status().as_u16();
        let app_data: serde_json::Value = match get_response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if get_status != 200 {
            return Ok(handle_argocd_error(get_status, &app_data));
        }

        // Check for auto-sync
        let has_auto_sync = app_data
            .get("spec")
            .and_then(|s| s.get("syncPolicy"))
            .and_then(|sp| sp.get("automated"))
            .is_some();

        if has_auto_sync {
            return Ok(ToolResult::error(
                "Rollback unavailable: automated sync is enabled. \
                 Disable auto-sync first using: argocd app set <app> --sync-policy none"
                    .to_string(),
            ));
        }

        // Perform rollback
        let url = format!(
            "{}/api/v1/applications/{}/rollback",
            endpoint.trim_end_matches('/'),
            app_name
        );

        let payload = serde_json::json!({
            "id": revision.parse::<i64>().unwrap_or(0),
            "prune": prune,
            "dryRun": dry_run
        });

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!(
                        "ArgoCD rollback failed: {}",
                        e
                    )));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if status != 200 {
            return Ok(handle_argocd_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "app_name": app_name,
            "status": "rollback_initiated",
            "target_revision": revision,
            "dry_run": dry_run,
            "prune": prune
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// ArgoCD App History Tool
// ============================================================================

/// Get ArgoCD application sync history
pub struct ArgoCDAppHistoryTool {
    config: ToolConfig,
}

impl ArgoCDAppHistoryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name"
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token", "app_name"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_history",
                "Get synchronization history for an ArgoCD application. Shows past deployments and revisions.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for ArgoCDAppHistoryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppHistoryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let app_name: String = input.get_arg("app_name")?;
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, app_name = %app_name, "Getting ArgoCD application history");

        let client = create_argocd_client(&token, verify_tls)?;

        let url = format!(
            "{}/api/v1/applications/{}",
            endpoint.trim_end_matches('/'),
            app_name
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else {
                    return Ok(ToolResult::error(format!(
                        "ArgoCD request failed: {}",
                        e
                    )));
                }
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        if status != 200 {
            return Ok(handle_argocd_error(status, &body));
        }

        // Extract history from status
        let history = body
            .get("status")
            .and_then(|s| s.get("history"))
            .cloned()
            .unwrap_or(serde_json::json!([]));

        let history_entries: Vec<serde_json::Value> = history
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "id": entry.get("id"),
                    "revision": entry.get("revision"),
                    "deployedAt": entry.get("deployedAt"),
                    "deployStartedAt": entry.get("deployStartedAt"),
                    "source": entry.get("source")
                })
            })
            .collect();

        Ok(ToolResult::success(serde_json::json!({
            "app_name": app_name,
            "history": history_entries,
            "count": history_entries.len()
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// ArgoCD App Diff Tool
// ============================================================================

/// Show diff between Git and live state
pub struct ArgoCDAppDiffTool {
    config: ToolConfig,
}

impl ArgoCDAppDiffTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "ArgoCD server URL"
                },
                "token": {
                    "type": "string",
                    "description": "JWT authentication token"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name"
                },
                "revision": {
                    "type": "string",
                    "description": "Git revision to compare against"
                },
                "verify_tls": {
                    "type": "boolean",
                    "description": "Verify TLS certificates",
                    "default": true
                }
            }),
            vec!["endpoint", "token", "app_name"],
        );

        Self {
            config: tool_config_with_timeout(
                "argocd_app_diff",
                "Show differences between desired state (Git) and live state (Kubernetes cluster).",
                parameters,
                60,
            ),
        }
    }
}

impl Default for ArgoCDAppDiffTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ArgoCDAppDiffTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let token: String = input.get_arg("token")?;
        let app_name: String = input.get_arg("app_name")?;
        let revision: Option<String> = input.get_arg("revision").ok();
        let verify_tls: bool = input.get_arg("verify_tls").unwrap_or(true);

        debug!(endpoint = %endpoint, app_name = %app_name, "Getting ArgoCD application diff");

        let client = create_argocd_client(&token, verify_tls)?;

        // Get manifests (desired state)
        let manifests_url = format!(
            "{}/api/v1/applications/{}/manifests",
            endpoint.trim_end_matches('/'),
            app_name
        );

        let mut params: Vec<(&str, String)> = vec![];
        if let Some(rev) = &revision {
            params.push(("revision", rev.clone()));
        }

        let manifests_response = match client.get(&manifests_url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to get manifests: {}",
                    e
                )));
            }
        };

        let manifests_status = manifests_response.status().as_u16();
        let manifests_body: serde_json::Value = match manifests_response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse manifests: {}",
                    e
                )));
            }
        };

        if manifests_status != 200 {
            return Ok(handle_argocd_error(manifests_status, &manifests_body));
        }

        // Get application status (live state info)
        let app_url = format!(
            "{}/api/v1/applications/{}",
            endpoint.trim_end_matches('/'),
            app_name
        );

        let app_response = match client.get(&app_url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to get application: {}",
                    e
                )));
            }
        };

        let app_status = app_response.status().as_u16();
        let app_body: serde_json::Value = match app_response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to parse application: {}",
                    e
                )));
            }
        };

        if app_status != 200 {
            return Ok(handle_argocd_error(app_status, &app_body));
        }

        // Extract sync status which contains diff info
        let sync_status = app_body
            .get("status")
            .and_then(|s| s.get("sync"))
            .cloned()
            .unwrap_or_default();

        let resources = app_body
            .get("status")
            .and_then(|s| s.get("resources"))
            .cloned()
            .unwrap_or(serde_json::json!([]));

        // Categorize resources by sync status
        let mut in_sync = vec![];
        let mut out_of_sync = vec![];

        if let Some(res_array) = resources.as_array() {
            for res in res_array {
                let res_status = res
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown");

                let resource_summary = serde_json::json!({
                    "kind": res.get("kind"),
                    "name": res.get("name"),
                    "namespace": res.get("namespace"),
                    "status": res_status
                });

                if res_status == "Synced" {
                    in_sync.push(resource_summary);
                } else {
                    out_of_sync.push(resource_summary);
                }
            }
        }

        Ok(ToolResult::success(serde_json::json!({
            "app_name": app_name,
            "sync_status": sync_status.get("status"),
            "revision": sync_status.get("revision"),
            "target_revision": revision,
            "summary": {
                "in_sync": in_sync.len(),
                "out_of_sync": out_of_sync.len()
            },
            "out_of_sync_resources": out_of_sync,
            "in_sync_resources": in_sync
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
    fn test_argocd_app_list_tool_creation() {
        let tool = ArgoCDAppListTool::new();
        assert_eq!(tool.config().name, "argocd_app_list");
    }

    #[test]
    fn test_argocd_app_get_tool_creation() {
        let tool = ArgoCDAppGetTool::new();
        assert_eq!(tool.config().name, "argocd_app_get");
    }

    #[test]
    fn test_argocd_app_sync_tool_creation() {
        let tool = ArgoCDAppSyncTool::new();
        assert_eq!(tool.config().name, "argocd_app_sync");
    }

    #[test]
    fn test_argocd_app_rollback_tool_creation() {
        let tool = ArgoCDAppRollbackTool::new();
        assert_eq!(tool.config().name, "argocd_app_rollback");
    }

    #[test]
    fn test_argocd_app_history_tool_creation() {
        let tool = ArgoCDAppHistoryTool::new();
        assert_eq!(tool.config().name, "argocd_app_history");
    }

    #[test]
    fn test_argocd_app_diff_tool_creation() {
        let tool = ArgoCDAppDiffTool::new();
        assert_eq!(tool.config().name, "argocd_app_diff");
    }

    #[test]
    fn test_handle_argocd_error_401() {
        let body = serde_json::json!({"message": "Unauthorized"});
        let result = handle_argocd_error(401, &body);
        assert!(!result.success);
    }

    #[test]
    fn test_handle_argocd_error_404() {
        let body = serde_json::json!({"message": "application not found"});
        let result = handle_argocd_error(404, &body);
        assert!(!result.success);
    }

    #[test]
    fn test_argocd_tools_all() {
        let tools = ArgoCDTools::all();
        assert_eq!(tools.len(), 6);
    }
}
