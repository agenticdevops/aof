// Placeholder for AgentCreator implementation
// Will be implemented in task 06-02-04

use async_trait::async_trait;
use anyhow::Result;
use crate::specialists::traits::{Specialist, SpecialistOutput};
use crate::types::{IntentClassification, ConversationSession};

pub struct AgentCreator;

#[async_trait]
impl Specialist for AgentCreator {
    async fn handle(
        &self,
        _intent: &IntentClassification,
        _session: &ConversationSession,
    ) -> Result<SpecialistOutput> {
        todo!("AgentCreator implementation coming in task 06-02-04")
    }

    fn name(&self) -> &str {
        "agent_creator"
    }
}
