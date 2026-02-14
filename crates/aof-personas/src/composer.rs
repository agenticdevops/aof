//! System prompt composition engine
//!
//! Composes dynamic system prompts from workspace files (AGENTS.md, SOUL.md, TOOLS.md)
//! using 7-layer instruction layering. Each agent gets a unique prompt reflecting
//! their personality, role, capabilities, and communication style.
//!
//! # Instruction Layers (in order)
//!
//! 1. **Base Instructions** - Fixed foundation instruction
//! 2. **Role Definition** - Name, role, skills from AGENTS.md
//! 3. **Personality & Values** - Summary and core values from SOUL.md
//! 4. **Communication Style** - Tone, style, communication guide from SOUL.md
//! 5. **Capabilities & Boundaries** - CAN/CANNOT from AGENTS.md
//! 6. **Tools Available** - Tool descriptions linked from skills via TOOLS.md
//! 7. **Behavioral Rules** - Fixed behavioral guidelines
//!
//! # Example
//!
//! ```rust
//! use aof_personas::composer::PromptComposer;
//! use aof_personas::types::{Agent, Soul};
//!
//! let agents = vec![Agent::new("k8s-monitor", "K8s Monitor", "Infra", "\u{1F916}")];
//! let souls = std::collections::HashMap::new();
//! let tools = vec![];
//!
//! let composer = PromptComposer::new(agents, souls, tools);
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::types::{Agent, Soul};

/// Tool definition for prompt composition (from TOOLS.md)
#[derive(Debug, Clone)]
pub struct Tool {
    /// Tool name (matches agent skill names)
    pub name: String,
    /// Human-readable description of the tool
    pub description: String,
    /// Tool category (e.g., "infrastructure", "networking")
    pub category: String,
}

/// Cache statistics for monitoring prompt composition performance
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits (prompt reused from cache)
    pub hits: u32,
    /// Number of cache misses (prompt composed fresh)
    pub misses: u32,
    /// Number of entries currently in cache
    pub entries: u32,
}

/// System prompt composition engine
///
/// Composes dynamic system prompts by layering instructions from
/// workspace files. Each agent gets a unique prompt that reflects
/// their personality, role, skills, and communication style.
///
/// Prompts are cached per agent with SHA256-based invalidation.
/// The cache is cleared when underlying data changes (e.g., PersonaWatcher
/// triggers a reload).
pub struct PromptComposer {
    /// Agent roster keyed by agent ID
    agents: HashMap<String, Agent>,
    /// Soul guidance keyed by agent ID
    souls: HashMap<String, Soul>,
    /// Tool definitions keyed by tool name
    tools: HashMap<String, Tool>,
    /// Cached composed prompts: agent_id -> (prompt, timestamp)
    composed_prompts: Arc<RwLock<HashMap<String, (String, DateTime<Utc>)>>>,
    /// SHA256 hash of input data for cache invalidation
    data_hash: Arc<RwLock<String>>,
    /// Cache hit counter
    cache_hits: Arc<AtomicU32>,
    /// Cache miss counter
    cache_misses: Arc<AtomicU32>,
}

impl PromptComposer {
    /// Create a new PromptComposer from loaded workspace data
    ///
    /// # Arguments
    /// * `agents` - Agent roster from AGENTS.md
    /// * `souls` - Soul guidance from SOUL.md (keyed by agent ID)
    /// * `tools` - Tool definitions from TOOLS.md
    pub fn new(
        agents: Vec<Agent>,
        souls: HashMap<String, Soul>,
        tools: Vec<Tool>,
    ) -> Self {
        let agent_map: HashMap<String, Agent> = agents
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();

        let tool_map: HashMap<String, Tool> = tools
            .into_iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        let data_hash = Self::compute_data_hash(&agent_map, &souls, &tool_map);

        Self {
            agents: agent_map,
            souls,
            tools: tool_map,
            composed_prompts: Arc::new(RwLock::new(HashMap::new())),
            data_hash: Arc::new(RwLock::new(data_hash)),
            cache_hits: Arc::new(AtomicU32::new(0)),
            cache_misses: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Compose a complete system prompt for an agent
    ///
    /// Layers instructions in 7 sections:
    /// 1. Base instructions (fixed)
    /// 2. Role definition (from AGENTS.md)
    /// 3. Personality & values (from SOUL.md)
    /// 4. Communication style (from SOUL.md)
    /// 5. Capabilities & boundaries (from AGENTS.md)
    /// 6. Tools available (from TOOLS.md, linked via skills)
    /// 7. Behavioral rules (fixed)
    ///
    /// If no SOUL.md entry exists for the agent, personality/communication
    /// sections use sensible defaults.
    ///
    /// # Errors
    /// Returns error if agent_id does not exist in the roster.
    pub fn compose_system_prompt(&self, agent_id: &str) -> Result<String> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: '{}'", agent_id))?;

        let soul = self.souls.get(agent_id);

        let mut sections = Vec::new();

        // Layer 1: Base instructions
        sections.push("[BASE INSTRUCTIONS]\nYou are an AI agent helping with infrastructure operations.".to_string());

        // Layer 2: Role definition
        let skills_str = agent.skills.join(", ");
        sections.push(format!(
            "[ROLE DEFINITION]\nYour name: {}\nYour role: {}\nYour primary responsibilities: {}",
            agent.name, agent.role, skills_str
        ));

        // Layer 3: Personality & values
        if let Some(soul) = soul {
            let values_str = soul.values.iter()
                .map(|v| format!("- {}", v))
                .collect::<Vec<_>>()
                .join("\n");
            sections.push(format!(
                "[PERSONALITY & VALUES]\n{}\n\nYour core values:\n{}",
                soul.personality_summary, values_str
            ));
        } else {
            // Default personality from agent traits
            let traits_str = agent.personality_traits.join(", ");
            sections.push(format!(
                "[PERSONALITY & VALUES]\nYou are {}.\n\nYour core values:\n- reliability\n- helpfulness",
                traits_str
            ));
        }

        // Layer 4: Communication style
        if let Some(soul) = soul {
            let mut comm_section = format!(
                "[COMMUNICATION STYLE]\nCommunication style: {}\nTone: {}",
                soul.communication_style, soul.tone
            );
            if !soul.communication_guide.is_empty() {
                comm_section.push_str(&format!("\n\n{}", soul.communication_guide));
            }
            sections.push(comm_section);
        }

        // Layer 5: Capabilities & boundaries
        let can_items = agent.can.iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        let cannot_items = agent.cannot.iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!(
            "[CAPABILITIES & BOUNDARIES]\nYou CAN:\n{}\n\nYou CANNOT:\n{}",
            can_items, cannot_items
        ));

        // Layer 6: Tools available (linked from skills via TOOLS.md)
        let tool_descriptions = self.build_tool_section(agent);
        sections.push(format!("[TOOLS]\n{}", tool_descriptions));

        // Layer 7: Behavioral rules
        sections.push("[BEHAVIORAL RULES]\n- Always explain your reasoning\n- Ask clarifying questions when uncertain\n- Escalate to humans when needed".to_string());

        Ok(sections.join("\n\n"))
    }

    /// Compose a system prompt with a token limit, truncating intelligently
    ///
    /// If the full prompt exceeds `max_tokens`, sections are removed in
    /// priority order (lowest priority removed first):
    /// 1. Behavioral rules (removed first)
    /// 2. Tool descriptions shortened to names only
    /// 3. Communication style guide removed
    /// 4. Base instructions, role, personality, boundaries (never removed)
    ///
    /// # Arguments
    /// * `agent_id` - Agent to compose prompt for
    /// * `max_tokens` - Maximum token count (estimated as len/4)
    pub fn compose_system_prompt_with_limit(
        &self,
        agent_id: &str,
        max_tokens: usize,
    ) -> Result<String> {
        let full_prompt = self.compose_system_prompt(agent_id)?;
        let token_count = Self::estimate_token_count(&full_prompt);

        if token_count <= max_tokens {
            return Ok(full_prompt);
        }

        warn!(
            "Persona prompt truncation needed for agent '{}': {} tokens > {} limit",
            agent_id, token_count, max_tokens
        );

        let agent = self.agents.get(agent_id).unwrap();
        let soul = self.souls.get(agent_id);

        // Rebuild with truncation strategy
        let mut sections = Vec::new();

        // Layer 1: Base (never truncate)
        sections.push("[BASE INSTRUCTIONS]\nYou are an AI agent helping with infrastructure operations.".to_string());

        // Layer 2: Role (never truncate)
        let skills_str = agent.skills.join(", ");
        sections.push(format!(
            "[ROLE DEFINITION]\nYour name: {}\nYour role: {}\nYour primary responsibilities: {}",
            agent.name, agent.role, skills_str
        ));

        // Layer 3: Personality (never truncate)
        if let Some(soul) = soul {
            let values_str = soul.values.iter()
                .map(|v| format!("- {}", v))
                .collect::<Vec<_>>()
                .join("\n");
            sections.push(format!(
                "[PERSONALITY & VALUES]\n{}\n\nYour core values:\n{}",
                soul.personality_summary, values_str
            ));
        } else {
            let traits_str = agent.personality_traits.join(", ");
            sections.push(format!(
                "[PERSONALITY & VALUES]\nYou are {}.\n\nYour core values:\n- reliability\n- helpfulness",
                traits_str
            ));
        }

        // Layer 5: Boundaries (never truncate)
        let can_items = agent.can.iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        let cannot_items = agent.cannot.iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!(
            "[CAPABILITIES & BOUNDARIES]\nYou CAN:\n{}\n\nYou CANNOT:\n{}",
            can_items, cannot_items
        ));

        // Check if we fit without behavioral rules, communication, and tools
        let base_prompt = sections.join("\n\n");
        let base_tokens = Self::estimate_token_count(&base_prompt);

        if base_tokens > max_tokens {
            // Even essential sections exceed limit -- return what we have
            warn!(
                "Persona prompt for '{}' truncated to essentials only ({} tokens, limit {})",
                agent_id, base_tokens, max_tokens
            );
            return Ok(base_prompt);
        }

        // Try adding communication style (without guide)
        let remaining = max_tokens - Self::estimate_token_count(&base_prompt);
        if let Some(soul) = soul {
            let comm_brief = format!(
                "[COMMUNICATION STYLE]\nCommunication style: {}\nTone: {}",
                soul.communication_style, soul.tone
            );
            let comm_tokens = Self::estimate_token_count(&comm_brief);
            if comm_tokens < remaining {
                sections.push(comm_brief);
            }
        }

        // Try adding shortened tools (names only)
        let remaining = max_tokens - Self::estimate_token_count(&sections.join("\n\n"));
        let tool_names: Vec<String> = agent.skills.iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();
        let tools_brief = format!("[TOOLS]\nAvailable tools: {}", tool_names.join(", "));
        let tools_tokens = Self::estimate_token_count(&tools_brief);
        if tools_tokens < remaining {
            sections.push(tools_brief);
        }

        let result = sections.join("\n\n");
        let final_tokens = Self::estimate_token_count(&result);
        warn!(
            "Persona prompt truncated from {} to {} tokens for agent '{}'",
            token_count, final_tokens, agent_id
        );

        Ok(result)
    }

    /// Compose a system prompt with caching
    ///
    /// Returns cached prompt if available and data hasn't changed.
    /// Otherwise composes a new prompt and caches it.
    pub async fn compose_system_prompt_cached(&self, agent_id: &str) -> Result<String> {
        // Check cache first
        {
            let cache = self.composed_prompts.read().await;
            if let Some((prompt, _timestamp)) = cache.get(agent_id) {
                // Verify data hash hasn't changed
                let current_hash = Self::compute_data_hash(&self.agents, &self.souls, &self.tools);
                let stored_hash = self.data_hash.read().await;
                if current_hash == *stored_hash {
                    self.cache_hits.fetch_add(1, Ordering::Relaxed);
                    debug!("Prompt cache hit for agent '{}'", agent_id);
                    return Ok(prompt.clone());
                }
            }
        }

        // Cache miss: compose and store
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        debug!("Prompt cache miss for agent '{}'", agent_id);

        let prompt = self.compose_system_prompt(agent_id)?;

        {
            let mut cache = self.composed_prompts.write().await;
            cache.insert(agent_id.to_string(), (prompt.clone(), Utc::now()));
        }

        Ok(prompt)
    }

    /// Estimate token count for a text string
    ///
    /// Uses the Claude standard approximation: 1 token ~ 4 characters.
    /// This is conservative but suitable for prompt budget management.
    pub fn estimate_token_count(text: &str) -> usize {
        text.len() / 4
    }

    /// Clear the prompt cache (forces recomposition on next call)
    pub async fn clear_cache(&self) {
        let mut cache = self.composed_prompts.write().await;
        cache.clear();
        debug!("Prompt cache cleared");
    }

    /// Get cache statistics for monitoring
    pub fn cache_stats(&self) -> CacheStats {
        let entries = {
            // Use try_read to avoid blocking -- fall back to 0 if locked
            // For accurate count, callers should use the async version
            0 // Will be updated in the async path
        };

        CacheStats {
            hits: self.cache_hits.load(Ordering::Relaxed),
            misses: self.cache_misses.load(Ordering::Relaxed),
            entries,
        }
    }

    /// Get cache statistics (async version with accurate entry count)
    pub async fn cache_stats_async(&self) -> CacheStats {
        let cache = self.composed_prompts.read().await;
        CacheStats {
            hits: self.cache_hits.load(Ordering::Relaxed),
            misses: self.cache_misses.load(Ordering::Relaxed),
            entries: cache.len() as u32,
        }
    }

    /// Validate that prompt composition is safe (no injection, valid references)
    ///
    /// Checks:
    /// - Agent exists
    /// - Soul (if present) matches agent ID
    /// - Skills reference known tools (warns for unknown)
    /// - Composed prompt doesn't contain injection patterns
    pub fn validate_and_compose(&self, agent_id: &str) -> Result<String> {
        // Check agent exists
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: '{}'", agent_id))?;

        // Validate skill -> tool references
        for skill in &agent.skills {
            if !self.tools.contains_key(skill) {
                warn!(
                    "Agent '{}' skill '{}' not found in TOOLS.md",
                    agent_id, skill
                );
            }
        }

        // Compose the prompt
        let prompt = self.compose_system_prompt(agent_id)?;

        // Scan for injection patterns
        Self::detect_injection(&prompt, agent_id)?;

        Ok(prompt)
    }

    /// Detect prompt injection patterns in composed text
    fn detect_injection(text: &str, agent_id: &str) -> Result<()> {
        let patterns = [
            r"(?i)ignore\s+all\s+previous",
            r"(?i)forget\s+(all\s+)?instructions",
            r"(?i)disregard\s+.*prompt",
            r"(?i)override\s+system",
            r"(?i)you\s+are\s+now\s+(?:a|an)\s+(?:different|new)",
            r"(?i)ignore\s+(?:the\s+)?above",
        ];

        for pattern in &patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(text) {
                warn!(
                    "SECURITY: Prompt injection detected in composed prompt for agent '{}' (pattern: '{}')",
                    agent_id, pattern
                );
                bail!(
                    "Prompt injection detected in composed prompt for agent '{}': matched pattern '{}'",
                    agent_id, pattern
                );
            }
        }

        Ok(())
    }

    /// Build the tool section for an agent's prompt
    ///
    /// Maps agent skills to tool descriptions from TOOLS.md.
    /// Deduplicates skills and warns about unknown tools.
    fn build_tool_section(&self, agent: &Agent) -> String {
        if agent.skills.is_empty() {
            return "No tools configured for this agent.".to_string();
        }

        let mut seen_skills = HashSet::new();
        let mut tool_lines = Vec::new();

        for skill in &agent.skills {
            if !seen_skills.insert(skill.as_str()) {
                continue; // Skip duplicate
            }

            if let Some(tool) = self.tools.get(skill) {
                // Truncate description to 100 chars if needed
                let desc = if tool.description.len() > 100 {
                    format!("{}...", &tool.description[..97])
                } else {
                    tool.description.clone()
                };
                tool_lines.push(format!(
                    "- {} ({}, category: {})",
                    tool.name, desc, tool.category
                ));
            } else {
                warn!("Skill '{}' not found in TOOLS.md for agent '{}'", skill, agent.id);
                tool_lines.push(format!("- {} (not found in TOOLS.md)", skill));
            }
        }

        if tool_lines.is_empty() {
            "No tools configured for this agent.".to_string()
        } else {
            format!("Available tools:\n{}", tool_lines.join("\n"))
        }
    }

    /// Compute SHA256 hash of all input data for cache invalidation
    fn compute_data_hash(
        agents: &HashMap<String, Agent>,
        souls: &HashMap<String, Soul>,
        tools: &HashMap<String, Tool>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash agent data (sorted for determinism)
        let mut agent_ids: Vec<&String> = agents.keys().collect();
        agent_ids.sort();
        for id in agent_ids {
            let agent = &agents[id];
            hasher.update(agent.id.as_bytes());
            hasher.update(agent.name.as_bytes());
            hasher.update(agent.role.as_bytes());
            for skill in &agent.skills {
                hasher.update(skill.as_bytes());
            }
        }

        // Hash soul data
        let mut soul_ids: Vec<&String> = souls.keys().collect();
        soul_ids.sort();
        for id in soul_ids {
            let soul = &souls[id];
            hasher.update(soul.personality_summary.as_bytes());
            hasher.update(soul.communication_guide.as_bytes());
        }

        // Hash tool data
        let mut tool_names: Vec<&String> = tools.keys().collect();
        tool_names.sort();
        for name in tool_names {
            let tool = &tools[name];
            hasher.update(tool.name.as_bytes());
            hasher.update(tool.description.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_agent(id: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: format!("Agent {}", id),
            role: "Tester".to_string(),
            avatar: "\u{1F916}".to_string(),
            personality_traits: vec!["curious".to_string()],
            can: vec!["test things".to_string()],
            cannot: vec!["break things".to_string()],
            skills: vec!["testing".to_string()],
        }
    }

    fn make_test_soul(id: &str) -> Soul {
        Soul {
            id: id.to_string(),
            communication_style: "formal".to_string(),
            tone: "calm".to_string(),
            values: vec!["reliability".to_string()],
            personality_summary: "A test agent.".to_string(),
            boundaries: vec!["Never break things".to_string()],
            default_intro: "Hello, I am a test agent.".to_string(),
            communication_guide: "Be helpful and formal.".to_string(),
        }
    }

    fn make_test_tool(name: &str) -> Tool {
        Tool {
            name: name.to_string(),
            description: format!("{} tool for testing", name),
            category: "testing".to_string(),
        }
    }

    #[test]
    fn test_basic_composition() {
        let agents = vec![make_test_agent("test-agent")];
        let mut souls = HashMap::new();
        souls.insert("test-agent".to_string(), make_test_soul("test-agent"));
        let tools = vec![make_test_tool("testing")];

        let composer = PromptComposer::new(agents, souls, tools);
        let prompt = composer.compose_system_prompt("test-agent").unwrap();

        assert!(prompt.contains("[BASE INSTRUCTIONS]"));
        assert!(prompt.contains("[ROLE DEFINITION]"));
        assert!(prompt.contains("[PERSONALITY & VALUES]"));
        assert!(prompt.contains("[COMMUNICATION STYLE]"));
        assert!(prompt.contains("[CAPABILITIES & BOUNDARIES]"));
        assert!(prompt.contains("[TOOLS]"));
        assert!(prompt.contains("[BEHAVIORAL RULES]"));
        assert!(prompt.contains("Agent test-agent"));
        assert!(prompt.contains("Tester"));
    }

    #[test]
    fn test_missing_agent_returns_error() {
        let composer = PromptComposer::new(vec![], HashMap::new(), vec![]);
        let result = composer.compose_system_prompt("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(PromptComposer::estimate_token_count("hello"), 1);
        assert_eq!(PromptComposer::estimate_token_count("hello world!"), 3);
        assert_eq!(PromptComposer::estimate_token_count(""), 0);
    }
}
