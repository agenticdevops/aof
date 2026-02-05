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
}
