// TDD RED phase — these tests are written BEFORE the implementation exists.
// They will fail to compile until trigger_event.rs is created.

use agentix_core::{TriggerEvent, TriggerRunRegistry, TriggerSource, TriggerTrait};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Mock trigger for registry tests
// ============================================================================

struct MockTrigger {
    id: String,
    source: TriggerSource,
}

#[async_trait]
impl TriggerTrait for MockTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        self.source.clone()
    }

    async fn start(
        &self,
        _sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), agentix_core::AgentixError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), agentix_core::AgentixError> {
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_trigger_event_cron() {
    let event = TriggerEvent::new(
        TriggerSource::Cron,
        json!({"expression": "0 9 * * 1"}),
        "cron-trigger-1",
    );

    assert!(matches!(event.source, TriggerSource::Cron));
    assert_eq!(event.payload["expression"], "0 9 * * 1");
    assert!(event.context.is_empty());
    assert_eq!(event.trigger_id, "cron-trigger-1");
}

#[test]
fn test_trigger_event_github() {
    let event = TriggerEvent::new(
        TriggerSource::GitHub,
        json!({"action": "opened", "number": 42}),
        "github-trigger-1",
    )
    .with_context("repo", "openagentix/openagentix");

    assert!(matches!(event.source, TriggerSource::GitHub));
    assert_eq!(event.payload["action"], "opened");
    assert_eq!(event.payload["number"], 42);
    assert!(event.context.get("repo").is_some());
    assert_eq!(event.context.get("repo").unwrap(), "openagentix/openagentix");
}

#[test]
fn test_trigger_event_serialization() {
    let event = TriggerEvent::new(
        TriggerSource::Webhook,
        json!({"key": "value", "count": 1}),
        "webhook-1",
    )
    .with_context("env", "production");

    // Serialize to JSON
    let serialized = serde_json::to_string(&event).expect("serialization failed");
    assert!(!serialized.is_empty());

    // Deserialize back
    let deserialized: TriggerEvent =
        serde_json::from_str(&serialized).expect("deserialization failed");

    assert!(matches!(deserialized.source, TriggerSource::Webhook));
    assert_eq!(deserialized.payload["key"], "value");
    assert_eq!(deserialized.context.get("env").unwrap(), "production");
    assert_eq!(deserialized.trigger_id, "webhook-1");
}

#[test]
fn test_trigger_source_display() {
    assert_eq!(TriggerSource::Cron.to_string(), "cron");
    assert_eq!(TriggerSource::Webhook.to_string(), "webhook");
    assert_eq!(TriggerSource::GitHub.to_string(), "github");
    assert_eq!(TriggerSource::Jira.to_string(), "jira");
    assert_eq!(TriggerSource::Slack.to_string(), "slack");
    assert_eq!(TriggerSource::Discord.to_string(), "discord");
    assert_eq!(TriggerSource::Telegram.to_string(), "telegram");
    assert_eq!(TriggerSource::Agent.to_string(), "agent");
    assert_eq!(TriggerSource::Cli.to_string(), "cli");
}

#[tokio::test]
async fn test_trigger_run_registry_register_and_get() {
    let mut registry = TriggerRunRegistry::new();

    let trigger = Arc::new(MockTrigger {
        id: "test-trigger".to_string(),
        source: TriggerSource::Webhook,
    });

    registry.register(trigger);

    assert!(registry.get("test-trigger").is_some());
    assert!(registry.get("missing").is_none());
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}
