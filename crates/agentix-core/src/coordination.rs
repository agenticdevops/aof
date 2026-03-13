//! Agent coordination primitives — inbox, delegation messages, results, and the coordinator protocol.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::AgentixResult;

/// Capacity of each agent's bounded inbox.
pub const INBOX_CAPACITY: usize = 256;

/// A task delegated from a coordinator to a specialist agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationMessage {
    /// Unique delegation ID (UUID recommended).
    pub id: String,
    /// Name of the agent that issued this delegation.
    pub from_agent: String,
    /// Human-readable task description (used as LLM input).
    pub task: String,
    /// Structured parameters for the task.
    pub payload: serde_json::Value,
    /// Name of the agent that should receive the result (usually the coordinator).
    pub reply_to: Option<String>,
}

/// Outcome of a completed delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    /// ID of the delegation this result corresponds to.
    pub delegation_id: String,
    /// Name of the agent that produced this result.
    pub from_agent: String,
    /// Natural-language output produced by the specialist.
    pub output: String,
    /// Whether the delegation succeeded or failed.
    pub status: DelegationStatus,
    /// When the specialist completed the task.
    pub completed_at: DateTime<Utc>,
}

/// Status of a completed delegation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    Success,
    Failure,
}

/// Bounded message queue for an agent.
///
/// Each registered agent has one inbox. Coordinators push DelegationMessages
/// into it; the agent's run loop pops them and executes.
pub struct AgentInbox {
    tx: mpsc::Sender<DelegationMessage>,
    rx: tokio::sync::Mutex<mpsc::Receiver<DelegationMessage>>,
}

impl AgentInbox {
    /// Create a new inbox with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }

    /// Create a new inbox with the default INBOX_CAPACITY.
    pub fn default_capacity() -> Self {
        Self::new(INBOX_CAPACITY)
    }

    /// Send a delegation message to this inbox.
    /// Returns an error if the inbox is full or the receiver has been dropped.
    pub async fn send(&self, msg: DelegationMessage) -> AgentixResult<()> {
        self.tx.send(msg).await.map_err(|e| {
            crate::AgentixError::runtime(format!("AgentInbox send failed: {e}"))
        })
    }

    /// Receive the next delegation message from this inbox.
    /// Returns None if the channel is closed.
    pub async fn receive(&self) -> Option<DelegationMessage> {
        self.rx.lock().await.recv().await
    }

    /// Get a clone of the sender for external use (e.g., from coordinator thread).
    pub fn sender(&self) -> mpsc::Sender<DelegationMessage> {
        self.tx.clone()
    }
}

/// Protocol for coordinating agents — delegate tasks and collect results.
#[async_trait]
pub trait CoordinatorProtocol: Send + Sync {
    /// Delegate a task to a specialist agent and await the result.
    ///
    /// - `target`: name of the specialist agent
    /// - `task`: natural-language task description
    /// - `payload`: structured parameters
    async fn delegate(
        &self,
        target: &str,
        task: &str,
        payload: serde_json::Value,
    ) -> AgentixResult<DelegationResult>;

    /// Delegate tasks to multiple specialist agents in parallel.
    /// Returns results in the same order as the input targets.
    async fn delegate_parallel(
        &self,
        delegations: Vec<(String, String, serde_json::Value)>,
    ) -> AgentixResult<Vec<DelegationResult>>;
}
