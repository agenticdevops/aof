//! AOF Runtime - Agent execution runtime with task orchestration
//!
//! This crate provides the core execution engine for AOF agents, handling:
//! - Agent lifecycle management
//! - Tool call execution loops
//! - Context management
//! - Error handling and recovery
//! - Task orchestration

pub mod executor;
pub mod orchestrator;
pub mod task;

pub use executor::{AgentExecutor, Runtime, StreamEvent};
pub use orchestrator::RuntimeOrchestrator;
pub use task::{Task, TaskHandle, TaskStatus};

// Re-export core types
pub use aof_core::{AofError, AofResult};
