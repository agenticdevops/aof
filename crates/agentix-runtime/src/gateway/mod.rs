//! Gateway module — HTTP server for loading and running agents.
//!
//! The gateway is the core runtime service that:
//! 1. Scans an `agents_dir` and loads all agent definitions on startup
//! 2. Exposes a REST API for listing, registering, and running agents
//! 3. Streams `ReActEvent`s to HTTP clients via Server-Sent Events (SSE)
//!
//! # Quick Start
//!
//! ```no_run
//! use std::path::Path;
//! use agentix_runtime::gateway::Gateway;
//! use agentix_core::{GatewayConfig, WorkspaceConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = GatewayConfig::default();
//!     Gateway::start(config, None, Path::new("./agents")).await.unwrap();
//! }
//! ```

pub mod agent_manager;
pub mod api;
pub mod router;
pub mod run_store;
pub mod websocket;

pub use agent_manager::{AgentManager, AgentStatus, AgentSummary, LoadedAgent, RunState, RunStatus};
pub use router::Gateway;
pub use run_store::{RunRecord, RunStore};
pub use websocket::{EventBroadcaster, GatewayEvent};
