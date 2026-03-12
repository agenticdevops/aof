// TDD RED phase — written before CliTrigger implementation.

use agentix_core::{TriggerSource, TriggerTrait};
use agentix_triggers::CliTrigger;

#[test]
fn test_cli_trigger_event() {
    let trigger = CliTrigger::new("trig-cli", "my-agent");
    let event = trigger.create_event("check replication lag");

    assert!(matches!(event.source, TriggerSource::Cli));
    assert_eq!(event.payload["input"], "check replication lag");
    assert_eq!(event.context.get("invocation").map(|s| s.as_str()), Some("cli"));
    assert_eq!(event.trigger_id, "trig-cli");
}

#[test]
fn test_cli_trigger_source() {
    let trigger = CliTrigger::new("trig-cli-2", "my-agent");
    assert!(matches!(trigger.source(), TriggerSource::Cli));
}
