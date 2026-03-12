// TDD RED phase — written before WebhookTrigger implementation.

use agentix_core::TriggerSource;
use agentix_triggers::WebhookTrigger;
use std::collections::HashMap;

#[test]
fn test_webhook_trigger_fire() {
    let trigger = WebhookTrigger::new("webhook-1", "my-agent", "/webhooks/my-webhook", None);
    let body = serde_json::json!({"message": "deploy started", "env": "production"});
    let headers = HashMap::new();

    let result = trigger.receive_payload(body.clone(), &headers);
    assert!(result.is_ok(), "receive_payload should succeed: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::Webhook));
    assert_eq!(event.payload["message"], "deploy started");
}

#[test]
fn test_webhook_trigger_no_secret() {
    let trigger = WebhookTrigger::new("webhook-2", "agent-2", "/webhooks/agent-2", None);
    let body = serde_json::json!({"data": "anything"});
    let mut headers = HashMap::new();
    headers.insert("x-custom-header".to_string(), "value".to_string());

    // No secret configured → any payload accepted
    let result = trigger.receive_payload(body, &headers);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_webhook_trigger_with_secret() {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "test-secret-123";
    let trigger = WebhookTrigger::new("webhook-3", "agent-3", "/webhooks/agent-3", Some(secret.to_string()));

    let body_str = r#"{"event":"test"}"#;
    let body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    // HMAC must match what the implementation computes (re-serialized JSON bytes)
    let body_bytes = serde_json::to_vec(&body).unwrap();

    // Compute valid HMAC
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(&body_bytes);
    let valid_sig = hex::encode(mac.finalize().into_bytes());

    // Valid signature → accepted
    let mut headers_ok = HashMap::new();
    headers_ok.insert("x-webhook-signature".to_string(), valid_sig);
    let result_ok = trigger.receive_payload(body.clone(), &headers_ok);
    assert!(result_ok.is_ok(), "valid signature should be accepted");
    assert!(result_ok.unwrap().is_some());

    // Bad signature → rejected (returns None or Err)
    let mut headers_bad = HashMap::new();
    headers_bad.insert("x-webhook-signature".to_string(), "badhash123".to_string());
    let result_bad = trigger.receive_payload(body, &headers_bad);
    // Either Err or Ok(None) is acceptable
    let rejected = result_bad.is_err() || result_bad.unwrap().is_none();
    assert!(rejected, "invalid signature should be rejected");
}
