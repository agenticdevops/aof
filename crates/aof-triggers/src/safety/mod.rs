//! Safety Layer - Tool Classification and Platform Policies
//!
//! This module provides:
//! - Tool classification by action class (read/write/delete/dangerous)
//! - Platform-aware policy enforcement (stricter on mobile platforms)
//! - Approval workflow integration for sensitive operations
//!
//! Design Philosophy:
//! - Safety-first: Default to most restrictive classification when unknown
//! - Platform-aware: Different trust levels for different platforms
//! - Human-in-the-loop: Require approval for risky operations
//! - Transparent: Clear feedback on why operations are blocked

mod classifier;
mod policy;
mod context;

pub use classifier::{
    ToolClassifier, ActionClass, ClassificationResult, ToolClassifications,
};
pub use policy::{
    PlatformPolicy, PolicyDecision, PolicyEngine,
};
pub use context::{
    SafetyContext, SafetyConfig,
};
