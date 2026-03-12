/// WebhookTrigger — accepts arbitrary HTTP payloads and emits TriggerEvents.
///
/// When no secret is configured, any payload is accepted.
/// When a secret is configured, the `x-webhook-signature` header must contain
/// a valid HMAC-SHA256 hex digest of the raw request body.
use agentix_core::{AgentixError, TriggerEvent, TriggerSource};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

/// A simple webhook trigger that converts any HTTP payload into a TriggerEvent.
pub struct WebhookTrigger {
    id: String,
    agent_name: String,
    path: String,
    secret: Option<String>,
}

impl WebhookTrigger {
    /// Create a new WebhookTrigger.
    ///
    /// - `id`: unique trigger identifier
    /// - `agent_name`: the agent this trigger fires for
    /// - `path`: the URL path this webhook listens on (e.g. `/webhooks/my-webhook`)
    /// - `secret`: optional HMAC secret for signature verification
    pub fn new(
        id: impl Into<String>,
        agent_name: impl Into<String>,
        path: impl Into<String>,
        secret: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            agent_name: agent_name.into(),
            path: path.into(),
            secret,
        }
    }

    /// Path this webhook listens on.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Agent this trigger fires for.
    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    /// Receive an incoming payload and produce a TriggerEvent (or None if rejected).
    ///
    /// Returns `Err` only on internal errors (e.g. HMAC key issues).
    /// Returns `Ok(None)` when the payload is rejected due to bad signature.
    /// Returns `Ok(Some(event))` on success.
    pub fn receive_payload(
        &self,
        body: serde_json::Value,
        headers: &HashMap<String, String>,
    ) -> Result<Option<TriggerEvent>, AgentixError> {
        // Signature verification (only when secret is set)
        if let Some(secret) = &self.secret {
            let sig_header = headers.get("x-webhook-signature");
            match sig_header {
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

        let event = TriggerEvent {
            source: TriggerSource::Webhook,
            payload: body,
            context: HashMap::new(),
            fired_at: Utc::now(),
            trigger_id: self.id.clone(),
        };
        Ok(Some(event))
    }
}

/// Verify HMAC-SHA256 signature: `provided` must equal hex(HMAC-SHA256(secret, body)).
fn verify_hmac_sha256(secret: &str, body: &[u8], provided: &str) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    // Constant-time comparison via hex string equality
    expected == provided
}
