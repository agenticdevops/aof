//! AOF Personas - Agent personality system for workspace files
//!
//! This crate provides loaders, validators, and caching for agent persona
//! configuration files (AGENTS.md and SOUL.md). These workspace files define
//! agent identities, communication styles, and behavioral boundaries.
//!
//! # Architecture
//!
//! ```text
//! workspace/AGENTS.md ──┐
//!                       ├──► AgentLoader ──► Vec<Agent>
//!                       │                        │
//! workspace/SOUL.md  ───┤                        ▼
//!                       └──► SoulLoader ──► HashMap<String, Soul>
//!                                                │
//!                                                ▼
//!                                         validate_personas()
//!                                                │
//!                                                ▼
//!                                     PersonaWatcher (file watch + reload)
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_personas::{AgentLoader, SoulLoader, validate_personas};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let agents = AgentLoader::load_from_file("workspace/AGENTS.md").await?;
//!     let souls = SoulLoader::load_from_file("workspace/SOUL.md").await?;
//!     validate_personas(&agents, &souls)?;
//!     Ok(())
//! }
//! ```

pub mod loader;
pub mod types;
pub mod validation;
pub mod watcher;

// Re-export primary types
pub use loader::{AgentCache, AgentLoader, SoulLoader};
pub use types::{Agent, AgentsFile, Soul, SoulFrontmatter};
pub use validation::{validate_agents, validate_personas, validate_souls};
pub use watcher::{PersonaUpdate, PersonaWatcher};
