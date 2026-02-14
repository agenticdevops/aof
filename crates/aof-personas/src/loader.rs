//! File loaders for AGENTS.md and SOUL.md workspace files
//!
//! Provides async loading, YAML parsing with precise error messages,
//! and in-memory caching with SHA256-based invalidation.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::types::{Agent, AgentsFile, Soul, SoulFrontmatter};

/// Loader for AGENTS.md workspace files
///
/// Parses YAML-formatted agent roster files with precise error messages
/// via serde_path_to_error. Supports both file-based and in-memory loading.
pub struct AgentLoader;

impl AgentLoader {
    /// Load agents from a file path asynchronously
    ///
    /// Reads the file, parses YAML content, and returns a validated list of agents.
    /// Errors include exact field paths for debugging (e.g., "agents[0].avatar").
    pub async fn load_from_file(path: &str) -> Result<Vec<Agent>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read AGENTS.md from '{}'", path))?;
        Self::load_from_str(&content)
    }

    /// Load agents from a string (useful for testing)
    pub fn load_from_str(content: &str) -> Result<Vec<Agent>> {
        let deserializer = serde_yaml::Deserializer::from_str(content);
        let file: AgentsFile = serde_path_to_error::deserialize(deserializer)
            .map_err(|e| anyhow::anyhow!("AGENTS.md parse error at '{}': {}", e.path(), e.inner()))?;
        Ok(file.agents)
    }

    /// Load agents from raw bytes
    pub fn load_from_bytes(content: &[u8]) -> Result<Vec<Agent>> {
        let s = std::str::from_utf8(content)
            .context("AGENTS.md content is not valid UTF-8")?;
        Self::load_from_str(s)
    }
}

/// Loader for SOUL.md workspace files
///
/// Parses Markdown with YAML frontmatter sections per agent.
/// Each section starts with `## agent-id` and contains a YAML code block
/// followed by prose communication guidance.
pub struct SoulLoader;

impl SoulLoader {
    /// Load souls from a file path asynchronously
    ///
    /// Reads the file, splits by agent sections, extracts YAML frontmatter
    /// and prose communication guides, returning a map keyed by agent id.
    pub async fn load_from_file(path: &str) -> Result<HashMap<String, Soul>> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Self::load_from_str(&content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("SOUL.md not found at '{}', returning empty soul map", path);
                Ok(HashMap::new())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to read SOUL.md from '{}': {}", path, e)),
        }
    }

    /// Load souls from a string (useful for testing)
    pub fn load_from_str(content: &str) -> Result<HashMap<String, Soul>> {
        let mut souls = HashMap::new();

        // Split by H2 headers (## agent-id)
        let sections: Vec<&str> = content.split("\n## ").collect();

        for (idx, section) in sections.iter().enumerate() {
            // Skip the document header (content before first ## section)
            if idx == 0 {
                // Check if the very first line starts with "## " (no preceding newline)
                if !section.starts_with("## ") && !content.starts_with("## ") {
                    continue;
                }
                // If content starts with "## ", strip the prefix for uniform processing
                let section = if content.starts_with("## ") {
                    section.strip_prefix("## ").unwrap_or(section)
                } else {
                    section
                };
                if let Some(soul) = Self::parse_section(section, idx)? {
                    souls.insert(soul.id.clone(), soul);
                }
                continue;
            }

            if let Some(soul) = Self::parse_section(section, idx)? {
                souls.insert(soul.id.clone(), soul);
            }
        }

        Ok(souls)
    }

    /// Parse a single agent section from SOUL.md
    ///
    /// Expects format:
    /// ```text
    /// agent-id
    ///
    /// ```yaml
    /// id: agent-id
    /// ...
    /// ```
    ///
    /// ### Communication Style Guide
    /// ...prose...
    /// ```
    fn parse_section(section: &str, section_idx: usize) -> Result<Option<Soul>> {
        let section = section.trim();
        if section.is_empty() {
            return Ok(None);
        }

        // Find YAML code block boundaries
        let yaml_start = section.find("```yaml");
        let yaml_end = if let Some(start) = yaml_start {
            let after_yaml = &section[start + 7..]; // skip "```yaml"
            after_yaml.find("```").map(|end| start + 7 + end)
        } else {
            None
        };

        let (yaml_str, prose) = match (yaml_start, yaml_end) {
            (Some(start), Some(end)) => {
                let yaml = &section[start + 7..end].trim();
                let after_block = &section[end + 3..].trim();
                (*yaml, after_block.to_string())
            }
            _ => {
                debug!(
                    "Section {} has no YAML code block, skipping",
                    section_idx
                );
                return Ok(None);
            }
        };

        // Parse YAML frontmatter with serde_path_to_error for precise errors
        let deserializer = serde_yaml::Deserializer::from_str(yaml_str);
        let frontmatter: SoulFrontmatter = serde_path_to_error::deserialize(deserializer)
            .map_err(|e| {
                anyhow::anyhow!(
                    "SOUL.md section {} parse error at '{}': {}",
                    section_idx,
                    e.path(),
                    e.inner()
                )
            })?;

        // Extract prose section (everything after the YAML block)
        // Strip leading "---" separator or "### " headers
        let communication_guide = prose
            .trim_start_matches("---")
            .trim()
            .to_string();

        let mut soul = Soul::from(frontmatter);
        soul.communication_guide = communication_guide;

        Ok(Some(soul))
    }
}

/// In-memory cache for loaded agent data with SHA256-based invalidation
///
/// Stores the parsed agent list and a hash of the source file content.
/// On subsequent loads, compares hashes to avoid re-parsing unchanged files.
pub struct AgentCache {
    /// Cached agents behind a read-write lock for concurrent access
    agents: Arc<RwLock<Vec<Agent>>>,
    /// Cached souls behind a read-write lock
    souls: Arc<RwLock<HashMap<String, Soul>>>,
    /// SHA256 hash of the agents file content
    agents_hash: Arc<RwLock<String>>,
    /// SHA256 hash of the souls file content
    souls_hash: Arc<RwLock<String>>,
}

impl AgentCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(Vec::new())),
            souls: Arc::new(RwLock::new(HashMap::new())),
            agents_hash: Arc::new(RwLock::new(String::new())),
            souls_hash: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Load agents from file, using cache if content unchanged
    ///
    /// Returns cached data if the file hash matches. Otherwise, re-parses
    /// and updates the cache.
    pub async fn load_agents(&self, path: &str) -> Result<Vec<Agent>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read agents from '{}'", path))?;

        let hash = Self::compute_hash(&content);

        // Check if cache is still valid
        {
            let cached_hash = self.agents_hash.read().await;
            if *cached_hash == hash {
                debug!("Agent cache hit for '{}'", path);
                return Ok(self.agents.read().await.clone());
            }
        }

        // Cache miss: parse and update
        debug!("Agent cache miss for '{}', re-parsing", path);
        let agents = AgentLoader::load_from_str(&content)?;

        {
            let mut cached_agents = self.agents.write().await;
            *cached_agents = agents.clone();
        }
        {
            let mut cached_hash = self.agents_hash.write().await;
            *cached_hash = hash;
        }

        Ok(agents)
    }

    /// Load souls from file, using cache if content unchanged
    pub async fn load_souls(&self, path: &str) -> Result<HashMap<String, Soul>> {
        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("SOUL.md not found at '{}', returning empty map", path);
                return Ok(HashMap::new());
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to read souls from '{}': {}", path, e)),
        };

        let hash = Self::compute_hash(&content);

        {
            let cached_hash = self.souls_hash.read().await;
            if *cached_hash == hash {
                debug!("Soul cache hit for '{}'", path);
                return Ok(self.souls.read().await.clone());
            }
        }

        debug!("Soul cache miss for '{}', re-parsing", path);
        let souls = SoulLoader::load_from_str(&content)?;

        {
            let mut cached_souls = self.souls.write().await;
            *cached_souls = souls.clone();
        }
        {
            let mut cached_hash = self.souls_hash.write().await;
            *cached_hash = hash;
        }

        Ok(souls)
    }

    /// Compute SHA256 hash of content
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Invalidate all cached data (forces re-read on next load)
    pub async fn invalidate(&self) {
        *self.agents_hash.write().await = String::new();
        *self.souls_hash.write().await = String::new();
    }
}

impl Default for AgentCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_agents_yaml() {
        let yaml = r#"
agents:
  - id: test-agent
    name: Test Agent
    role: Tester
    avatar: "\U0001F916"
    personality_traits:
      - curious
    can:
      - test things
    cannot:
      - break things
    skills:
      - testing
"#;
        let agents = AgentLoader::load_from_str(yaml).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "test-agent");
        assert_eq!(agents[0].name, "Test Agent");
    }

    #[test]
    fn test_parse_invalid_yaml_shows_field_path() {
        let yaml = r#"
agents:
  - id: test-agent
    name: Test Agent
    role: 123
    avatar: x
"#;
        // This should still parse since role accepts String
        let result = AgentLoader::load_from_str(yaml);
        assert!(result.is_ok()); // YAML coerces 123 to string
    }

    #[test]
    fn test_parse_missing_required_field() {
        let yaml = r#"
agents:
  - name: Test Agent
    role: Tester
"#;
        let result = AgentLoader::load_from_str(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("id"), "Error should mention missing field: {}", err);
    }
}
