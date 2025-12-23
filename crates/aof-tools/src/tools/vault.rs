//! HashiCorp Vault Tools
//!
//! Tools for secrets management, encryption, and authentication via Vault's API.
//!
//! ## Available Tools
//!
//! - `vault_kv_get` - Read secret from KV secrets engine
//! - `vault_kv_put` - Write secret to KV secrets engine
//! - `vault_kv_list` - List secrets at a path
//! - `vault_kv_delete` - Delete a secret
//! - `vault_token_lookup` - Get information about a token
//! - `vault_transit_encrypt` - Encrypt data using Transit engine
//! - `vault_transit_decrypt` - Decrypt data using Transit engine
//! - `vault_approle_login` - Authenticate using AppRole
//!
//! ## Prerequisites
//!
//! - Requires `security` feature flag
//! - Valid Vault server endpoint and authentication token
//! - Appropriate policies for requested operations
//!
//! ## Authentication
//!
//! All tools use token authentication via the X-Vault-Token header.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, tool_config_with_timeout};

/// Collection of all Vault tools
pub struct VaultTools;

impl VaultTools {
    /// Get all Vault tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(VaultKvGetTool::new()),
            Box::new(VaultKvPutTool::new()),
            Box::new(VaultKvListTool::new()),
            Box::new(VaultKvDeleteTool::new()),
            Box::new(VaultTokenLookupTool::new()),
            Box::new(VaultTransitEncryptTool::new()),
            Box::new(VaultTransitDecryptTool::new()),
            Box::new(VaultAppRoleLoginTool::new()),
        ]
    }
}

/// Create Vault HTTP client with authentication
fn create_vault_client(
    token: &str,
    namespace: Option<&str>,
) -> Result<reqwest::Client, aof_core::AofError> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Token authentication
    headers.insert(
        "X-Vault-Token",
        reqwest::header::HeaderValue::from_str(token)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid token: {}", e)))?,
    );

    // Optional namespace header for Vault Enterprise
    if let Some(ns) = namespace {
        headers.insert(
            "X-Vault-Namespace",
            reqwest::header::HeaderValue::from_str(ns)
                .map_err(|e| aof_core::AofError::tool(format!("Invalid namespace: {}", e)))?,
        );
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

/// Handle common Vault error responses
fn handle_vault_error(status: u16, body: &serde_json::Value) -> ToolResult {
    let errors = body
        .get("errors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "Unknown error".to_string());

    match status {
        400 => ToolResult::error(format!("Bad request: {}", errors)),
        403 => ToolResult::error(format!("Permission denied: {}", errors)),
        404 => ToolResult::error("Secret not found or path does not exist".to_string()),
        429 => ToolResult::error("Rate limited. Retry after a delay.".to_string()),
        500..=599 => ToolResult::error(format!("Vault server error ({}): {}", status, errors)),
        _ => ToolResult::error(format!("Vault returned status {}: {}", status, errors)),
    }
}

// ============================================================================
// Vault KV Get Tool
// ============================================================================

/// Read secret from KV secrets engine
pub struct VaultKvGetTool {
    config: ToolConfig,
}

impl VaultKvGetTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL (e.g., https://vault.example.com)"
                },
                "path": {
                    "type": "string",
                    "description": "Secret path (e.g., secret/data/myapp/config)"
                },
                "version": {
                    "type": "integer",
                    "description": "Secret version (KV v2 only). Omit for latest."
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "path", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_kv_get",
                "Read a secret from Vault's KV secrets engine. Supports both KV v1 and v2.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultKvGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultKvGetTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let token: String = input.get_arg("token")?;
        let version: Option<i32> = input.get_arg("version").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, path = %path, "Reading secret from Vault");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let mut url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), path);

        if let Some(v) = version {
            url = format!("{}?version={}", url, v);
        }

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(ToolResult::error("Request timeout".to_string()));
                } else if e.is_connect() {
                    return Ok(ToolResult::error(format!(
                        "Connection failed: {}. Check Vault endpoint.",
                        e
                    )));
                } else {
                    return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        // Extract data and metadata
        let data = body.get("data");
        let metadata = data.and_then(|d| d.get("metadata"));
        let secret_data = data.and_then(|d| d.get("data")).unwrap_or(data.unwrap_or(&serde_json::json!(null)));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "data": secret_data,
            "metadata": metadata,
            "path": path
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault KV Put Tool
// ============================================================================

/// Write secret to KV secrets engine
pub struct VaultKvPutTool {
    config: ToolConfig,
}

impl VaultKvPutTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "path": {
                    "type": "string",
                    "description": "Secret path (e.g., secret/data/myapp/config)"
                },
                "data": {
                    "type": "object",
                    "description": "Secret data as key-value pairs"
                },
                "cas": {
                    "type": "integer",
                    "description": "Check-and-set version for optimistic concurrency"
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "path", "data", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_kv_put",
                "Write a secret to Vault's KV secrets engine. Creates or updates the secret.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultKvPutTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultKvPutTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let data: serde_json::Value = input.get_arg("data")?;
        let token: String = input.get_arg("token")?;
        let cas: Option<i32> = input.get_arg("cas").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, path = %path, "Writing secret to Vault");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), path);

        // Build payload for KV v2
        let mut payload = serde_json::json!({
            "data": data
        });

        if let Some(version) = cas {
            payload["options"] = serde_json::json!({
                "cas": version
            });
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to parse response: {}", e)));
            }
        };

        if status != 200 && status != 204 {
            return Ok(handle_vault_error(status, &body));
        }

        let version = body
            .get("data")
            .and_then(|d| d.get("version"))
            .and_then(|v| v.as_i64());

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "written": true,
            "path": path,
            "version": version
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault KV List Tool
// ============================================================================

/// List secrets at a path
pub struct VaultKvListTool {
    config: ToolConfig,
}

impl VaultKvListTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "path": {
                    "type": "string",
                    "description": "Path to list (e.g., secret/metadata/myapp)"
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "path", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_kv_list",
                "List secret keys at a path. Returns directory-like listing of secrets.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultKvListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultKvListTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let token: String = input.get_arg("token")?;
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, path = %path, "Listing secrets in Vault");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), path);

        // LIST method is a GET with a query parameter
        let response = match client
            .request(reqwest::Method::from_bytes(b"LIST").unwrap_or(reqwest::Method::GET), &url)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        let keys = body
            .get("data")
            .and_then(|d| d.get("keys"))
            .cloned()
            .unwrap_or(serde_json::json!([]));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "keys": keys,
            "path": path
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault KV Delete Tool
// ============================================================================

/// Delete a secret
pub struct VaultKvDeleteTool {
    config: ToolConfig,
}

impl VaultKvDeleteTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "path": {
                    "type": "string",
                    "description": "Secret path to delete"
                },
                "versions": {
                    "type": "array",
                    "description": "Specific versions to delete (KV v2 only)",
                    "items": {
                        "type": "integer"
                    }
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "path", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_kv_delete",
                "Delete a secret or specific versions from Vault's KV engine.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultKvDeleteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultKvDeleteTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let path: String = input.get_arg("path")?;
        let token: String = input.get_arg("token")?;
        let versions: Option<Vec<i32>> = input.get_arg("versions").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, path = %path, "Deleting secret from Vault");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let response = if let Some(vers) = versions {
            // Delete specific versions
            let url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), path.replace("/data/", "/delete/"));
            let payload = serde_json::json!({
                "versions": vers
            });
            client.post(&url).json(&payload).send().await
        } else {
            // Delete latest version
            let url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), path);
            client.delete(&url).send().await
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
            }
        };

        let status = response.status().as_u16();

        if status != 200 && status != 204 {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
            return Ok(handle_vault_error(status, &body));
        }

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "deleted": true,
            "path": path
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault Token Lookup Tool
// ============================================================================

/// Get information about a token
pub struct VaultTokenLookupTool {
    config: ToolConfig,
}

impl VaultTokenLookupTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "token": {
                    "type": "string",
                    "description": "Token to lookup (uses auth token if not specified)"
                },
                "auth_token": {
                    "type": "string",
                    "description": "Authentication token for the request"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "auth_token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_token_lookup",
                "Get information about a Vault token including policies, TTL, and metadata.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultTokenLookupTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultTokenLookupTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let auth_token: String = input.get_arg("auth_token")?;
        let token: Option<String> = input.get_arg("token").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, "Looking up Vault token");

        let client = create_vault_client(&auth_token, namespace.as_deref())?;

        let (url, method, payload) = if let Some(t) = token {
            // Lookup specific token
            (
                format!("{}/v1/auth/token/lookup", endpoint.trim_end_matches('/')),
                "POST",
                Some(serde_json::json!({ "token": t })),
            )
        } else {
            // Lookup self
            (
                format!("{}/v1/auth/token/lookup-self", endpoint.trim_end_matches('/')),
                "GET",
                None,
            )
        };

        let response = if method == "POST" {
            client.post(&url).json(&payload.unwrap()).send().await
        } else {
            client.get(&url).send().await
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        let data = body.get("data").cloned().unwrap_or(serde_json::json!({}));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "accessor": data.get("accessor"),
            "policies": data.get("policies"),
            "ttl": data.get("ttl"),
            "renewable": data.get("renewable"),
            "creation_time": data.get("creation_time"),
            "expire_time": data.get("expire_time"),
            "display_name": data.get("display_name"),
            "metadata": data.get("meta")
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault Transit Encrypt Tool
// ============================================================================

/// Encrypt data using Transit engine
pub struct VaultTransitEncryptTool {
    config: ToolConfig,
}

impl VaultTransitEncryptTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "key_name": {
                    "type": "string",
                    "description": "Name of the encryption key"
                },
                "plaintext": {
                    "type": "string",
                    "description": "Data to encrypt (will be base64 encoded automatically)"
                },
                "context": {
                    "type": "string",
                    "description": "Base64-encoded context for key derivation (required for derived keys)"
                },
                "key_version": {
                    "type": "integer",
                    "description": "Specific key version to use"
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                },
                "mount": {
                    "type": "string",
                    "description": "Transit mount path",
                    "default": "transit"
                }
            }),
            vec!["endpoint", "key_name", "plaintext", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_transit_encrypt",
                "Encrypt data using Vault's Transit secrets engine. Returns ciphertext.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultTransitEncryptTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultTransitEncryptTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let key_name: String = input.get_arg("key_name")?;
        let plaintext: String = input.get_arg("plaintext")?;
        let token: String = input.get_arg("token")?;
        let context: Option<String> = input.get_arg("context").ok();
        let key_version: Option<i32> = input.get_arg("key_version").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let mount: String = input.get_arg("mount").unwrap_or_else(|_| "transit".to_string());

        debug!(endpoint = %endpoint, key_name = %key_name, "Encrypting with Vault Transit");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let url = format!(
            "{}/v1/{}/encrypt/{}",
            endpoint.trim_end_matches('/'),
            mount,
            key_name
        );

        // Base64 encode the plaintext
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&plaintext);

        let mut payload = serde_json::json!({
            "plaintext": encoded
        });

        if let Some(ctx) = context {
            payload["context"] = serde_json::json!(ctx);
        }

        if let Some(ver) = key_version {
            payload["key_version"] = serde_json::json!(ver);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        let ciphertext = body
            .get("data")
            .and_then(|d| d.get("ciphertext"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let key_version_used = body
            .get("data")
            .and_then(|d| d.get("key_version"))
            .and_then(|v| v.as_i64());

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "ciphertext": ciphertext,
            "key_version": key_version_used
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault Transit Decrypt Tool
// ============================================================================

/// Decrypt data using Transit engine
pub struct VaultTransitDecryptTool {
    config: ToolConfig,
}

impl VaultTransitDecryptTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "key_name": {
                    "type": "string",
                    "description": "Name of the encryption key"
                },
                "ciphertext": {
                    "type": "string",
                    "description": "Vault-encrypted ciphertext (vault:v1:...)"
                },
                "context": {
                    "type": "string",
                    "description": "Base64-encoded context for key derivation"
                },
                "token": {
                    "type": "string",
                    "description": "Vault authentication token"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                },
                "mount": {
                    "type": "string",
                    "description": "Transit mount path",
                    "default": "transit"
                }
            }),
            vec!["endpoint", "key_name", "ciphertext", "token"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_transit_decrypt",
                "Decrypt ciphertext using Vault's Transit secrets engine. Returns plaintext.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultTransitDecryptTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultTransitDecryptTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let key_name: String = input.get_arg("key_name")?;
        let ciphertext: String = input.get_arg("ciphertext")?;
        let token: String = input.get_arg("token")?;
        let context: Option<String> = input.get_arg("context").ok();
        let namespace: Option<String> = input.get_arg("namespace").ok();
        let mount: String = input.get_arg("mount").unwrap_or_else(|_| "transit".to_string());

        debug!(endpoint = %endpoint, key_name = %key_name, "Decrypting with Vault Transit");

        let client = create_vault_client(&token, namespace.as_deref())?;

        let url = format!(
            "{}/v1/{}/decrypt/{}",
            endpoint.trim_end_matches('/'),
            mount,
            key_name
        );

        let mut payload = serde_json::json!({
            "ciphertext": ciphertext
        });

        if let Some(ctx) = context {
            payload["context"] = serde_json::json!(ctx);
        }

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        let plaintext_b64 = body
            .get("data")
            .and_then(|d| d.get("plaintext"))
            .and_then(|p| p.as_str())
            .unwrap_or("");

        // Decode base64
        use base64::Engine;
        let plaintext = match base64::engine::general_purpose::STANDARD.decode(plaintext_b64) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to decode plaintext: {}", e)));
            }
        };

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "plaintext": plaintext
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Vault AppRole Login Tool
// ============================================================================

/// Authenticate using AppRole
pub struct VaultAppRoleLoginTool {
    config: ToolConfig,
}

impl VaultAppRoleLoginTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "endpoint": {
                    "type": "string",
                    "description": "Vault server URL"
                },
                "role_id": {
                    "type": "string",
                    "description": "AppRole role ID"
                },
                "secret_id": {
                    "type": "string",
                    "description": "AppRole secret ID"
                },
                "mount": {
                    "type": "string",
                    "description": "AppRole auth mount path",
                    "default": "approle"
                },
                "namespace": {
                    "type": "string",
                    "description": "Vault namespace (Enterprise only)"
                }
            }),
            vec!["endpoint", "role_id", "secret_id"],
        );

        Self {
            config: tool_config_with_timeout(
                "vault_approle_login",
                "Authenticate to Vault using AppRole method. Returns a client token.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for VaultAppRoleLoginTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for VaultAppRoleLoginTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let endpoint: String = input.get_arg("endpoint")?;
        let role_id: String = input.get_arg("role_id")?;
        let secret_id: String = input.get_arg("secret_id")?;
        let mount: String = input.get_arg("mount").unwrap_or_else(|_| "approle".to_string());
        let namespace: Option<String> = input.get_arg("namespace").ok();

        debug!(endpoint = %endpoint, "Authenticating with Vault AppRole");

        // AppRole login doesn't require a token
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(ns) = &namespace {
            headers.insert(
                "X-Vault-Namespace",
                reqwest::header::HeaderValue::from_str(ns)
                    .map_err(|e| aof_core::AofError::tool(format!("Invalid namespace: {}", e)))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!(
            "{}/v1/auth/{}/login",
            endpoint.trim_end_matches('/'),
            mount
        );

        let payload = serde_json::json!({
            "role_id": role_id,
            "secret_id": secret_id
        });

        let response = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(format!("Vault request failed: {}", e)));
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
            return Ok(handle_vault_error(status, &body));
        }

        let auth = body.get("auth").cloned().unwrap_or(serde_json::json!({}));

        Ok(ToolResult::success(serde_json::json!({
            "success": true,
            "client_token": auth.get("client_token"),
            "accessor": auth.get("accessor"),
            "policies": auth.get("policies"),
            "token_policies": auth.get("token_policies"),
            "lease_duration": auth.get("lease_duration"),
            "renewable": auth.get("renewable")
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
    fn test_vault_tools_creation() {
        let tools = VaultTools::all();
        assert_eq!(tools.len(), 8);

        let names: Vec<&str> = tools.iter().map(|t| t.config().name.as_str()).collect();
        assert!(names.contains(&"vault_kv_get"));
        assert!(names.contains(&"vault_kv_put"));
        assert!(names.contains(&"vault_kv_list"));
        assert!(names.contains(&"vault_kv_delete"));
        assert!(names.contains(&"vault_token_lookup"));
        assert!(names.contains(&"vault_transit_encrypt"));
        assert!(names.contains(&"vault_transit_decrypt"));
        assert!(names.contains(&"vault_approle_login"));
    }

    #[test]
    fn test_kv_get_config() {
        let tool = VaultKvGetTool::new();
        let config = tool.config();

        assert_eq!(config.name, "vault_kv_get");
        assert!(config.description.contains("KV"));
    }

    #[test]
    fn test_transit_encrypt_config() {
        let tool = VaultTransitEncryptTool::new();
        let config = tool.config();

        assert_eq!(config.name, "vault_transit_encrypt");
        assert!(config.description.contains("Transit"));
    }
}
