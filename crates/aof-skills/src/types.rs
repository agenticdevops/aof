//! Core types for the AOF Skills platform.
//!
//! Skills are defined as `SKILL.md` files with YAML frontmatter containing metadata
//! and markdown content with instructions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A skill definition loaded from SKILL.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill name (e.g., "k8s-debug", "prometheus-query")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Optional homepage URL for more documentation
    pub homepage: Option<String>,

    /// Markdown content after frontmatter (the actual skill instructions)
    pub content: String,

    /// Skill metadata from frontmatter
    pub metadata: SkillMetadata,

    /// Where this skill was loaded from
    pub source: SkillSource,
}

/// Metadata extracted from SKILL.md frontmatter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Optional emoji for display
    pub emoji: Option<String>,

    /// Requirements that must be met for skill to be eligible
    #[serde(default)]
    pub requires: SkillRequirements,

    /// Install specifications for missing dependencies
    #[serde(default)]
    pub install: Vec<InstallSpec>,

    /// OS restrictions (e.g., ["darwin", "linux"])
    pub os: Option<Vec<String>>,

    /// If true, skill is always loaded regardless of requirements
    #[serde(default)]
    pub always: bool,

    /// Tags for categorization and search
    #[serde(default)]
    pub tags: Vec<String>,

    /// Version string
    pub version: Option<String>,

    /// Author information
    pub author: Option<String>,

    /// License
    pub license: Option<String>,
}

/// Requirements that must be satisfied for a skill to be eligible
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillRequirements {
    /// Required binaries that must be in PATH
    #[serde(default)]
    pub bins: Vec<String>,

    /// At least one of these binaries must be available
    #[serde(default)]
    pub any_bins: Vec<String>,

    /// Required environment variables
    #[serde(default)]
    pub env: Vec<String>,

    /// Required config file paths
    #[serde(default)]
    pub config: Vec<String>,
}

/// Install specification for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSpec {
    /// Unique identifier for this installer
    pub id: String,

    /// Type of installer
    pub kind: InstallerKind,

    /// Package name or formula
    pub package: String,

    /// Binaries provided by this package
    #[serde(default)]
    pub bins: Vec<String>,

    /// Optional URL for manual instructions
    pub url: Option<String>,
}

/// Supported installer types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstallerKind {
    /// Homebrew (macOS/Linux)
    Brew,
    /// apt-get (Debian/Ubuntu)
    Apt,
    /// dnf/yum (Fedora/RHEL)
    Dnf,
    /// npm (Node.js)
    Npm,
    /// pip (Python)
    Pip,
    /// cargo (Rust)
    Cargo,
    /// Manual installation with URL
    Manual,
}

/// Where a skill was loaded from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillSource {
    /// Bundled with AOF distribution
    Bundled,

    /// From public skills registry (OpsSkillsHub)
    PublicRegistry {
        version: String,
    },

    /// From enterprise/organization registry
    EnterpriseRegistry {
        org: String,
        version: String,
    },

    /// From local workspace (highest precedence)
    Workspace {
        path: PathBuf,
    },
}

impl SkillSource {
    /// Returns the precedence of this source (higher = takes priority)
    pub fn precedence(&self) -> u8 {
        match self {
            SkillSource::Bundled => 0,
            SkillSource::PublicRegistry { .. } => 1,
            SkillSource::EnterpriseRegistry { .. } => 2,
            SkillSource::Workspace { .. } => 3,
        }
    }
}

/// Result of searching for skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    /// The matched skill
    pub skill: Skill,

    /// Search relevance score (0.0 - 1.0)
    pub score: f32,

    /// Matched terms
    pub matches: Vec<String>,
}

/// Configuration for skill loading
#[derive(Debug, Clone, Default)]
pub struct SkillConfig {
    /// Directory for workspace-local skills
    pub workspace_dir: Option<PathBuf>,

    /// URL for enterprise registry
    pub enterprise_url: Option<String>,

    /// URL for public registry
    pub public_url: Option<String>,

    /// Enable hot-reload via file watching
    pub watch: bool,

    /// Directories for bundled skills
    pub bundled_dirs: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_precedence() {
        assert!(SkillSource::Workspace { path: PathBuf::new() }.precedence()
            > SkillSource::EnterpriseRegistry { org: "test".into(), version: "1.0".into() }.precedence());
        assert!(SkillSource::EnterpriseRegistry { org: "test".into(), version: "1.0".into() }.precedence()
            > SkillSource::PublicRegistry { version: "1.0".into() }.precedence());
        assert!(SkillSource::PublicRegistry { version: "1.0".into() }.precedence()
            > SkillSource::Bundled.precedence());
    }

    #[test]
    fn test_skill_requirements_default() {
        let reqs = SkillRequirements::default();
        assert!(reqs.bins.is_empty());
        assert!(reqs.any_bins.is_empty());
        assert!(reqs.env.is_empty());
        assert!(reqs.config.is_empty());
    }
}
