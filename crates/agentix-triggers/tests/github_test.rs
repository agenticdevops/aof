// TDD RED phase — written before GitHubTrigger implementation.

use agentix_core::TriggerSource;
use agentix_triggers::GitHubTrigger;
use std::collections::HashMap;

#[test]
fn test_github_trigger_push_event() {
    let trigger = GitHubTrigger::new(
        "github-1",
        "my-agent",
        vec!["push".to_string(), "pull_request".to_string()],
        None,
    );

    let body = serde_json::json!({
        "ref": "refs/heads/main",
        "repository": { "full_name": "org/repo" },
        "commits": [{ "message": "fix: resolve auth bug" }]
    });

    let mut headers = HashMap::new();
    headers.insert("x-github-event".to_string(), "push".to_string());

    let result = trigger.receive_payload(body.clone(), &headers);
    assert!(result.is_ok(), "push event should be accepted: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::GitHub));
    assert_eq!(event.payload["ref"], "refs/heads/main");
}

#[test]
fn test_github_trigger_filters_unsubscribed_event() {
    let trigger = GitHubTrigger::new(
        "github-2",
        "my-agent",
        vec!["push".to_string()],
        None,
    );

    let body = serde_json::json!({ "action": "opened" });
    let mut headers = HashMap::new();
    headers.insert("x-github-event".to_string(), "pull_request".to_string());

    let result = trigger.receive_payload(body, &headers);
    assert!(result.is_ok());
    // pull_request not in subscription list → filtered out
    assert!(result.unwrap().is_none());
}

#[test]
fn test_github_trigger_with_valid_signature() {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "github-secret-abc";
    let trigger = GitHubTrigger::new(
        "github-3",
        "my-agent",
        vec!["push".to_string()],
        Some(secret.to_string()),
    );

    let body_str = r#"{"ref":"refs/heads/main"}"#;
    let body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    // HMAC must match what the implementation computes (re-serialized JSON bytes)
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(&body_bytes);
    let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    let mut headers = HashMap::new();
    headers.insert("x-github-event".to_string(), "push".to_string());
    headers.insert("x-hub-signature-256".to_string(), sig);

    let result = trigger.receive_payload(body, &headers);
    assert!(result.is_ok(), "valid signature should be accepted");
    assert!(result.unwrap().is_some());
}

#[test]
fn test_github_trigger_rejects_invalid_signature() {
    let secret = "github-secret-abc";
    let trigger = GitHubTrigger::new(
        "github-4",
        "my-agent",
        vec!["push".to_string()],
        Some(secret.to_string()),
    );

    let body = serde_json::json!({"ref": "refs/heads/main"});
    let mut headers = HashMap::new();
    headers.insert("x-github-event".to_string(), "push".to_string());
    headers.insert("x-hub-signature-256".to_string(), "sha256=badhash".to_string());

    let result = trigger.receive_payload(body, &headers);
    let rejected = result.is_err() || result.unwrap().is_none();
    assert!(rejected, "invalid signature should be rejected");
}
