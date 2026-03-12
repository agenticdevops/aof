// TDD RED phase — written BEFORE CronTrigger implementation.
// Tests will fail to compile until cron.rs is created.

use agentix_core::{TriggerSource, TriggerTrait};
use agentix_triggers::CronTrigger;
use tokio::sync::mpsc;

#[test]
fn test_cron_trigger_new() {
    let trigger = CronTrigger::new("0 9 * * 1", "my-agent", "trigger-1")
        .expect("should construct with valid cron expression");
    assert_eq!(trigger.trigger_id(), "trigger-1");
    assert!(matches!(trigger.source(), TriggerSource::Cron));
}

#[test]
fn test_cron_trigger_invalid_expression() {
    let result = CronTrigger::new("not-a-cron", "my-agent", "trigger-bad");
    assert!(result.is_err(), "invalid cron expression should fail");
}

#[tokio::test]
async fn test_cron_trigger_next_tick() {
    // Use a 5-field cron that fires "every minute" to test the channel
    // We mock time by checking the event payload is sent
    let (tx, mut rx) = mpsc::channel(4);

    // "0/1 * * * * * *" — every second if seconds are supported
    // Fall back to testing that start() completes without error for a valid expression
    let trigger = CronTrigger::new("0 * * * *", "test-agent", "tick-trigger")
        .expect("should construct");

    // Start the trigger (should not block or fail)
    let result = trigger.start(tx).await;
    assert!(result.is_ok(), "start() should succeed: {:?}", result);

    // Stop the trigger
    let stop_result = trigger.stop().await;
    assert!(stop_result.is_ok(), "stop() should succeed");

    // After stopping, the channel should not block indefinitely
    // (we just verify stop works correctly)
    drop(rx);
}

#[test]
fn test_cron_event_payload() {
    use serde_json::json;

    let trigger = CronTrigger::new("0 9 * * 1", "my-agent", "weekly")
        .expect("valid expression");

    let event = trigger.create_test_event();
    assert!(matches!(event.source, TriggerSource::Cron));
    assert!(event.payload.get("expression").is_some(), "payload must have expression field");
    assert_eq!(event.payload["expression"], json!("0 9 * * 1"));
    assert!(event.payload.get("scheduled_at").is_some(), "payload must have scheduled_at field");
}
