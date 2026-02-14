//! Type definitions for agent personas
//!
//! Defines the core data structures for AGENTS.md (agent roster) and
//! SOUL.md (personality guidance) workspace files.

use serde::{Deserialize, Serialize};

/// Root structure for AGENTS.md YAML content
///
/// Contains the full list of agents defined in a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsFile {
    /// List of agent definitions
    pub agents: Vec<Agent>,
}

/// An agent definition from AGENTS.md
///
/// Represents a single agent with identity, capabilities, and personality traits.
/// Each agent has a unique id, display name, role description, emoji avatar,
/// personality traits, capability boundaries (can/cannot), and skill references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier (lowercase-hyphenated, e.g., "k8s-monitor")
    pub id: String,
    /// Display name (e.g., "Kubernetes Monitor")
    pub name: String,
    /// Role description (e.g., "Infrastructure Specialist")
    pub role: String,
    /// Emoji avatar (single emoji character, e.g., "🤖")
    pub avatar: String,
    /// Personality adjectives (e.g., ["methodical", "detail-oriented", "proactive"])
    #[serde(default)]
    pub personality_traits: Vec<String>,
    /// What the agent can do (capability list)
    #[serde(default)]
    pub can: Vec<String>,
    /// What the agent cannot do (boundary list)
    #[serde(default)]
    pub cannot: Vec<String>,
    /// Skill/tool references (link to TOOLS.md entries)
    #[serde(default)]
    pub skills: Vec<String>,
}

/// YAML frontmatter extracted from a SOUL.md agent section
///
/// Contains structured personality metadata parsed from the YAML
/// code block within each agent's section in SOUL.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulFrontmatter {
    /// Agent ID (must match an id in AGENTS.md)
    pub id: String,
    /// Communication style descriptor (e.g., "formal-technical")
    pub communication_style: String,
    /// Tone descriptor (e.g., "calm-professional")
    pub tone: String,
    /// Core values that guide agent decisions
    pub values: Vec<String>,
    /// One-line personality description
    pub personality_summary: String,
    /// Hard behavioral rules for this agent
    pub boundaries: Vec<String>,
    /// Introduction message when agent joins a squad
    pub default_intro: String,
}

/// Complete soul definition combining frontmatter and prose
///
/// Extends SoulFrontmatter with the communication guide prose section
/// that follows the YAML block in SOUL.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soul {
    /// Agent ID (must match an id in AGENTS.md)
    pub id: String,
    /// Communication style descriptor
    pub communication_style: String,
    /// Tone descriptor
    pub tone: String,
    /// Core values
    pub values: Vec<String>,
    /// One-line personality description
    pub personality_summary: String,
    /// Hard behavioral rules
    pub boundaries: Vec<String>,
    /// Introduction message
    pub default_intro: String,
    /// Free-form communication style guide (prose from SOUL.md)
    pub communication_guide: String,
}

impl From<SoulFrontmatter> for Soul {
    fn from(fm: SoulFrontmatter) -> Self {
        Soul {
            id: fm.id,
            communication_style: fm.communication_style,
            tone: fm.tone,
            values: fm.values,
            personality_summary: fm.personality_summary,
            boundaries: fm.boundaries,
            default_intro: fm.default_intro,
            communication_guide: String::new(),
        }
    }
}
