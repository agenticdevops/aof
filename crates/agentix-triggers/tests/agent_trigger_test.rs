// TDD RED phase — written before AgentTrigger implementation.

use agentix_core::{TriggerSource, TriggerTrait};
use agentix_triggers::AgentTrigger;

#[test]
fn test_agent_trigger_new() {
    let trigger = AgentTrigger::new("trig-1", "target-agent");
    assert_eq!(trigger.trigger_id(), "trig-1");
    assert!(matches!(trigger.source(), TriggerSource::Agent));
}

#[test]
fn test_agent_trigger_event() {
    let trigger = AgentTrigger::new("trig-2", "target-agent");
    let payload = serde_json::json!({"task": "analyze logs", "timerange": "24h"});
    let event = trigger.create_event(payload, "parent-agent");

    assert!(matches!(event.source, TriggerSource::Agent));
    assert_eq!(event.payload["task"], "analyze logs");
    assert_eq!(event.payload["timerange"], "24h");
    assert_eq!(event.context.get("caller_agent").map(|s| s.as_str()), Some("parent-agent"));
    assert_eq!(event.trigger_id, "trig-2");
}
