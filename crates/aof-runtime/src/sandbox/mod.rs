//! Sandbox isolation and security hardening
//!
//! This module provides defense-in-depth container isolation through:
//! - Custom seccomp profiles per tool type
//! - Linux capability dropping (--cap-drop=ALL by default)
//! - Read-only root filesystems
//! - Resource limits (memory, CPU, PIDs)

pub mod capabilities;
pub mod seccomp;

pub use capabilities::CapabilityConfig;
pub use seccomp::{SeccompProfile, SeccompProfileManager};
