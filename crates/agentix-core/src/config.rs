//! Configuration API types and parsers
//!
//! This module provides types and functions for loading and parsing
//! workspace configuration files (AGENTS.md, TOOLS.md) into JSON
//! for the Mission Control UI.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Agent configuration from AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    pub personality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub skills: Vec<String>,
}

/// Tool configuration from TOOLS.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Container for agents list in AGENTS.md
#[derive(Debug, Deserialize)]
struct AgentsFile {
    agents: Vec<AgentConfig>,
}

/// Container for tools list in TOOLS.md
#[derive(Debug, Deserialize)]
struct ToolsFile {
    tools: Vec<ToolConfig>,
}

/// Parse AGENTS.md file and return list of agent configs
///
/// # Errors
///
/// Returns error if file cannot be read or contains invalid YAML.
/// Uses `serde_path_to_error` to provide exact field paths on parse errors.
///
/// # Example
///
/// ```rust,no_run
/// use agentix_core::config::parse_agents_md;
///
/// let agents = parse_agents_md("workspace/AGENTS.md")?;
/// println!("Loaded {} agents", agents.len());
/// # Ok::<(), agentix_core::AofError>(())
/// ```
pub fn parse_agents_md<P: AsRef<Path>>(path: P) -> crate::AofResult<Vec<AgentConfig>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| crate::AofError::config(format!(
            "Failed to read AGENTS.md at {}: {}",
            path.display(),
            e
        )))?;

    let deserializer = serde_yaml::Deserializer::from_str(&content);
    let agents_file: AgentsFile = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| crate::AofError::config(format!(
            "Failed to parse AGENTS.md at {}\nField: {}\nError: {}",
            path.display(),
            e.path(),
            e.inner()
        )))?;

    Ok(agents_file.agents)
}

/// Parse TOOLS.md file and return list of tool configs
///
/// # Errors
///
/// Returns error if file cannot be read or contains invalid YAML.
/// Uses `serde_path_to_error` to provide exact field paths on parse errors.
///
/// # Example
///
/// ```rust,no_run
/// use agentix_core::config::parse_tools_md;
///
/// let tools = parse_tools_md("workspace/TOOLS.md")?;
/// println!("Loaded {} tools", tools.len());
/// # Ok::<(), agentix_core::AofError>(())
/// ```
pub fn parse_tools_md<P: AsRef<Path>>(path: P) -> crate::AofResult<Vec<ToolConfig>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| crate::AofError::config(format!(
            "Failed to read TOOLS.md at {}: {}",
            path.display(),
            e
        )))?;

    let deserializer = serde_yaml::Deserializer::from_str(&content);
    let tools_file: ToolsFile = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| crate::AofError::config(format!(
            "Failed to parse TOOLS.md at {}\nField: {}\nError: {}",
            path.display(),
            e.path(),
            e.inner()
        )))?;

    Ok(tools_file.tools)
}

/// Calculate version hash from configuration files
///
/// Returns SHA256 hash of concatenated AGENTS.md + TOOLS.md content.
/// Used for cache invalidation on the frontend.
///
/// # Example
///
/// ```rust,no_run
/// use agentix_core::config::version_hash;
///
/// let hash = version_hash("workspace/AGENTS.md", "workspace/TOOLS.md")?;
/// assert_eq!(hash.len(), 64); // SHA256 hex string
/// # Ok::<(), agentix_core::AofError>(())
/// ```
pub fn version_hash<P: AsRef<Path>>(agents_path: P, tools_path: P) -> crate::AofResult<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    // Hash AGENTS.md content (or empty string if missing)
    if let Ok(content) = std::fs::read(agents_path.as_ref()) {
        hasher.update(&content);
    }

    // Hash TOOLS.md content (or empty string if missing)
    if let Ok(content) = std::fs::read(tools_path.as_ref()) {
        hasher.update(&content);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_agents_md_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let mut file = std::fs::File::create(&agents_file).unwrap();
        write!(file, r#"
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    personality: Methodical and detail-oriented
    avatar: 🤖
    skills:
      - kubectl
      - pod-debugging
  - id: log-analyzer
    name: Log Analyzer
    role: Debugging Expert
    personality: Curious investigator
    skills:
      - log-parsing
      - pattern-matching
"#).unwrap();

        let agents = parse_agents_md(&agents_file).unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].id, "k8s-monitor");
        assert_eq!(agents[0].name, "Kubernetes Monitor");
        assert_eq!(agents[0].skills.len(), 2);
        assert_eq!(agents[1].id, "log-analyzer");
    }

    #[test]
    fn test_parse_agents_md_malformed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let mut file = std::fs::File::create(&agents_file).unwrap();
        write!(file, r#"
agents:
  - id: test
    name: Test Agent
    skills: not-an-array
"#).unwrap();

        let result = parse_agents_md(&agents_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("agents[0].skills"));
    }

    #[test]
    fn test_parse_tools_md_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tools_file = temp_dir.path().join("TOOLS.md");
        let mut file = std::fs::File::create(&tools_file).unwrap();
        write!(file, r#"
tools:
  - name: kubectl
    description: Kubernetes CLI tool
    category: infrastructure
  - name: curl
    description: HTTP client
    category: networking
    input_schema:
      type: object
      properties:
        url:
          type: string
"#).unwrap();

        let tools = parse_tools_md(&tools_file).unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "kubectl");
        assert_eq!(tools[0].category, "infrastructure");
        assert!(tools[0].input_schema.is_none());
        assert!(tools[1].input_schema.is_some());
    }

    #[test]
    fn test_version_hash_deterministic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let tools_file = temp_dir.path().join("TOOLS.md");

        std::fs::write(&agents_file, "agents: []").unwrap();
        std::fs::write(&tools_file, "tools: []").unwrap();

        let hash1 = version_hash(&agents_file, &tools_file).unwrap();
        let hash2 = version_hash(&agents_file, &tools_file).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_version_hash_changes_on_content_change() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let tools_file = temp_dir.path().join("TOOLS.md");

        std::fs::write(&agents_file, "agents: []").unwrap();
        std::fs::write(&tools_file, "tools: []").unwrap();
        let hash1 = version_hash(&agents_file, &tools_file).unwrap();

        std::fs::write(&agents_file, "agents:\n  - id: new-agent\n    name: New").unwrap();
        let hash2 = version_hash(&agents_file, &tools_file).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_version_hash_missing_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let agents_file = temp_dir.path().join("AGENTS.md");
        let tools_file = temp_dir.path().join("TOOLS.md");

        // Both missing - should still return a hash (empty hash)
        let hash = version_hash(&agents_file, &tools_file).unwrap();
        assert_eq!(hash.len(), 64);
    }
}
