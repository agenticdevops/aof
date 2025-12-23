//! Opsgenie Webhook adapter for AOF
//!
//! This module provides integration with Opsgenie's Alert API and webhooks, supporting:
//! - Alert events (created, acknowledged, closed, escalated)
//! - Note events (added to alerts)
//! - Custom action events
//! - Integration token verification
//! - Alert management API (acknowledge, close, add notes, create alerts)
//!
//! # Design Philosophy
//!
//! This platform is designed as a **pluggable component** that can be:
//! - Enabled/disabled via Cargo feature flags
//! - Extended with custom event handlers
//! - Integrated with AgentFlow workflows for incident response automation
//!
//! # Example Usage
//!
//! ```yaml
//! apiVersion: aof.dev/v1
//! kind: AgentFlow
//! spec:
//!   trigger:
//!     type: Opsgenie
//!     config:
//!       api_url: https://api.opsgenie.com
//!       api_key_env: OPSGENIE_API_KEY
//!       webhook_token_env: OPSGENIE_WEBHOOK_TOKEN
//!       allowed_actions:
//!         - Create
//!         - AddNote
//!       priority_filter:
//!         - P1
//!         - P2
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Opsgenie platform adapter
///
/// Implements the `TriggerPlatform` trait for Opsgenie webhooks.
/// Supports alert lifecycle events and provides methods for interacting
/// with the Opsgenie API (posting notes, acknowledging, closing alerts).
pub struct OpsgeniePlatform {
    config: OpsgenieConfig,
    client: reqwest::Client,
}

/// Opsgenie configuration
///
/// All configuration options for the Opsgenie platform.
/// Supports environment variable resolution for secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsgenieConfig {
    /// Opsgenie API base URL
    /// - US: https://api.opsgenie.com
    /// - EU: https://api.eu.opsgenie.com
    /// - On-prem: https://opsgenie.yourcompany.com
    pub api_url: String,

    /// API key for Opsgenie integration
    /// Generate from: Opsgenie → Settings → Integrations → API
    pub api_key: String,

    /// Webhook verification token (optional but recommended)
    /// Used to verify webhook authenticity
    #[serde(default)]
    pub webhook_token: Option<String>,

    /// Bot/Integration name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Alert action filters (which events to process)
    /// Options: ["Create", "Acknowledge", "Close", "Escalate", "AddNote", "CustomAction"]
    /// Default: all actions if empty
    #[serde(default)]
    pub allowed_actions: Vec<String>,

    /// Priority filter (e.g., ["P1", "P2"])
    /// Only process alerts matching these priorities
    #[serde(default)]
    pub priority_filter: Option<Vec<String>>,

    /// Tag filter (e.g., ["production", "critical"])
    /// Only process alerts with ALL these tags
    #[serde(default)]
    pub tag_filter: Option<Vec<String>>,

    /// Team filter (team IDs or names)
    /// Only process alerts assigned to these teams
    #[serde(default)]
    pub team_filter: Option<Vec<String>>,

    /// Source filter (e.g., ["datadog", "prometheus"])
    /// Only process alerts from these sources
    #[serde(default)]
    pub source_filter: Option<Vec<String>>,

    /// Enable posting notes to alerts
    #[serde(default = "default_true")]
    pub enable_notes: bool,

    /// Enable updating alert fields
    #[serde(default = "default_true")]
    pub enable_updates: bool,

    /// Enable closing alerts
    #[serde(default = "default_true")]
    pub enable_close: bool,

    /// Enable acknowledging alerts
    #[serde(default = "default_true")]
    pub enable_acknowledge: bool,

    /// Enable creating new alerts
    #[serde(default = "default_true")]
    pub enable_create: bool,

    /// Integration ID for verification (optional)
    #[serde(default)]
    pub integration_id: Option<String>,

    /// Verify webhook by making API callback (adds latency but more secure)
    #[serde(default)]
    pub verify_with_api_callback: bool,

    /// Rate limit: max API calls per minute
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,

    /// HTTP timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_true() -> bool {
    true
}

fn default_rate_limit() -> u32 {
    600 // Opsgenie limit: 600 requests/minute
}

fn default_timeout() -> u64 {
    30
}

// ============================================================================
// WEBHOOK PAYLOAD TYPES
// ============================================================================

/// Opsgenie webhook payload
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpsgenieWebhookPayload {
    /// Action type: Create, Acknowledge, Close, Escalate, AddNote, CustomAction
    pub action: String,

    /// Alert details
    pub alert: OpsgenieAlert,

    /// Integration metadata
    #[serde(default)]
    pub integration_id: Option<String>,

    #[serde(default)]
    pub integration_name: Option<String>,

    /// Source of action (user, integration, schedule)
    #[serde(default)]
    pub source: Option<OpsgenieSource>,
}

/// Opsgenie alert information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpsgenieAlert {
    /// Alert UUID
    pub alert_id: String,

    /// Short numeric ID (for display)
    #[serde(default)]
    pub tiny_id: Option<String>,

    /// Alert message/title
    pub message: String,

    /// Alert description (can be very long)
    #[serde(default)]
    pub description: Option<String>,

    /// Priority: P1, P2, P3, P4, P5
    #[serde(default)]
    pub priority: Option<String>,

    /// Alert tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Source system (e.g., "datadog", "prometheus")
    #[serde(default)]
    pub source: Option<String>,

    /// Entity (resource identifier, e.g., "db-prod-01")
    #[serde(default)]
    pub entity: Option<String>,

    /// Alert alias (deduplication key)
    #[serde(default)]
    pub alias: Option<String>,

    /// Current owner (who acknowledged)
    #[serde(default)]
    pub owner: Option<String>,

    /// Responders (teams/users assigned)
    #[serde(default)]
    pub responders: Vec<OpsgenieResponder>,

    /// Custom fields (key-value pairs)
    #[serde(default)]
    pub details: HashMap<String, String>,

    /// Note added (for AddNote action)
    #[serde(default)]
    pub note: Option<String>,

    /// Timestamps (in milliseconds)
    #[serde(default)]
    pub created_at: Option<i64>,

    #[serde(default)]
    pub updated_at: Option<i64>,

    #[serde(default)]
    pub acknowledged_at: Option<i64>,

    #[serde(default)]
    pub closed_at: Option<i64>,
}

/// Opsgenie responder (team or user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsgenieResponder {
    /// Type: team, user, escalation, schedule
    #[serde(rename = "type")]
    pub responder_type: String,

    /// ID
    pub id: String,

    /// Name
    #[serde(default)]
    pub name: Option<String>,
}

/// Source of the action
#[derive(Debug, Clone, Deserialize)]
pub struct OpsgenieSource {
    /// Source name (username or integration name)
    pub name: String,

    /// Source type: user, integration, schedule
    #[serde(rename = "type")]
    pub source_type: String,
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/// Note creation response
#[derive(Debug, Deserialize)]
struct NoteResponse {
    #[serde(default)]
    data: Option<NoteData>,
}

#[derive(Debug, Deserialize)]
struct NoteData {
    id: String,
}

/// Alert creation response
#[derive(Debug, Deserialize)]
struct AlertResponse {
    #[serde(default)]
    data: Option<AlertData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlertData {
    #[serde(default)]
    alert_id: Option<String>,
}

/// Opsgenie API error response
#[derive(Debug, Deserialize)]
struct OpsgenieApiError {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    errors: HashMap<String, Vec<String>>,
}

// ============================================================================
// PLATFORM IMPLEMENTATION
// ============================================================================

impl OpsgeniePlatform {
    /// Create new Opsgenie platform adapter
    ///
    /// # Errors
    /// Returns error if required config fields are empty
    pub fn new(config: OpsgenieConfig) -> Result<Self, PlatformError> {
        if config.api_url.is_empty() || config.api_key.is_empty() {
            return Err(PlatformError::ParseError(
                "Opsgenie api_url and api_key are required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent(format!("AOF/{} ({})", env!("CARGO_PKG_VERSION"), config.bot_name))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify integration ID from webhook payload
    fn verify_integration(&self, payload: &OpsgenieWebhookPayload) -> bool {
        if let Some(ref expected_id) = self.config.integration_id {
            if let Some(ref actual_id) = payload.integration_id {
                if actual_id != expected_id {
                    warn!("Integration ID mismatch: expected {}, got {}", expected_id, actual_id);
                    return false;
                }
            } else {
                warn!("Integration ID missing in webhook payload");
                return false;
            }
        }
        true
    }

    /// Verify alert via API callback (optional, adds latency)
    async fn verify_alert_via_api(&self, alert_id: &str) -> bool {
        let url = format!("{}/v2/alerts/{}", self.config.api_url, alert_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("GenieKey {}", self.config.api_key))
            .send()
            .await;

        response.map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Check if action is allowed
    fn is_action_allowed(&self, action: &str) -> bool {
        if self.config.allowed_actions.is_empty() {
            true // All actions allowed if not configured
        } else {
            self.config.allowed_actions.iter().any(|a| a.eq_ignore_ascii_case(action))
        }
    }

    /// Check if priority is allowed
    fn is_priority_allowed(&self, priority: &str) -> bool {
        if let Some(ref allowed) = self.config.priority_filter {
            allowed.iter().any(|p| p.eq_ignore_ascii_case(priority))
        } else {
            true // All priorities allowed if not configured
        }
    }

    /// Check if tags match filter (ALL tags must be present)
    fn is_tag_match(&self, alert_tags: &[String]) -> bool {
        if let Some(ref required_tags) = self.config.tag_filter {
            required_tags.iter().all(|required| {
                alert_tags.iter().any(|tag| tag.eq_ignore_ascii_case(required))
            })
        } else {
            true // No filter, all tags allowed
        }
    }

    /// Check if source is allowed
    fn is_source_allowed(&self, source: &str) -> bool {
        if let Some(ref allowed) = self.config.source_filter {
            allowed.iter().any(|s| s.eq_ignore_ascii_case(source))
        } else {
            true // All sources allowed if not configured
        }
    }

    /// Parse webhook payload
    fn parse_webhook_payload(&self, payload: &[u8]) -> Result<OpsgenieWebhookPayload, PlatformError> {
        serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to parse Opsgenie webhook payload: {}", e);
            PlatformError::ParseError(format!("Invalid Opsgenie webhook payload: {}", e))
        })
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_url, path)
    }

    /// Get authorization header value
    fn auth_header(&self) -> String {
        format!("GenieKey {}", self.config.api_key)
    }

    // =========================================================================
    // PUBLIC API METHODS - For use by AgentFlow nodes and handlers
    // =========================================================================

    /// Add a note/comment to an alert
    ///
    /// # Arguments
    /// * `alert_id` - Alert UUID
    /// * `note` - Note text (supports Markdown)
    ///
    /// # Returns
    /// Note ID on success
    pub async fn add_note(&self, alert_id: &str, note: &str) -> Result<String, PlatformError> {
        if !self.config.enable_notes {
            return Err(PlatformError::ApiError("Notes are disabled".to_string()));
        }

        let url = self.api_url(&format!("/v2/alerts/{}/notes", alert_id));

        let payload = serde_json::json!({
            "note": note
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: OpsgenieApiError = response.json().await.unwrap_or(OpsgenieApiError {
                message: Some("Unknown error".to_string()),
                errors: HashMap::new(),
            });
            let msg = error.message.unwrap_or_else(|| "API error".to_string());
            return Err(PlatformError::ApiError(msg));
        }

        let note_response: NoteResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        let note_id = note_response
            .data
            .and_then(|d| Some(d.id))
            .unwrap_or_else(|| "unknown".to_string());

        info!("Posted note {} to alert {}", note_id, alert_id);
        Ok(note_id)
    }

    /// Acknowledge an alert
    ///
    /// # Arguments
    /// * `alert_id` - Alert UUID
    /// * `note` - Optional acknowledgement note
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        note: Option<&str>,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_acknowledge {
            return Err(PlatformError::ApiError("Acknowledge is disabled".to_string()));
        }

        let url = self.api_url(&format!("/v2/alerts/{}/acknowledge", alert_id));

        let mut payload = serde_json::json!({
            "user": self.config.bot_name
        });

        if let Some(note_text) = note {
            payload["note"] = serde_json::json!(note_text);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: OpsgenieApiError = response.json().await.unwrap_or(OpsgenieApiError {
                message: Some("Unknown error".to_string()),
                errors: HashMap::new(),
            });
            let msg = error.message.unwrap_or_else(|| "API error".to_string());
            return Err(PlatformError::ApiError(msg));
        }

        info!("Acknowledged alert {}", alert_id);
        Ok(())
    }

    /// Close an alert
    ///
    /// # Arguments
    /// * `alert_id` - Alert UUID
    /// * `note` - Optional close note
    pub async fn close_alert(
        &self,
        alert_id: &str,
        note: Option<&str>,
    ) -> Result<(), PlatformError> {
        if !self.config.enable_close {
            return Err(PlatformError::ApiError("Close is disabled".to_string()));
        }

        let url = self.api_url(&format!("/v2/alerts/{}/close", alert_id));

        let mut payload = serde_json::json!({
            "user": self.config.bot_name
        });

        if let Some(note_text) = note {
            payload["note"] = serde_json::json!(note_text);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: OpsgenieApiError = response.json().await.unwrap_or(OpsgenieApiError {
                message: Some("Unknown error".to_string()),
                errors: HashMap::new(),
            });
            let msg = error.message.unwrap_or_else(|| "API error".to_string());
            return Err(PlatformError::ApiError(msg));
        }

        info!("Closed alert {}", alert_id);
        Ok(())
    }

    /// Create a new alert
    ///
    /// # Arguments
    /// * `message` - Alert message/title
    /// * `description` - Detailed description
    /// * `priority` - P1/P2/P3/P4/P5
    /// * `tags` - Alert tags
    /// * `details` - Custom fields
    ///
    /// # Returns
    /// Alert ID
    pub async fn create_alert(
        &self,
        message: &str,
        description: Option<&str>,
        priority: Option<&str>,
        tags: Vec<String>,
        details: HashMap<String, String>,
    ) -> Result<String, PlatformError> {
        if !self.config.enable_create {
            return Err(PlatformError::ApiError("Create is disabled".to_string()));
        }

        let url = self.api_url("/v2/alerts");

        let mut payload = serde_json::json!({
            "message": message,
            "source": self.config.bot_name,
            "tags": tags,
            "details": details
        });

        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }

        if let Some(prio) = priority {
            payload["priority"] = serde_json::json!(prio);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error: OpsgenieApiError = response.json().await.unwrap_or(OpsgenieApiError {
                message: Some("Unknown error".to_string()),
                errors: HashMap::new(),
            });
            let msg = error.message.unwrap_or_else(|| "API error".to_string());
            return Err(PlatformError::ApiError(msg));
        }

        let alert_response: AlertResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

        let alert_id = alert_response
            .data
            .and_then(|d| d.alert_id)
            .unwrap_or_else(|| "unknown".to_string());

        info!("Created alert {}", alert_id);
        Ok(alert_id)
    }

    /// Get the config (for external use)
    pub fn config(&self) -> &OpsgenieConfig {
        &self.config
    }

    /// Format response for Opsgenie (plain text note)
    fn format_response_text(&self, response: &TriggerResponse) -> String {
        let status_text = match response.status {
            crate::response::ResponseStatus::Success => "✅ SUCCESS",
            crate::response::ResponseStatus::Error => "❌ ERROR",
            crate::response::ResponseStatus::Warning => "⚠️ WARNING",
            crate::response::ResponseStatus::Info => "ℹ️ INFO",
        };

        format!("{}\n{}", status_text, response.text)
    }

    /// Build TriggerMessage from parsed webhook
    fn build_trigger_message(
        &self,
        payload: &OpsgenieWebhookPayload,
    ) -> Result<TriggerMessage, PlatformError> {
        let alert = &payload.alert;

        // Build message text based on action
        let text = match payload.action.as_str() {
            "Create" => {
                format!(
                    "alert:created:{} {} - {}",
                    alert.alert_id,
                    alert.message,
                    alert.description.as_deref().unwrap_or("")
                )
            }
            "Acknowledge" => {
                let note = alert.note.as_deref().unwrap_or("");
                format!("alert:acknowledged:{} {}", alert.alert_id, note)
            }
            "Close" => {
                let note = alert.note.as_deref().unwrap_or("");
                format!("alert:closed:{} {}", alert.alert_id, note)
            }
            "AddNote" => {
                let note = alert.note.as_deref().unwrap_or("");
                format!("alert:note:{} {}", alert.alert_id, note)
            }
            "Escalate" => {
                format!("alert:escalated:{}", alert.alert_id)
            }
            "CustomAction" => {
                format!("alert:action:{}", alert.alert_id)
            }
            _ => format!("alert:{}:{}", payload.action.to_lowercase(), alert.alert_id),
        };

        // Extract user from source
        let (user_id, user_name) = if let Some(ref source) = payload.source {
            (source.name.clone(), Some(source.name.clone()))
        } else {
            ("system".to_string(), Some("Opsgenie System".to_string()))
        };

        let trigger_user = TriggerUser {
            id: user_id.clone(),
            username: Some(user_name.clone().unwrap_or_default()),
            display_name: user_name,
            is_bot: payload.source.as_ref().map(|s| s.source_type == "integration").unwrap_or(false),
        };

        // Channel ID is the alert ID (for threading)
        let channel_id = alert.alert_id.clone();

        // Build metadata with ALL alert fields
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), serde_json::json!(payload.action));
        metadata.insert("alert_id".to_string(), serde_json::json!(alert.alert_id));

        if let Some(ref tiny_id) = alert.tiny_id {
            metadata.insert("tiny_id".to_string(), serde_json::json!(tiny_id));
        }

        metadata.insert("message".to_string(), serde_json::json!(alert.message));

        if let Some(ref desc) = alert.description {
            metadata.insert("description".to_string(), serde_json::json!(desc));
        }

        if let Some(ref priority) = alert.priority {
            metadata.insert("priority".to_string(), serde_json::json!(priority));
        }

        metadata.insert("tags".to_string(), serde_json::json!(alert.tags));

        if let Some(ref source) = alert.source {
            metadata.insert("source".to_string(), serde_json::json!(source));
        }

        if let Some(ref entity) = alert.entity {
            metadata.insert("entity".to_string(), serde_json::json!(entity));
        }

        if let Some(ref alias) = alert.alias {
            metadata.insert("alias".to_string(), serde_json::json!(alias));
        }

        if let Some(ref owner) = alert.owner {
            metadata.insert("owner".to_string(), serde_json::json!(owner));
        }

        // Store all custom details
        if !alert.details.is_empty() {
            metadata.insert("details".to_string(), serde_json::json!(alert.details));
        }

        // Store responders
        if !alert.responders.is_empty() {
            metadata.insert("responders".to_string(), serde_json::json!(alert.responders));
        }

        // Message ID from alert ID and action
        let message_id = format!("opsgenie-{}-{}", alert.alert_id, payload.action);

        // Thread ID is alert ID (all actions on same alert are threaded)
        let thread_id = Some(alert.alert_id.clone());

        // Timestamp from created_at or now
        let timestamp = alert.created_at
            .and_then(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0))
            .unwrap_or_else(chrono::Utc::now);

        Ok(TriggerMessage {
            id: message_id,
            platform: "opsgenie".to_string(),
            channel_id,
            user: trigger_user,
            text,
            timestamp,
            metadata,
            thread_id,
            reply_to: None,
        })
    }
}

// ============================================================================
// TRIGGER PLATFORM TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl TriggerPlatform for OpsgeniePlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // 1. Parse webhook payload
        let payload = self.parse_webhook_payload(raw)?;

        // 2. Verify integration ID (if configured)
        if !self.verify_integration(&payload) {
            return Err(PlatformError::InvalidSignature(
                "Invalid integration ID".to_string()
            ));
        }

        // 3. Optional: Verify via API callback
        if self.config.verify_with_api_callback {
            if !self.verify_alert_via_api(&payload.alert.alert_id).await {
                return Err(PlatformError::InvalidSignature(
                    "Alert verification via API failed".to_string()
                ));
            }
        }

        // 4. Apply action filter
        if !self.is_action_allowed(&payload.action) {
            debug!("Action {} not allowed", payload.action);
            return Err(PlatformError::UnsupportedMessageType);
        }

        // 5. Apply priority filter
        if let Some(ref priority) = payload.alert.priority {
            if !self.is_priority_allowed(priority) {
                debug!("Priority {} not allowed", priority);
                return Err(PlatformError::UnsupportedMessageType);
            }
        }

        // 6. Apply tag filter
        if !self.is_tag_match(&payload.alert.tags) {
            debug!("Tags {:?} do not match filter", payload.alert.tags);
            return Err(PlatformError::UnsupportedMessageType);
        }

        // 7. Apply source filter
        if let Some(ref source) = payload.alert.source {
            if !self.is_source_allowed(source) {
                debug!("Source {} not allowed", source);
                return Err(PlatformError::UnsupportedMessageType);
            }
        }

        // 8. Build TriggerMessage
        self.build_trigger_message(&payload)
    }

    async fn send_response(
        &self,
        channel: &str, // Alert ID
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        let note = self.format_response_text(&response);
        self.add_note(channel, &note).await?;
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "opsgenie"
    }

    async fn verify_signature(&self, payload: &[u8], _signature: &str) -> bool {
        // Opsgenie uses integration ID validation, not HMAC
        let webhook: OpsgenieWebhookPayload = match serde_json::from_slice(payload) {
            Ok(w) => w,
            Err(_) => return false,
        };

        self.verify_integration(&webhook)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // All alert actions are threaded by alert ID
    }

    fn supports_interactive(&self) -> bool {
        true // Custom actions supported
    }

    fn supports_files(&self) -> bool {
        false // Opsgenie API doesn't support file attachments via notes
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> OpsgenieConfig {
        OpsgenieConfig {
            api_url: "https://api.opsgenie.com".to_string(),
            api_key: "test_key".to_string(),
            webhook_token: Some("test_token".to_string()),
            bot_name: "test-aof-bot".to_string(),
            allowed_actions: vec![],
            priority_filter: None,
            tag_filter: None,
            team_filter: None,
            source_filter: None,
            enable_notes: true,
            enable_updates: true,
            enable_close: true,
            enable_acknowledge: true,
            enable_create: true,
            integration_id: None,
            verify_with_api_callback: false,
            rate_limit: 600,
            timeout_secs: 30,
        }
    }

    #[test]
    fn test_opsgenie_platform_new() {
        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_opsgenie_platform_invalid_config() {
        let config = OpsgenieConfig {
            api_url: "".to_string(),
            api_key: "".to_string(),
            webhook_token: None,
            bot_name: "".to_string(),
            allowed_actions: vec![],
            priority_filter: None,
            tag_filter: None,
            team_filter: None,
            source_filter: None,
            enable_notes: true,
            enable_updates: true,
            enable_close: true,
            enable_acknowledge: true,
            enable_create: true,
            integration_id: None,
            verify_with_api_callback: false,
            rate_limit: 600,
            timeout_secs: 30,
        };
        let platform = OpsgeniePlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_action_allowed_all() {
        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_action_allowed("Create"));
        assert!(platform.is_action_allowed("Acknowledge"));
        assert!(platform.is_action_allowed("Close"));
    }

    #[test]
    fn test_action_allowed_filter() {
        let mut config = create_test_config();
        config.allowed_actions = vec!["Create".to_string(), "AddNote".to_string()];

        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_action_allowed("Create"));
        assert!(platform.is_action_allowed("AddNote"));
        assert!(!platform.is_action_allowed("Close"));
    }

    #[test]
    fn test_priority_allowed_all() {
        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_priority_allowed("P1"));
        assert!(platform.is_priority_allowed("P2"));
        assert!(platform.is_priority_allowed("P3"));
    }

    #[test]
    fn test_priority_allowed_filter() {
        let mut config = create_test_config();
        config.priority_filter = Some(vec!["P1".to_string(), "P2".to_string()]);

        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_priority_allowed("P1"));
        assert!(platform.is_priority_allowed("P2"));
        assert!(!platform.is_priority_allowed("P3"));
    }

    #[test]
    fn test_tag_match_all() {
        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_tag_match(&["production".to_string(), "database".to_string()]));
        assert!(platform.is_tag_match(&[]));
    }

    #[test]
    fn test_tag_match_filter() {
        let mut config = create_test_config();
        config.tag_filter = Some(vec!["production".to_string()]);

        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_tag_match(&["production".to_string(), "database".to_string()]));
        assert!(!platform.is_tag_match(&["staging".to_string()]));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "opsgenie");
        assert_eq!(platform.bot_name(), "test-aof-bot");
        assert!(platform.supports_threading());
        assert!(platform.supports_interactive());
        assert!(!platform.supports_files());
    }

    #[tokio::test]
    async fn test_parse_create_alert_webhook() {
        let webhook_json = r#"{
            "action": "Create",
            "alert": {
                "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
                "message": "CPU usage above 90%",
                "priority": "P1",
                "tags": ["production", "database"],
                "source": "datadog"
            },
            "integrationId": "test-integration"
        }"#;

        let mut config = create_test_config();
        config.integration_id = Some("test-integration".to_string());

        let platform = OpsgeniePlatform::new(config).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "opsgenie");
        assert!(message.text.contains("alert:created"));
        assert_eq!(message.metadata.get("alert_id").unwrap(), &serde_json::json!("1c222736-5ec3-46e7-aeeb-1d608a7c1c12"));
        assert_eq!(message.metadata.get("priority").unwrap(), &serde_json::json!("P1"));
    }

    #[tokio::test]
    async fn test_parse_add_note_webhook() {
        let webhook_json = r#"{
            "action": "AddNote",
            "alert": {
                "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
                "message": "CPU usage above 90%",
                "note": "/run diagnostics --verbose"
            },
            "source": {
                "name": "john.doe@example.com",
                "type": "user"
            }
        }"#;

        let config = create_test_config();
        let platform = OpsgeniePlatform::new(config).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.text.contains("alert:note"));
        assert!(message.text.contains("/run diagnostics --verbose"));
        assert_eq!(message.user.id, "john.doe@example.com");
        assert!(!message.user.is_bot);
    }
}
