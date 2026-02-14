//! AOF Coordination Protocols - Multi-agent coordination and communication
//!
//! This crate implements coordination protocols for multi-agent communication,
//! including:
//! - Session tools: Async message queues between agent pairs
//! - Heartbeat protocol: Proactive health monitoring (Plan 02)
//! - Standup protocol: Daily status reports (Plan 03)
//! - Token metrics: Coordination overhead tracking (Plan 04)
//!
//! All coordination is opt-in per agent via CoordinationMode.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  aof-core (CoordinationActivity types)  │
//! └────────────────┬────────────────────────┘
//!                  │
//! ┌────────────────▼────────────────────────┐
//! │  aof-coordination (EventBroadcaster)    │
//! └────────────────┬────────────────────────┘
//!                  │
//! ┌────────────────▼────────────────────────┐
//! │  aof-coordination-protocols             │
//! │   - SessionTools (mpsc message queues)  │
//! │   - Heartbeat (health monitoring)       │
//! │   - Standup (daily reports)             │
//! │   - Metrics (token tracking)            │
//! └────────────────┬────────────────────────┘
//!                  │
//! ┌────────────────▼────────────────────────┐
//! │  aofctl serve (schedulers, routing)     │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Example: Session Tools
//!
//! ```rust,no_run
//! use aof_coordination_protocols::{SessionTools, SessionMessage, MessageType};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create session tools with 100-message capacity, 30-minute TTL
//!     let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));
//!
//!     // Register agents
//!     session_tools.register_agent("agent-a").await?;
//!     session_tools.register_agent("agent-b").await?;
//!
//!     // Agent A sends message to Agent B
//!     let message = SessionMessage::new(
//!         "agent-a",
//!         "agent-b",
//!         MessageType::Announcement,
//!         "Starting analysis",
//!         Duration::from_secs(30 * 60),
//!     );
//!     session_tools.send_message(message).await?;
//!
//!     // Agent B drains messages
//!     let messages = session_tools.drain_messages("agent-b").await;
//!     println!("Agent B received {} messages", messages.len());
//!
//!     Ok(())
//! }
//! ```

pub mod session_tools;
pub mod events;
pub mod error;
pub mod heartbeat;

// Re-exports
pub use session_tools::SessionTools;
pub use events::{
    SessionMessage, MessageType, AgentHealthStatus,
    StandupReport, CoordinationMode,
};
pub use error::CoordinationProtocolError;
pub use heartbeat::{HeartbeatScheduler, HeartbeatConfig, AgentHealthRecord};
