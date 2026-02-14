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

impl Agent {
    /// Create a new agent with all required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: impl Into<String>,
        avatar: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role: role.into(),
            avatar: avatar.into(),
            personality_traits: Vec::new(),
            can: Vec::new(),
            cannot: Vec::new(),
            skills: Vec::new(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_construction() {
        let agent = Agent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            role: "Tester".to_string(),
            avatar: "\u{1F916}".to_string(),
            personality_traits: vec!["curious".to_string()],
            can: vec!["test things".to_string()],
            cannot: vec!["break things".to_string()],
            skills: vec!["testing".to_string()],
        };
        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.name, "Test Agent");

        // Test Clone
        let cloned = agent.clone();
        assert_eq!(cloned.id, agent.id);

        // Test Debug
        let debug_str = format!("{:?}", agent);
        assert!(debug_str.contains("test-agent"));
    }

    #[test]
    fn test_agent_new_constructor() {
        let agent = Agent::new("k8s-monitor", "K8s Monitor", "Infra Specialist", "\u{1F916}");
        assert_eq!(agent.id, "k8s-monitor");
        assert!(agent.personality_traits.is_empty());
        assert!(agent.skills.is_empty());
    }

    #[test]
    fn test_soul_from_frontmatter() {
        let fm = SoulFrontmatter {
            id: "test".to_string(),
            communication_style: "formal".to_string(),
            tone: "calm".to_string(),
            values: vec!["reliability".to_string()],
            personality_summary: "A test agent".to_string(),
            boundaries: vec!["Never break things".to_string()],
            default_intro: "Hello, I am a test agent".to_string(),
        };
        let soul = Soul::from(fm);
        assert_eq!(soul.id, "test");
        assert_eq!(soul.communication_style, "formal");
        assert!(soul.communication_guide.is_empty());
    }

    #[test]
    fn test_agent_serialization_roundtrip() {
        let agent = Agent {
            id: "test".to_string(),
            name: "Test".to_string(),
            role: "Tester".to_string(),
            avatar: "\u{1F916}".to_string(),
            personality_traits: vec!["curious".to_string()],
            can: vec!["test".to_string()],
            cannot: vec!["break".to_string()],
            skills: vec!["testing".to_string()],
        };

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, agent.id);
        assert_eq!(deserialized.skills, agent.skills);
    }

    #[test]
    fn test_agents_file_yaml_parsing() {
        let yaml = r#"
agents:
  - id: test
    name: Test
    role: Tester
    avatar: "\U0001F916"
    personality_traits: [curious]
    can: [test]
    cannot: [break]
    skills: [testing]
"#;
        let file: AgentsFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(file.agents.len(), 1);
        assert_eq!(file.agents[0].id, "test");
    }
}
