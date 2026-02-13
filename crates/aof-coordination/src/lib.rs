//! AOF Coordination - Real-time agent event streaming and coordination
//!
//! This crate provides the coordination layer for multi-agent systems, enabling:
//! - Event broadcasting via tokio::sync::broadcast (pub/sub pattern)
//! - Session state persistence across daemon restarts
//! - Convenience constructors for common agent activities
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Agents    │ emit ActivityEvents
//! └──────┬──────┘
//!        │
//!        v
//! ┌─────────────────────┐
//! │ CoordinationEvent   │ wraps with routing metadata
//! │  (agent_id,         │
//! │   session_id,       │
//! │   event_id)         │
//! └──────┬──────────────┘
//!        │
//!        v
//! ┌─────────────────────┐
//! │ EventBroadcaster    │ broadcast to N subscribers
//! │  (tokio::broadcast) │
//! └──────┬──────────────┘
//!        │
//!        v
//! ┌─────────────────────┐
//! │  WebSocket Clients  │ (Mission Control UI, CLIs, etc.)
//! └─────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_coordination::{EventBroadcaster, SessionPersistence, CoordinationEvent};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create event broadcaster
//!     let broadcaster = EventBroadcaster::new(1000);
//!
//!     // Subscribe to events
//!     let mut receiver = broadcaster.subscribe();
//!
//!     // Emit events
//!     let event = CoordinationEvent::agent_started("agent-1", "session-123");
//!     broadcaster.emit(event);
//!
//!     // Receive events
//!     if let Ok(event) = receiver.recv().await {
//!         println!("Received event from {}", event.agent_id);
//!     }
//!
//!     // Persist session state
//!     let persistence = SessionPersistence::new(PathBuf::from("./data")).await?;
//!     // ... save/restore session state ...
//!
//!     Ok(())
//! }
//! ```

pub mod broadcaster;
pub mod decision_log;
pub mod events;
pub mod persistence;

// Re-export core types
pub use aof_core::coordination::{
    AgentState, AgentStatus, CoordinationEvent, DecisionLogEntry, SessionState, TaskInfo, TaskStatus,
};
pub use broadcaster::EventBroadcaster;
pub use decision_log::{DecisionLogger, DecisionSearch};
pub use persistence::SessionPersistence;
