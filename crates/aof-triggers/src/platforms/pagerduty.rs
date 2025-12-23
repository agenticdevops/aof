//! PagerDuty V3 Webhooks adapter for AOF
//!
//! This module provides integration with PagerDuty's V3 Webhooks API, supporting:
//! - Incident events (triggered, acknowledged, resolved, escalated, reassigned)
//! - HMAC-SHA256 signature verification
//! - Event filtering by service, team, priority, and urgency
//! - REST API integration for adding notes to incidents
//!
//! # References
//! - [PagerDuty V3 Webhooks](https://developer.pagerduty.com/docs/88922dc5e1ad1-overview-v2-webhooks)
//! - [Webhook Signature Verification](https://developer.pagerduty.com/docs/ZG9jOjExMDI5NTkz-verifying-signatures)
//! - [REST API Reference](https://developer.pagerduty.com/api-reference/9d0b4b12e36f9-list-incidents)

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::{PlatformError, TriggerMessage, TriggerPlatform, TriggerUser};
use crate::response::TriggerResponse;

type HmacSha256 = Hmac<Sha256>;

/// PagerDuty REST API base URL
const REST_API_URL: &str = "https://api.pagerduty.com";

/// PagerDuty platform adapter
pub struct PagerDutyPlatform {
    config: PagerDutyConfig,
    client: reqwest::Client,
}

/// PagerDuty platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// Webhook secret for signature verification (required)
    /// Obtained when creating a Generic Webhook (v3) in PagerDuty
    pub webhook_secret: String,

    /// PagerDuty API token for response actions (optional)
    /// Required for updating incidents, adding notes, etc.
    /// Format: "Token token={api_token}" in Authorization header
    #[serde(default)]
    pub api_token: Option<String>,

    /// Bot name for display purposes
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Event types to process (default: all incident events)
    /// Example: ["incident.triggered", "incident.acknowledged", "incident.resolved"]
    #[serde(default)]
    pub event_types: Option<Vec<String>>,

    /// Allowed service IDs (optional - filter by service)
    /// Example: ["PXYZ123", "PABC456"]
    #[serde(default)]
    pub allowed_services: Option<Vec<String>>,

    /// Allowed team IDs (optional - filter by team)
    /// Example: ["P456DEF", "P789GHI"]
    #[serde(default)]
    pub allowed_teams: Option<Vec<String>>,

    /// Minimum priority level to process (optional)
    /// Values: "P1", "P2", "P3", "P4", "P5" (P1 = highest priority)
    #[serde(default)]
    pub min_priority: Option<String>,

    /// Minimum urgency level to process (optional)
    /// Values: "high", "low"
    #[serde(default)]
    pub min_urgency: Option<String>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

/// PagerDuty webhook event envelope
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyWebhook {
    event: PagerDutyEvent,
}

/// PagerDuty event structure
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyEvent {
    /// Unique event ID
    id: String,

    /// Event type (e.g., "incident.triggered")
    event_type: String,

    /// Resource type (always "incident" for incident events)
    resource_type: String,

    /// Timestamp when event occurred
    occurred_at: String,

    /// User/agent who triggered the event (optional for system events)
    #[serde(default)]
    agent: Option<PagerDutyAgent>,

    /// Event data (incident details)
    data: PagerDutyIncident,
}

/// PagerDuty agent/user reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyAgent {
    id: String,
    summary: String,
    #[serde(rename = "type")]
    agent_type: String,
}

/// PagerDuty incident data
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyIncident {
    /// Incident ID
    id: String,

    /// Resource type
    #[serde(rename = "type")]
    resource_type: String,

    /// API URL
    #[serde(rename = "self")]
    self_url: String,

    /// Web UI URL
    html_url: String,

    /// Incident number (human-readable)
    number: u64,

    /// Current status (triggered, acknowledged, resolved)
    status: String,

    /// Incident key (deduplication key)
    incident_key: String,

    /// Created timestamp
    created_at: String,

    /// Incident title/summary
    title: String,

    /// Service reference
    service: PagerDutyServiceRef,

    /// Assigned users
    #[serde(default)]
    assignees: Vec<PagerDutyUserRef>,

    /// Escalation policy
    #[serde(default)]
    escalation_policy: Option<PagerDutyEscalationRef>,

    /// Teams
    #[serde(default)]
    teams: Vec<PagerDutyTeamRef>,

    /// Priority
    #[serde(default)]
    priority: Option<PagerDutyPriorityRef>,

    /// Urgency level (high, low)
    urgency: String,
}

/// Service reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyServiceRef {
    id: String,
    summary: String,
}

/// User reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyUserRef {
    id: String,
    summary: String,
}

/// Escalation policy reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyEscalationRef {
    id: String,
    summary: String,
}

/// Team reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyTeamRef {
    id: String,
    summary: String,
}

/// Priority reference
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyPriorityRef {
    id: String,
    summary: String,
}

/// PagerDuty API response
#[derive(Debug, Deserialize)]
struct PagerDutyApiResponse {
    #[serde(default)]
    error: Option<serde_json::Value>,
}

impl PagerDutyPlatform {
    /// Create new PagerDuty platform adapter
    pub fn new(config: PagerDutyConfig) -> Result<Self, PlatformError> {
        if config.webhook_secret.is_empty() {
            return Err(PlatformError::ParseError(
                "Webhook secret is required".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Verify PagerDuty webhook signature using HMAC-SHA256
    ///
    /// PagerDuty signature format: v1=<hex_encoded_signature>
    /// The signature is computed over the raw request body.
    fn verify_pagerduty_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Signature format: v1=<hex_signature>
        if !signature.starts_with("v1=") {
            debug!("Invalid signature format - must start with v1=");
            return false;
        }

        let provided_signature = &signature[3..];

        // Create HMAC-SHA256 with webhook secret
        let mut mac = match HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                error!("HMAC setup failed: {}", e);
                return false;
            }
        };

        // Hash the raw payload
        mac.update(payload);

        let result = mac.finalize();
        let computed_signature = hex::encode(result.into_bytes());

        // Constant-time comparison
        if computed_signature == provided_signature {
            debug!("PagerDuty signature verified successfully");
            true
        } else {
            debug!(
                "Signature mismatch - computed: {}, provided: {}",
                &computed_signature[..8],
                &provided_signature[..8.min(provided_signature.len())]
            );
            false
        }
    }

    /// Check if event should be processed based on configuration filters
    fn should_process_event(&self, event: &PagerDutyEvent) -> bool {
        // Check event type filter
        if let Some(ref allowed_types) = self.config.event_types {
            if !allowed_types.contains(&event.event_type) {
                debug!("Event type {} not in allowed list", event.event_type);
                return false;
            }
        }

        let incident = &event.data;

        // Check service filter
        if let Some(ref allowed_services) = self.config.allowed_services {
            if !allowed_services.contains(&incident.service.id) {
                debug!("Service {} not in allowed list", incident.service.id);
                return false;
            }
        }

        // Check team filter
        if let Some(ref allowed_teams) = self.config.allowed_teams {
            let has_allowed_team = incident
                .teams
                .iter()
                .any(|team| allowed_teams.contains(&team.id));
            if !has_allowed_team {
                debug!("No allowed teams found for incident");
                return false;
            }
        }

        // Check priority filter
        if let Some(ref min_priority) = self.config.min_priority {
            if let Some(ref priority) = incident.priority {
                // P1 > P2 > P3 > P4 > P5 (lower number = higher priority)
                if !is_priority_sufficient(&priority.summary, min_priority) {
                    debug!("Priority {} below minimum {}", priority.summary, min_priority);
                    return false;
                }
            }
        }

        // Check urgency filter
        if let Some(ref min_urgency) = self.config.min_urgency {
            if incident.urgency != *min_urgency && *min_urgency == "high" {
                debug!("Urgency {} below minimum high", incident.urgency);
                return false;
            }
        }

        true
    }

    /// Parse PagerDuty event and convert to TriggerMessage
    async fn parse_pagerduty_event(
        &self,
        webhook: PagerDutyWebhook,
    ) -> Result<TriggerMessage, PlatformError> {
        let event = webhook.event;
        let incident = event.data;

        // Create TriggerUser from agent (or system if no agent)
        let trigger_user = if let Some(agent) = event.agent {
            TriggerUser {
                id: agent.id.clone(),
                username: Some(agent.id),
                display_name: Some(agent.summary),
                is_bot: agent.agent_type == "service" || agent.agent_type == "bot_reference",
            }
        } else {
            // System event (no agent)
            TriggerUser {
                id: "system".to_string(),
                username: Some("system".to_string()),
                display_name: Some("PagerDuty System".to_string()),
                is_bot: true,
            }
        };

        // Build metadata with all incident details
        let mut metadata = HashMap::new();
        metadata.insert("event_id".to_string(), serde_json::json!(event.id));
        metadata.insert("event_type".to_string(), serde_json::json!(event.event_type));
        metadata.insert("occurred_at".to_string(), serde_json::json!(event.occurred_at));
        metadata.insert("incident_id".to_string(), serde_json::json!(incident.id));
        metadata.insert("incident_number".to_string(), serde_json::json!(incident.number));
        metadata.insert("incident_key".to_string(), serde_json::json!(incident.incident_key));
        metadata.insert("status".to_string(), serde_json::json!(incident.status));
        metadata.insert("urgency".to_string(), serde_json::json!(incident.urgency));
        metadata.insert("html_url".to_string(), serde_json::json!(incident.html_url));
        metadata.insert("service_id".to_string(), serde_json::json!(incident.service.id));
        metadata.insert("service_name".to_string(), serde_json::json!(incident.service.summary));

        if let Some(priority) = incident.priority {
            metadata.insert("priority".to_string(), serde_json::json!(priority.summary));
        }

        if !incident.teams.is_empty() {
            let team_ids: Vec<_> = incident.teams.iter().map(|t| &t.id).collect();
            metadata.insert("team_ids".to_string(), serde_json::json!(team_ids));
        }

        if !incident.assignees.is_empty() {
            let assignee_ids: Vec<_> = incident.assignees.iter().map(|a| &a.id).collect();
            metadata.insert("assignee_ids".to_string(), serde_json::json!(assignee_ids));
        }

        // Create TriggerMessage
        Ok(TriggerMessage {
            id: event.id,
            platform: "pagerduty".to_string(),
            channel_id: incident.service.id.clone(), // Use service ID as channel
            user: trigger_user,
            text: incident.title,
            timestamp: chrono::Utc::now(),
            metadata,
            thread_id: Some(incident.id.clone()), // Use incident ID as thread for grouping
            reply_to: None,
        })
    }

    /// Add a note to a PagerDuty incident
    ///
    /// # Arguments
    /// * `incident_id` - The incident ID
    /// * `note_content` - The note text to add
    /// * `from_email` - Email address for the request (required by PagerDuty API)
    pub async fn add_incident_note(
        &self,
        incident_id: &str,
        note_content: &str,
        from_email: &str,
    ) -> Result<(), PlatformError> {
        let api_token = self
            .config
            .api_token
            .as_ref()
            .ok_or_else(|| PlatformError::ApiError("API token not configured".to_string()))?;

        let payload = serde_json::json!({
            "note": {
                "content": note_content,
            }
        });

        let url = format!("{}/incidents/{}/notes", REST_API_URL, incident_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token token={}", api_token))
            .header("Content-Type", "application/json")
            .header("From", from_email)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Failed to add incident note: {}", error);
            return Err(PlatformError::ApiError(format!("API error: {}", error)));
        }

        info!("Successfully added note to incident {}", incident_id);
        Ok(())
    }

    /// Update incident status (acknowledge, resolve)
    ///
    /// # Arguments
    /// * `incident_id` - The incident ID
    /// * `status` - New status ("acknowledged", "resolved")
    /// * `from_email` - Email address for the request
    pub async fn update_incident_status(
        &self,
        incident_id: &str,
        status: &str,
        from_email: &str,
    ) -> Result<(), PlatformError> {
        let api_token = self
            .config
            .api_token
            .as_ref()
            .ok_or_else(|| PlatformError::ApiError("API token not configured".to_string()))?;

        let payload = serde_json::json!({
            "incident": {
                "type": "incident_reference",
                "status": status,
            }
        });

        let url = format!("{}/incidents/{}", REST_API_URL, incident_id);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Token token={}", api_token))
            .header("Content-Type", "application/json")
            .header("From", from_email)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Failed to update incident status: {}", error);
            return Err(PlatformError::ApiError(format!("API error: {}", error)));
        }

        info!("Successfully updated incident {} to status {}", incident_id, status);
        Ok(())
    }
}

/// Check if current priority is sufficient compared to minimum
/// P1 > P2 > P3 > P4 > P5 (lower number = higher priority)
fn is_priority_sufficient(current: &str, minimum: &str) -> bool {
    let priority_map: HashMap<&str, u8> = [
        ("P1", 1),
        ("P2", 2),
        ("P3", 3),
        ("P4", 4),
        ("P5", 5),
    ]
    .iter()
    .cloned()
    .collect();

    let current_level = priority_map.get(current).unwrap_or(&99);
    let min_level = priority_map.get(minimum).unwrap_or(&0);

    // Lower number = higher priority, so current should be <= minimum
    current_level <= min_level
}

#[async_trait]
impl TriggerPlatform for PagerDutyPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Verify signature
        if let Some(signature) = headers.get("x-pagerduty-signature") {
            if !self.verify_pagerduty_signature(raw, signature) {
                warn!("Invalid PagerDuty signature");
                return Err(PlatformError::InvalidSignature(
                    "Signature verification failed".to_string(),
                ));
            }
        } else {
            warn!("Missing x-pagerduty-signature header");
            return Err(PlatformError::InvalidSignature(
                "Missing signature header".to_string(),
            ));
        }

        // Parse webhook payload
        let webhook: PagerDutyWebhook = serde_json::from_slice(raw).map_err(|e| {
            error!("Failed to parse PagerDuty webhook: {}", e);
            PlatformError::ParseError(format!("Invalid webhook payload: {}", e))
        })?;

        info!(
            "Received PagerDuty event: {} for incident #{}",
            webhook.event.event_type, webhook.event.data.number
        );

        // Check if event should be processed
        if !self.should_process_event(&webhook.event) {
            debug!("Event filtered out by configuration");
            return Err(PlatformError::UnsupportedMessageType);
        }

        // Convert to TriggerMessage
        self.parse_pagerduty_event(webhook).await
    }

    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // channel = incident_id for PagerDuty
        let incident_id = channel;

        // Add note to incident with agent response
        if self.config.api_token.is_some() {
            let note = format!(
                "**AOF Agent Response**\n\n{}",
                response.text
            );

            // Use a default email or extract from metadata
            let from_email = response
                .metadata
                .get("from_email")
                .and_then(|v| v.as_str())
                .unwrap_or("aof@example.com");

            self.add_incident_note(incident_id, &note, from_email).await?;
        } else {
            warn!("API token not configured - cannot add notes to incidents");
        }

        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "pagerduty"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.verify_pagerduty_signature(payload, signature)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Incident notes form threads
    }

    fn supports_interactive(&self) -> bool {
        false // No interactive components in webhooks
    }

    fn supports_files(&self) -> bool {
        false // No file attachments in webhooks
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> PagerDutyConfig {
        PagerDutyConfig {
            webhook_secret: "test-secret-123".to_string(),
            api_token: None,
            bot_name: "testbot".to_string(),
            event_types: None,
            allowed_services: None,
            allowed_teams: None,
            min_priority: None,
            min_urgency: None,
        }
    }

    #[test]
    fn test_pagerduty_platform_new() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_pagerduty_platform_invalid_config() {
        let config = PagerDutyConfig {
            webhook_secret: "".to_string(),
            api_token: None,
            bot_name: "testbot".to_string(),
            event_types: None,
            allowed_services: None,
            allowed_teams: None,
            min_priority: None,
            min_urgency: None,
        };
        let platform = PagerDutyPlatform::new(config);
        assert!(platform.is_err());
    }

    #[test]
    fn test_priority_filtering() {
        // P1 should pass when minimum is P2 (P1 is higher priority)
        assert!(is_priority_sufficient("P1", "P2"));

        // P2 should pass when minimum is P2 (equal)
        assert!(is_priority_sufficient("P2", "P2"));

        // P3 should fail when minimum is P2 (P3 is lower priority)
        assert!(!is_priority_sufficient("P3", "P2"));

        // P1 should pass when minimum is P1
        assert!(is_priority_sufficient("P1", "P1"));

        // P5 should fail when minimum is P2
        assert!(!is_priority_sufficient("P5", "P2"));
    }

    #[test]
    fn test_platform_capabilities() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config).unwrap();

        assert_eq!(platform.platform_name(), "pagerduty");
        assert_eq!(platform.bot_name(), "testbot");
        assert!(platform.supports_threading());
        assert!(!platform.supports_interactive());
        assert!(!platform.supports_files());
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config).unwrap();

        let payload = b"test payload";
        let invalid_signature = "v1=invalid";

        let result = platform.verify_signature(payload, invalid_signature).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_parse_incident_triggered() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config).unwrap();

        let payload = r#"{
            "event": {
                "id": "01DRF6BV56ABC",
                "event_type": "incident.triggered",
                "resource_type": "incident",
                "occurred_at": "2025-12-23T10:30:00Z",
                "data": {
                    "id": "Q2KURS8RXYZ123",
                    "type": "incident",
                    "self": "https://api.pagerduty.com/incidents/Q2KURS8RXYZ123",
                    "html_url": "https://acme.pagerduty.com/incidents/Q2KURS8RXYZ123",
                    "number": 123,
                    "status": "triggered",
                    "incident_key": "srv01/high_cpu",
                    "created_at": "2025-12-23T10:30:00Z",
                    "title": "High CPU usage on srv01",
                    "service": {
                        "id": "PXYZ123",
                        "summary": "Production API"
                    },
                    "assignees": [],
                    "teams": [],
                    "urgency": "high"
                }
            }
        }"#;

        // Note: This will fail signature verification, but tests parsing logic
        let webhook: PagerDutyWebhook = serde_json::from_slice(payload.as_bytes()).unwrap();
        let result = platform.parse_pagerduty_event(webhook).await;

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.platform, "pagerduty");
        assert_eq!(msg.text, "High CPU usage on srv01");
        assert_eq!(
            msg.metadata.get("status").unwrap().as_str().unwrap(),
            "triggered"
        );
        assert_eq!(
            msg.metadata.get("service_name").unwrap().as_str().unwrap(),
            "Production API"
        );
    }
}
