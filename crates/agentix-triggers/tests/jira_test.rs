// TDD RED phase — written before JiraTrigger implementation.

use agentix_core::TriggerSource;
use agentix_triggers::JiraTrigger;
use std::collections::HashMap;

#[test]
fn test_jira_trigger_issue_created() {
    let trigger = JiraTrigger::new(
        "jira-1",
        "my-agent",
        vec!["jira:issue_created".to_string()],
        None,
    );

    let body = serde_json::json!({
        "webhookEvent": "jira:issue_created",
        "issue": {
            "id": "10001",
            "key": "PROJ-42",
            "fields": {
                "summary": "Build fails on CI",
                "status": { "name": "Open" }
            }
        }
    });

    let headers = HashMap::new();
    let result = trigger.receive_payload(body.clone(), &headers);
    assert!(result.is_ok(), "issue_created should be accepted: {:?}", result);

    let event_opt = result.unwrap();
    assert!(event_opt.is_some(), "should produce a TriggerEvent");
    let event = event_opt.unwrap();
    assert!(matches!(event.source, TriggerSource::Jira));
    assert_eq!(event.payload["issue"]["key"], "PROJ-42");
}

#[test]
fn test_jira_trigger_filters_unsubscribed_event() {
    let trigger = JiraTrigger::new(
        "jira-2",
        "my-agent",
        vec!["jira:issue_created".to_string()],
        None,
    );

    let body = serde_json::json!({
        "webhookEvent": "jira:issue_updated",
        "issue": { "key": "PROJ-43" }
    });

    let headers = HashMap::new();
    let result = trigger.receive_payload(body, &headers);
    assert!(result.is_ok());
    // issue_updated not in subscription list → filtered out
    assert!(result.unwrap().is_none());
}

#[test]
fn test_jira_trigger_multiple_events() {
    let trigger = JiraTrigger::new(
        "jira-3",
        "my-agent",
        vec![
            "jira:issue_created".to_string(),
            "jira:issue_updated".to_string(),
        ],
        None,
    );

    let body = serde_json::json!({
        "webhookEvent": "jira:issue_updated",
        "issue": { "key": "PROJ-44" }
    });

    let headers = HashMap::new();
    let result = trigger.receive_payload(body, &headers);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some(), "issue_updated should be accepted when subscribed");
}

#[test]
fn test_jira_trigger_with_secret_validation() {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "jira-secret-xyz";
    let trigger = JiraTrigger::new(
        "jira-4",
        "my-agent",
        vec!["jira:issue_created".to_string()],
        Some(secret.to_string()),
    );

    let body_str = r#"{"webhookEvent":"jira:issue_created","issue":{"key":"PROJ-1"}}"#;
    let body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    // HMAC must be computed over the same bytes the implementation uses (re-serialized JSON)
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(&body_bytes);
    let valid_sig = hex::encode(mac.finalize().into_bytes());

    let mut headers_ok = HashMap::new();
    headers_ok.insert("x-hub-signature".to_string(), valid_sig);
    let result_ok = trigger.receive_payload(body.clone(), &headers_ok);
    assert!(result_ok.is_ok(), "valid sig accepted");
    assert!(result_ok.unwrap().is_some());

    let mut headers_bad = HashMap::new();
    headers_bad.insert("x-hub-signature".to_string(), "badsig".to_string());
    let result_bad = trigger.receive_payload(body, &headers_bad);
    let rejected = result_bad.is_err() || result_bad.unwrap().is_none();
    assert!(rejected, "invalid sig rejected");
}
