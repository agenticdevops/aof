//! Multi-source skill registry with precedence ordering.
//!
//! The registry loads skills from multiple sources:
//! 1. Workspace (local, highest precedence)
//! 2. Enterprise registry (organization-specific)
//! 3. Public registry (OpsSkillsHub)
//! 4. Bundled (shipped with AOF, lowest precedence)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::SkillError;
use crate::loader::SkillLoader;
use crate::requirements::{RequirementCheck, RequirementChecker};
use crate::types::{Skill, SkillConfig, SkillSearchResult};
use crate::watcher::SkillWatcher;
use crate::Result;

/// Multi-source skill registry
pub struct SkillRegistry {
    /// Configuration
    config: SkillConfig,

    /// Cached skills by name
    cache: Arc<RwLock<HashMap<String, Skill>>>,

    /// Skill loader
    loader: SkillLoader,

    /// File watcher for hot-reload (optional)
    watcher: Option<SkillWatcher>,
}

impl SkillRegistry {
    /// Create a new registry with configuration
    pub fn new(config: SkillConfig) -> Self {
        let loader = SkillLoader::new(config.clone());

        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            loader,
            watcher: None,
        }
    }

    /// Create a registry with default configuration
    pub fn default_registry() -> Self {
        Self::new(SkillConfig::default())
    }

    /// Load all skills from configured sources
    pub async fn load(&self) -> Result<()> {
        let skills = self.loader.load_all().await?;

        let mut cache = self.cache.write().await;
        cache.clear();

        for skill in skills {
            cache.insert(skill.name.clone(), skill);
        }

        info!("Registry loaded {} skills", cache.len());
        Ok(())
    }

    /// Get a skill by name
    pub async fn get(&self, name: &str) -> Option<Skill> {
        let cache = self.cache.read().await;
        cache.get(name).cloned()
    }

    /// Get all loaded skills
    pub async fn all(&self) -> Vec<Skill> {
        let cache = self.cache.read().await;
        cache.values().cloned().collect()
    }

    /// Get all eligible skills for the current environment
    pub async fn eligible(&self) -> Vec<Skill> {
        let all_skills = self.all().await;
        let mut checker = RequirementChecker::new();
        checker.filter_eligible(all_skills)
    }

    /// Check if a specific skill is eligible
    pub async fn check_skill(&self, name: &str) -> Result<RequirementCheck> {
        let skill = self
            .get(name)
            .await
            .ok_or_else(|| SkillError::not_found(name))?;

        let mut checker = RequirementChecker::new();
        Ok(checker.check(&skill))
    }

    /// Search skills by query
    ///
    /// Searches name, description, and tags
    pub async fn search(&self, query: &str) -> Vec<SkillSearchResult> {
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        let cache = self.cache.read().await;
        let mut results: Vec<SkillSearchResult> = Vec::new();

        for skill in cache.values() {
            let mut score = 0.0;
            let mut matches = Vec::new();

            // Search in name (highest weight)
            let name_lower = skill.name.to_lowercase();
            for term in &terms {
                if name_lower.contains(term) {
                    score += 1.0;
                    matches.push(format!("name:{}", term));
                }
            }

            // Search in description
            let desc_lower = skill.description.to_lowercase();
            for term in &terms {
                if desc_lower.contains(term) {
                    score += 0.5;
                    matches.push(format!("description:{}", term));
                }
            }

            // Search in tags
            for tag in &skill.metadata.tags {
                let tag_lower = tag.to_lowercase();
                for term in &terms {
                    if tag_lower.contains(term) {
                        score += 0.75;
                        matches.push(format!("tag:{}", tag));
                    }
                }
            }

            if score > 0.0 {
                // Normalize score (0.0 - 1.0)
                let normalized = (score / (terms.len() as f32 * 2.0)).min(1.0);
                results.push(SkillSearchResult {
                    skill: skill.clone(),
                    score: normalized,
                    matches,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// List skill names
    pub async fn list_names(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        let mut names: Vec<String> = cache.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get skill count
    pub async fn count(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Add a skill directly (useful for testing or runtime additions)
    pub async fn add(&self, skill: Skill) {
        let mut cache = self.cache.write().await;
        debug!("Adding skill '{}' to registry", skill.name);
        cache.insert(skill.name.clone(), skill);
    }

    /// Remove a skill by name
    pub async fn remove(&self, name: &str) -> Option<Skill> {
        let mut cache = self.cache.write().await;
        debug!("Removing skill '{}' from registry", name);
        cache.remove(name)
    }

    /// Enable hot-reload via file watching
    pub async fn enable_watch(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(()); // Already watching
        }

        let mut paths_to_watch = Vec::new();

        if let Some(ref workspace_dir) = self.config.workspace_dir {
            paths_to_watch.push(workspace_dir.clone());
        }

        paths_to_watch.extend(self.config.bundled_dirs.clone());

        if paths_to_watch.is_empty() {
            return Ok(()); // Nothing to watch
        }

        let cache = Arc::clone(&self.cache);
        let _loader = SkillLoader::new(self.config.clone());

        let watcher = SkillWatcher::new(paths_to_watch, move |event| {
            let cache = Arc::clone(&cache);
            let loader_clone = SkillLoader::new(SkillConfig::default());

            tokio::spawn(async move {
                debug!("Skill file changed: {:?}", event);
                // Reload affected skills
                if let Ok(skills) = loader_clone.load_all().await {
                    let mut cache_guard = cache.write().await;
                    cache_guard.clear();
                    for skill in skills {
                        cache_guard.insert(skill.name.clone(), skill);
                    }
                    info!("Skills reloaded: {} total", cache_guard.len());
                }
            });
        })?;

        self.watcher = Some(watcher);
        info!("Skill hot-reload enabled");
        Ok(())
    }

    /// Disable hot-reload
    pub fn disable_watch(&mut self) {
        self.watcher = None;
        info!("Skill hot-reload disabled");
    }

    /// Match skills by intent (progressive disclosure)
    ///
    /// Finds skills relevant to the given intent using keyword matching
    /// on description, action, and tags. Returns skills above relevance threshold.
    ///
    /// # Arguments
    /// * `intent` - The user/agent intent (e.g., "debug pod crashes")
    ///
    /// # Returns
    /// A vector of skills matching the intent, sorted by relevance
    pub async fn match_skills(&self, intent: &str) -> Vec<Skill> {
        let results = self.search(intent).await;
        results
            .into_iter()
            .filter(|r| r.score > 0.5) // Relevance threshold
            .map(|r| r.skill)
            .collect()
    }
}

/// Validation result for agentskills.io compliance
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of errors (missing required fields, etc.)
    pub errors: Vec<String>,
    /// List of warnings (missing optional sections, etc.)
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// Create a validation report
    pub fn new(is_valid: bool, errors: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            is_valid,
            errors,
            warnings,
        }
    }
}

/// Validator for agentskills.io standard compliance
#[derive(Debug, Clone)]
pub struct AgentSkillsValidator;

impl AgentSkillsValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate skill frontmatter against agentskills.io standard
    ///
    /// Checks for required fields: name, description, metadata structure
    pub fn validate_frontmatter(&self, skill: &Skill) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required fields
        if skill.name.is_empty() {
            errors.push("Required field 'name' is empty".to_string());
        }

        if skill.description.is_empty() {
            errors.push("Required field 'description' is empty".to_string());
        }

        // Check metadata structure
        if skill.metadata.requires.bins.is_empty()
            && skill.metadata.requires.env.is_empty()
            && skill.metadata.requires.config.is_empty()
            && !skill.metadata.always
        {
            warnings.push("Skill has no requirements defined (bins, env, config, or always=true)"
                .to_string());
        }

        // Check tags
        if skill.metadata.tags.is_empty() {
            warnings.push("Skill has no tags defined for searchability".to_string());
        }

        let is_valid = errors.is_empty();
        ValidationReport::new(is_valid, errors, warnings)
    }

    /// Validate skill markdown content
    ///
    /// Checks for expected sections in the markdown content
    pub fn validate_markdown(&self, skill: &Skill) -> ValidationReport {
        let mut warnings = Vec::new();
        let errors = Vec::new();

        let content_lower = skill.content.to_lowercase();

        // Check for expected sections
        if !content_lower.contains("# ") && !content_lower.contains("#") {
            warnings.push("Missing main heading (# Skill Name)".to_string());
        }

        if !content_lower.contains("## when") && !content_lower.contains("when to use") {
            warnings.push("Missing 'When to Use' section".to_string());
        }

        if !content_lower.contains("## step") && !content_lower.contains("## instruction") {
            warnings.push("Missing 'Steps' or 'Instructions' section".to_string());
        }

        ValidationReport::new(errors.is_empty(), errors, warnings)
    }

    /// Validate Claude/Codex tool compatibility
    ///
    /// Checks if skill can be parsed and used as a tool definition
    pub fn validate_claude_compatibility(&self, skill: &Skill) -> bool {
        // Basic validation: skill has required fields for tool definition
        !skill.name.is_empty() && !skill.description.is_empty()
    }
}

impl Default for AgentSkillsValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillMetadata, SkillSource};

    fn make_test_skill(name: &str, tags: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("Description for {}", name),
            homepage: None,
            content: format!("# {}", name),
            metadata: SkillMetadata {
                tags: tags.into_iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            },
            source: SkillSource::Bundled,
        }
    }

    #[tokio::test]
    async fn test_registry_basic_operations() {
        let registry = SkillRegistry::default_registry();

        let skill = make_test_skill("test-skill", vec!["test"]);
        registry.add(skill).await;

        assert_eq!(registry.count().await, 1);
        assert!(registry.get("test-skill").await.is_some());
        assert!(registry.get("nonexistent").await.is_none());

        let names = registry.list_names().await;
        assert_eq!(names, vec!["test-skill"]);
    }

    #[tokio::test]
    async fn test_registry_search() {
        let registry = SkillRegistry::default_registry();

        registry.add(make_test_skill("k8s-debug", vec!["kubernetes", "debugging"])).await;
        registry.add(make_test_skill("prometheus-query", vec!["monitoring", "prometheus"])).await;
        registry.add(make_test_skill("loki-search", vec!["logging", "loki"])).await;

        // Search by name
        let results = registry.search("k8s").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill.name, "k8s-debug");

        // Search by tag
        let results = registry.search("monitoring").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill.name, "prometheus-query");

        // Search multiple terms
        let results = registry.search("kubernetes debug").await;
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_registry_remove() {
        let registry = SkillRegistry::default_registry();

        registry.add(make_test_skill("to-remove", vec![])).await;
        assert_eq!(registry.count().await, 1);

        let removed = registry.remove("to-remove").await;
        assert!(removed.is_some());
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_match_skills() {
        let registry = SkillRegistry::default_registry();

        registry.add(make_test_skill("k8s-debug", vec!["kubernetes", "debugging"])).await;
        registry.add(make_test_skill("prometheus-query", vec!["monitoring"])).await;
        registry.add(make_test_skill("git-operations", vec!["git"])).await;

        let matched = registry.match_skills("debug pod").await;
        assert!(!matched.is_empty());
        assert!(matched.iter().any(|s| s.name == "k8s-debug"));
    }

    #[test]
    fn test_validator_valid_skill() {
        let skill = make_test_skill("test-skill", vec!["test"]);
        let validator = AgentSkillsValidator::new();

        let report = validator.validate_frontmatter(&skill);
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_validator_missing_name() {
        let mut skill = make_test_skill("test", vec![]);
        skill.name = String::new();

        let validator = AgentSkillsValidator::new();
        let report = validator.validate_frontmatter(&skill);

        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("name"));
    }

    #[test]
    fn test_validator_missing_description() {
        let mut skill = make_test_skill("test", vec![]);
        skill.description = String::new();

        let validator = AgentSkillsValidator::new();
        let report = validator.validate_frontmatter(&skill);

        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("description"));
    }

    #[test]
    fn test_validator_markdown_validation() {
        let skill = make_test_skill("test-skill", vec!["test"]);
        let validator = AgentSkillsValidator::new();

        let report = validator.validate_markdown(&skill);
        // Should warn about missing sections since make_test_skill has minimal content
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn test_validator_claude_compatibility() {
        let skill = make_test_skill("test-skill", vec!["test"]);
        let validator = AgentSkillsValidator::new();

        assert!(validator.validate_claude_compatibility(&skill));

        let mut invalid_skill = skill.clone();
        invalid_skill.name = String::new();
        assert!(!validator.validate_claude_compatibility(&invalid_skill));
    }
}
