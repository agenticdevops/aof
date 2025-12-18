//! Flow module - FlowRegistry and FlowRouter for AgentFlow management
//!
//! This module provides:
//! - `FlowRegistry` - Loads and manages AgentFlow configurations
//! - `FlowRouter` - Routes incoming trigger events to matching flows

pub mod registry;
pub mod router;

pub use registry::FlowRegistry;
pub use router::{FlowMatch, FlowRouter, MatchReason};
