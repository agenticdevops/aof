/// GitHubTrigger — accepts GitHub webhook payloads and emits TriggerEvents.
///
/// Filters by subscribed event types (from `x-github-event` header).
/// Optionally verifies HMAC-SHA256 signature via `x-hub-signature-256` header.
use agentix_core::{AgentixError, TriggerEvent, TriggerSource};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

/// A GitHub webhook trigger.
pub struct GitHubTrigger {
    id: String,
    agent_name: String,
    /// Event types to subscribe to (e.g. "push", "pull_request", "issues")
    events: Vec<String>,
    /// Optional webhook secret for HMAC-SHA256 verification via `x-hub-signature-256`
    secret: Option<String>,
}

impl GitHubTrigger {
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

    /// Process a GitHub webhook payload.
    ///
    /// Returns `Ok(None)` when:
    /// - The event type (from `x-github-event` header) is not in the subscription list
    /// - Signature verification fails (when a secret is configured)
    ///
    /// Returns `Ok(Some(event))` when the payload is accepted.
    pub fn receive_payload(
        &self,
        body: serde_json::Value,
        headers: &HashMap<String, String>,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        // Check event type filter
        let event_type = headers.get("x-github-event").map(|s| s.as_str()).unwrap_or("");
        if !self.events.is_empty() && !self.events.iter().any(|e| e == event_type) {
            return Ok(None);
        }

        // Signature verification (only when secret is set)
        if let Some(secret) = &self.secret {
            match headers.get("x-hub-signature-256") {
                None => return Ok(None),
                Some(provided_sig) => {
                    let body_bytes = serde_json::to_vec(&body)
                        .map_err(|e| AgentixError::runtime(format!("serialization error: {e}")))?;
                    if !verify_github_signature(secret, &body_bytes, provided_sig) {
                        return Ok(None);
                    }
                }
            }
        }

        let mut context = HashMap::new();
        context.insert("github_event".to_string(), event_type.to_string());

        let event = TriggerEvent {
            source: TriggerSource::GitHub,
            payload: body,
            context,
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        };
        Ok(Some(event))
    }
}

/// Verify GitHub-style HMAC-SHA256 signature.
/// GitHub sends: `x-hub-signature-256: sha256=<hex>`
fn verify_github_signature(secret: &str, body: &[u8], provided: &str) -> bool {
    // Strip the "sha256=" prefix if present
    let hex_sig = provided.strip_prefix("sha256=").unwrap_or(provided);

    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == hex_sig
}
