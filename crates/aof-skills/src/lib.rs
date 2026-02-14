//! # AOF Skills
//!
//! Skills platform for AOF - codify tribal knowledge as executable agent capabilities.
//!
//! Skills are defined as `SKILL.md` files with YAML frontmatter containing metadata
//! and markdown content with instructions. This module provides:
//!
//! - Skill loading from workspace, enterprise registry, and bundled sources
//! - Frontmatter parsing with metadata extraction
//! - Requirements gating (binaries, env vars, config paths, OS)
//! - Hot-reload via file watching
//! - Prompt building for model consumption
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use aof_skills::{SkillRegistry, SkillConfig};
//!
//! #[tokio::main]
//! async fn main() -> aof_skills::Result<()> {
//!     // Create a registry with default config
//!     let registry = SkillRegistry::default_registry();
//!
//!     // Load all skills
//!     registry.load().await?;
//!
//!     // Get eligible skills (requirements met)
//!     let skills = registry.eligible().await;
//!
//!     // Build prompt for agent
//!     let prompt = aof_skills::build_skills_prompt(&skills);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## SKILL.md Format
//!
//! ```markdown
//! ---
//! name: k8s-debug
//! description: "Kubernetes pod debugging and troubleshooting"
//! metadata:
//!   emoji: "🐳"
//!   requires:
//!     bins: ["kubectl"]
//!   tags: ["kubernetes", "debugging"]
//! ---
//!
//! # Kubernetes Debug Skill
//!
//! Instructions for the agent...
//! ```

mod error;
mod frontmatter;
mod loader;
mod registry;
mod requirements;
mod types;
mod watcher;

pub use error::SkillError;
pub use frontmatter::{has_frontmatter, parse_frontmatter, ParsedSkill, SkillFrontmatter};
pub use loader::{build_skills_prompt, SkillLoader};
pub use registry::{AgentSkillsValidator, SkillRegistry, ValidationReport};
pub use requirements::{EligibilityContext, RequirementCheck, RequirementChecker};
pub use types::*;
pub use watcher::{SkillWatcher, SkillWatcherBuilder};

/// Re-export for convenience
pub type Result<T> = std::result::Result<T, SkillError>;
