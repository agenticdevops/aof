//! AOF Memory - High-performance memory backends for agent state
//!
//! This crate provides lock-free concurrent memory implementations using DashMap
//! for optimal read/write performance in multi-threaded agentic systems.
//!
//! ## Memory Backends
//!
//! - **InMemoryBackend**: Fast, ephemeral storage cleared on restart (default)
//! - **FileBackend**: Persistent JSON file storage that survives agent restarts
//!
//! ## Usage
//!
//! ```rust,no_run
//! use aof_memory::SimpleMemory;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // In-memory (ephemeral)
//! let memory = SimpleMemory::in_memory();
//!
//! // File-based (persistent)
//! let memory = SimpleMemory::file("./agent-memory.json").await?;
//! # Ok(())
//! # }
//! ```

pub mod backend;

// Re-export main types
pub use backend::file::FileBackend;
pub use backend::memory::InMemoryBackend;
pub use backend::SimpleMemory;

// Re-export core memory types
pub use aof_core::{Memory, MemoryBackend, MemoryEntry, MemoryQuery};
