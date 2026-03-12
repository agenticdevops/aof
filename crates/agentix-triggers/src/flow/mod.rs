//! Flow module - FlowRegistry and FlowRouter for AgentFlow management
//!
//! This module provides:
//! - `FlowRegistry` - Loads and manages AgentFlow configurations
//! - `FlowRouter` - Simple flow lookup by name
//! - `FlowMatch` - Container for matched flow with metadata
//!
//! Note: Routing decisions are now made at the Trigger level via command bindings.
//! AgentFlows are pure workflow definitions without embedded triggers.

pub mod registry;
pub mod router;

pub use registry::FlowRegistry;
pub use router::{FlowMatch, FlowRouter, MatchReason};

// ---------------------------------------------------------------------------
// AgentFlow stub — full implementation deferred to Phase 15
// ---------------------------------------------------------------------------

/// Stub metadata for AgentFlow. Full implementation in Phase 15.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentFlowMetadata {
    pub name: String,
}

/// Stub AgentFlow type. The v1.0 AgentFlow runtime was removed in Phase 13.
/// This stub allows agentix-triggers to compile; full re-implementation in Phase 15.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentFlow {
    pub metadata: AgentFlowMetadata,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl AgentFlow {
    /// Stub validate — always succeeds in Phase 13.
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
