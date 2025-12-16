//! AOF Memory - High-performance memory backends for agent state
//!
//! This crate provides lock-free concurrent memory implementations using DashMap
//! for optimal read/write performance in multi-threaded agentic systems.

pub mod backend;

// Re-export main types
pub use backend::memory::InMemoryBackend;
pub use backend::SimpleMemory;

// Re-export core memory types
pub use aof_core::{Memory, MemoryBackend, MemoryEntry, MemoryQuery};
