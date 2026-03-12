/// CliTrigger — represents a one-shot agent run initiated from the command line.
///
/// The CLI sends a TriggerEvent{source: Cli, payload: {input: "..."}} to the
/// gateway's `/api/v1/agents/:name/trigger` endpoint. The gateway records the
/// run with `trigger_source: cli` so it appears in `agentix runs` history.
///
/// This is a synchronous (non-background) trigger: there is no background task.
/// `start()` is a no-op; the CLI invokes the gateway directly.
use agentix_core::{AgentixError, TriggerEvent, TriggerSource, TriggerTrait};
use async_trait::async_trait;

/// A CLI-initiated one-shot agent trigger.
pub struct CliTrigger {
    id: String,
    agent_name: String,
}

impl CliTrigger {
    /// Create a new CliTrigger.
    ///
    /// - `id`: unique trigger identifier
    /// - `agent_name`: name of the agent to run
    pub fn new(id: impl Into<String>, agent_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            agent_name: agent_name.into(),
        }
    }

    /// Agent name this trigger fires.
    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    /// Create a TriggerEvent for a CLI-initiated run.
    ///
    /// The `input` is the raw text the user passed via `--input`.
    pub fn create_event(&self, input: &str) -> TriggerEvent {
        TriggerEvent::new(
            TriggerSource::Cli,
            serde_json::json!({ "input": input }),
            &self.id,
        )
        .with_context("invocation", "cli")
    }
}

#[async_trait]
impl TriggerTrait for CliTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        TriggerSource::Cli
    }

    /// No-op: CLI triggers fire synchronously, not via background task.
    async fn start(
        &self,
        _sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        Ok(())
    }
}
