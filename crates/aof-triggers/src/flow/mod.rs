//! Flow module - FlowRegistry and FlowRouter for AgentFlow management
//!
//! This module provides:
//! - `FlowRegistry` - Loads and manages AgentFlow configurations
//! - `FlowRouter` - Routes incoming trigger events to matching flows (legacy)
//! - `BindingRouter` - Routes events using FlowBinding resources (composable)

pub mod binding_router;
pub mod registry;
pub mod router;

pub use binding_router::{BindingMatch, BindingRouter, ResolvedExecutionContext};
pub use registry::FlowRegistry;
pub use router::{FlowMatch, FlowRouter, MatchReason};
