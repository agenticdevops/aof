/// AgentTrigger — fires when a parent agent delegates work to this agent.
///
/// This is a passive trigger: `start()` stores the sender channel so that the
/// gateway HTTP handler can call `create_event()` and dispatch it when a parent
/// agent calls `POST /api/v1/agents/:name/trigger`.
///
/// Agent-to-agent coordination in Phase 15 is fire-and-forget. Full result
/// routing (returning output back to the caller) is planned for Phase 16.
use agentix_core::{AgentixError, TriggerEvent, TriggerSource, TriggerTrait};
use async_trait::async_trait;
use std::sync::Arc;

/// Represents an agent-to-agent trigger.
///
/// When a parent agent needs to delegate a task, it calls the gateway's
/// `/api/v1/agents/:name/trigger` endpoint with a payload. The gateway
/// creates a TriggerEvent and dispatches it to the target agent.
pub struct AgentTrigger {
    id: String,
    target_agent: String,
    sender: tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<(String, TriggerEvent)>>>,
}

impl AgentTrigger {
    /// Create a new AgentTrigger.
    ///
    /// - `id`: unique trigger identifier (used as `trigger_id` in TriggerEvent)
    /// - `target_agent`: name of the agent this trigger fires
    pub fn new(id: impl Into<String>, target_agent: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            target_agent: target_agent.into(),
            sender: tokio::sync::Mutex::new(None),
        }
    }

    /// Name of the agent that this trigger fires.
    pub fn target_agent(&self) -> &str {
        &self.target_agent
    }

    /// Create a TriggerEvent representing a delegation from a parent agent.
    ///
    /// The `payload` contains the task description and any parameters.
    /// The `caller_agent` is the name of the parent agent that initiated the delegation.
    pub fn create_event(&self, payload: serde_json::Value, caller_agent: &str) -> TriggerEvent {
        TriggerEvent::new(TriggerSource::Agent, payload, &self.id)
            .with_context("caller_agent", caller_agent)
    }

    /// Dispatch a TriggerEvent to the registered channel.
    ///
    /// Called by the gateway HTTP handler when a trigger request arrives.
    pub async fn dispatch(&self, event: TriggerEvent) -> Result<(), AgentixError> {
        let guard = self.sender.lock().await;
        if let Some(sender) = &*guard {
            sender
                .send((self.target_agent.clone(), event))
                .await
                .map_err(|e| {
                    AgentixError::runtime(format!("AgentTrigger channel closed: {e}"))
                })?;
        }
        Ok(())
    }
}

#[async_trait]
impl TriggerTrait for AgentTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        TriggerSource::Agent
    }

    async fn start(
        &self,
        sender: tokio::sync::mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        // Passive trigger: store sender for use by gateway HTTP handler
        *self.sender.lock().await = Some(sender);
        Ok(())
    }

    async fn stop(&self) -> Result<(), AgentixError> {
        *self.sender.lock().await = None;
        Ok(())
    }
}
