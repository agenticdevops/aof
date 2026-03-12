//! Built-in skill pack registry for OpenAgentiX.
//!
//! Built-in skill packs are embedded into the binary at compile time using
//! `include_str!`. They ship with every `agentix` binary so agents can
//! reference them without filesystem access.
//!
//! # Usage
//!
//! ```rust
//! use agentix_core::skills::SkillRegistry;
//!
//! let registry = SkillRegistry::new();
//! for pack in registry.builtin_packs() {
//!     println!("{}: {}", pack.name, pack.description);
//! }
//! ```

/// A single built-in skill pack embedded in the binary.
#[derive(Debug, Clone)]
pub struct BuiltinSkillPack {
    /// Skill pack identifier (lowercase, hyphen-separated, e.g. "kubernetes").
    pub name: &'static str,
    /// Human-readable one-line description.
    pub description: &'static str,
    /// Full SKILL.md content embedded at compile time.
    pub content: &'static str,
}

/// Registry of all built-in skill packs shipped with the `agentix` binary.
///
/// Skill packs are embedded at compile time from the `skill-packs/` directory
/// at the repository root. They are injected into an agent's system prompt
/// when the agent's `skills/` directory contains a matching subdirectory.
///
/// ## Selection
///
/// Skill injection is **deterministic** — the agent's `skills/<name>/SKILL.md`
/// file is always loaded as a [`SkillEntry`][crate::SkillEntry] and
/// injected into the system prompt. No LLM call is made for skill selection.
pub struct SkillRegistry {
    packs: Vec<BuiltinSkillPack>,
}

impl SkillRegistry {
    /// Create a new registry pre-loaded with all 8 built-in skill packs.
    pub fn new() -> Self {
        Self {
            packs: vec![
                BuiltinSkillPack {
                    name: "aws",
                    description: "AWS cloud infrastructure management (EC2, S3, IAM, CloudWatch)",
                    content: include_str!("../../../skill-packs/aws/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "kubernetes",
                    description: "Kubernetes cluster management and workload operations (kubectl)",
                    content: include_str!("../../../skill-packs/kubernetes/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "terraform",
                    description: "Terraform infrastructure-as-code management (plan, apply, state)",
                    content: include_str!("../../../skill-packs/terraform/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "docker",
                    description: "Docker container and image management (build, run, inspect)",
                    content: include_str!("../../../skill-packs/docker/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "git",
                    description: "Git version control operations (commit, branch, rebase, merge)",
                    content: include_str!("../../../skill-packs/git/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "database",
                    description: "Database operations and query optimization (PostgreSQL, MySQL)",
                    content: include_str!("../../../skill-packs/database/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "security",
                    description: "Security scanning, vulnerability assessment, and hardening",
                    content: include_str!("../../../skill-packs/security/SKILL.md"),
                },
                BuiltinSkillPack {
                    name: "observability",
                    description: "Observability, monitoring, log analysis, and distributed tracing",
                    content: include_str!("../../../skill-packs/observability/SKILL.md"),
                },
            ],
        }
    }

    /// Return all 8 built-in skill packs.
    pub fn builtin_packs(&self) -> &[BuiltinSkillPack] {
        &self.packs
    }

    /// Look up a specific built-in skill pack by name (case-sensitive, exact match).
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinSkillPack> {
        self.packs.iter().find(|p| p.name == name)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
