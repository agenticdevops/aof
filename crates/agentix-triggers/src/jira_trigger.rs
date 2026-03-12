/// JiraTrigger — accepts Jira webhook payloads and emits TriggerEvents.
///
/// Filters by subscribed event types using the `webhookEvent` field in the JSON body.
/// Optionally verifies HMAC-SHA256 signature via `x-hub-signature` header.
use agentix_core::{AgentixError, TriggerEvent, TriggerSource};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

/// A Jira webhook trigger.
pub struct JiraTrigger {
    id: String,
    agent_name: String,
    /// Jira event types to subscribe to (e.g. "jira:issue_created", "jira:issue_updated")
    events: Vec<String>,
    /// Optional HMAC secret for signature verification via `x-hub-signature`
    secret: Option<String>,
}

impl JiraTrigger {
    pub fn new(
        id: impl Into<String>,
        agent_name: impl Into<String>,
        events: Vec<String>,
        secret: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            agent_name: agent_name.into(),
            events,
            secret,
        }
    }

    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    /// Process a Jira webhook payload.
    ///
    /// Returns `Ok(None)` when:
    /// - The `webhookEvent` field in the body is not in the subscription list
    /// - Signature verification fails (when a secret is configured)
    ///
    /// Returns `Ok(Some(event))` when the payload is accepted.
    pub fn receive_payload(
        &self,
        body: serde_json::Value,
        headers: &HashMap<String, String>,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        // Event type is carried in the JSON body as "webhookEvent"
        let event_type = body
            .get("webhookEvent")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !self.events.is_empty() && !self.events.iter().any(|e| e == event_type) {
            return Ok(None);
        }

        // Signature verification (only when secret is set)
        if let Some(secret) = &self.secret {
            match headers.get("x-hub-signature") {
                None => return Ok(None),
                Some(provided_sig) => {
                    let body_bytes = serde_json::to_vec(&body)
                        .map_err(|e| AgentixError::runtime(format!("serialization error: {e}")))?;
                    if !verify_hmac_sha256(secret, &body_bytes, provided_sig) {
                        return Ok(None);
                    }
                }
            }
        }

        let mut context = HashMap::new();
        context.insert("jira_event".to_string(), event_type.to_string());

        let event = TriggerEvent {
            source: TriggerSource::Jira,
            payload: body,
            context,
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        };
        Ok(Some(event))
    }
}

/// Verify HMAC-SHA256 signature: provided must equal hex(HMAC-SHA256(secret, body)).
fn verify_hmac_sha256(secret: &str, body: &[u8], provided: &str) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == provided
}
