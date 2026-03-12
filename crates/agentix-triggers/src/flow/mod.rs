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
