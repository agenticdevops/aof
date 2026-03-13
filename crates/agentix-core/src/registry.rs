// OpenAgentiX Core - Resource Registries
//
// Unified registries for loading, indexing, and resolving OpenAgentiX resource types.
// Each registry provides:
// - Directory loading (load all resources from a path)
// - Name-based lookup
// - Type-safe access to resources

use crate::agent::AgentConfig;
use crate::context::Context;
use crate::error::{AgentixError, AgentixResult};
use crate::trigger::Trigger;

use std::collections::HashMap;
use std::path::Path;

/// Common trait for all resource registries
pub trait Registry<T> {
    /// Load all resources from a directory
    fn load_directory(&mut self, path: &Path) -> AgentixResult<usize>;

    /// Get a resource by name
    fn get(&self, name: &str) -> Option<&T>;

    /// Get all resources
    fn get_all(&self) -> Vec<&T>;

    /// Register a resource
    fn register(&mut self, resource: T) -> AgentixResult<()>;

    /// Get the count of resources
    fn count(&self) -> usize;

    /// Check if a resource exists
    fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

// ============================================================================
// Agent Registry
// ============================================================================

/// Registry for Agent resources
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentConfig>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get agent names
    pub fn names(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }
}

impl Registry<AgentConfig> for AgentRegistry {
    fn load_directory(&mut self, path: &Path) -> AgentixResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                // Skip non-Agent YAML files (Trigger, etc.)
                if !yaml_file_has_kind(&file_path, "Agent") {
                    tracing::debug!("Skipping non-Agent file: {:?}", file_path);
                    continue;
                }

                match load_yaml_file::<AgentConfig>(&file_path) {
                    Ok(agent) => {
                        let name = agent.name.clone();
                        self.agents.insert(name.clone(), agent);
                        tracing::debug!("Loaded agent: {}", name);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load agent from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn get(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    fn get_all(&self) -> Vec<&AgentConfig> {
        self.agents.values().collect()
    }

    fn register(&mut self, resource: AgentConfig) -> AgentixResult<()> {
        let name = resource.name.clone();
        self.agents.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.agents.len()
    }
}

// ============================================================================
// Context Registry
// ============================================================================

/// Registry for Context resources
#[derive(Debug, Default)]
pub struct ContextRegistry {
    contexts: HashMap<String, Context>,
}

impl ContextRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get context names
    pub fn names(&self) -> Vec<&str> {
        self.contexts.keys().map(|s| s.as_str()).collect()
    }

    /// Get mutable reference to a context (for env var expansion)
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Context> {
        self.contexts.get_mut(name)
    }

    /// Expand environment variables in all contexts
    pub fn expand_all_env_vars(&mut self) {
        for context in self.contexts.values_mut() {
            context.expand_env_vars();
        }
    }
}

impl Registry<Context> for ContextRegistry {
    fn load_directory(&mut self, path: &Path) -> AgentixResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                match load_yaml_file::<Context>(&file_path) {
                    Ok(mut context) => {
                        context.expand_env_vars();
                        if let Err(e) = context.validate() {
                            tracing::warn!("Invalid context in {:?}: {}", file_path, e);
                            continue;
                        }
                        let name = context.metadata.name.clone();
                        self.contexts.insert(name.clone(), context);
                        tracing::debug!("Loaded context: {}", name);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load context from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn get(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    fn get_all(&self) -> Vec<&Context> {
        self.contexts.values().collect()
    }

    fn register(&mut self, resource: Context) -> AgentixResult<()> {
        resource.validate().map_err(AgentixError::Config)?;
        let name = resource.metadata.name.clone();
        self.contexts.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.contexts.len()
    }
}

// ============================================================================
// Trigger Registry
// ============================================================================

/// Registry for Trigger resources
#[derive(Debug, Default)]
pub struct TriggerRegistry {
    triggers: HashMap<String, Trigger>,
}

impl TriggerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get trigger names
    pub fn names(&self) -> Vec<&str> {
        self.triggers.keys().map(|s| s.as_str()).collect()
    }

    /// Get triggers by type
    pub fn get_by_type(&self, trigger_type: crate::trigger::StandaloneTriggerType) -> Vec<&Trigger> {
        self.triggers
            .values()
            .filter(|t| t.spec.trigger_type == trigger_type)
            .collect()
    }

    /// Expand environment variables in all triggers
    pub fn expand_all_env_vars(&mut self) {
        for trigger in self.triggers.values_mut() {
            trigger.expand_env_vars();
        }
    }
}

impl Registry<Trigger> for TriggerRegistry {
    fn load_directory(&mut self, path: &Path) -> AgentixResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                match load_yaml_file::<Trigger>(&file_path) {
                    Ok(mut trigger) => {
                        trigger.expand_env_vars();
                        if let Err(e) = trigger.validate() {
                            tracing::warn!("Invalid trigger in {:?}: {}", file_path, e);
                            continue;
                        }
                        let name = trigger.metadata.name.clone();
                        self.triggers.insert(name.clone(), trigger);
                        tracing::debug!("Loaded trigger: {}", name);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load trigger from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn get(&self, name: &str) -> Option<&Trigger> {
        self.triggers.get(name)
    }

    fn get_all(&self) -> Vec<&Trigger> {
        self.triggers.values().collect()
    }

    fn register(&mut self, resource: Trigger) -> AgentixResult<()> {
        resource.validate().map_err(AgentixError::Config)?;
        let name = resource.metadata.name.clone();
        self.triggers.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.triggers.len()
    }
}

// ============================================================================
// Resource Manager (Unified Access)
// ============================================================================

/// Unified resource manager holding all registries
#[derive(Debug, Default)]
pub struct ResourceManager {
    pub agents: AgentRegistry,
    pub contexts: ContextRegistry,
    pub triggers: TriggerRegistry,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all resources from a directory structure
    ///
    /// Expected structure:
    /// ```text
    /// root/
    /// ├── agents/
    /// ├── contexts/
    /// └── triggers/
    /// ```
    pub fn load_directory(&mut self, root: &Path) -> AgentixResult<ResourceLoadSummary> {
        let mut summary = ResourceLoadSummary::default();

        // Load agents
        let agents_dir = root.join("agents");
        if agents_dir.exists() {
            summary.agents = self.agents.load_directory(&agents_dir)?;
        }

        // Load contexts
        let contexts_dir = root.join("contexts");
        if contexts_dir.exists() {
            summary.contexts = self.contexts.load_directory(&contexts_dir)?;
        }

        // Load triggers
        let triggers_dir = root.join("triggers");
        if triggers_dir.exists() {
            summary.triggers = self.triggers.load_directory(&triggers_dir)?;
        }

        Ok(summary)
    }

    /// Get summary of loaded resources
    pub fn summary(&self) -> ResourceLoadSummary {
        ResourceLoadSummary {
            agents: self.agents.count(),
            contexts: self.contexts.count(),
            triggers: self.triggers.count(),
        }
    }
}

/// Summary of loaded resources
#[derive(Debug, Default, Clone)]
pub struct ResourceLoadSummary {
    pub agents: usize,
    pub contexts: usize,
    pub triggers: usize,
}

impl ResourceLoadSummary {
    pub fn total(&self) -> usize {
        self.agents + self.contexts + self.triggers
    }
}

impl std::fmt::Display for ResourceLoadSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loaded {} resources: {} agents, {} contexts, {} triggers",
            self.total(),
            self.agents,
            self.contexts,
            self.triggers,
        )
    }
}

/// Validation error for cross-references
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub resource_type: String,
    pub resource_name: String,
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} '{}' field '{}': {}",
            self.resource_type, self.resource_name, self.field, self.message
        )
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Load a YAML file and deserialize to type T
fn load_yaml_file<T: serde::de::DeserializeOwned>(path: &Path) -> AgentixResult<T> {
    let content = std::fs::read_to_string(path)?;
    let resource: T = serde_yaml::from_str(&content)?;
    Ok(resource)
}

/// Check if a YAML file has a specific `kind` field value
/// Returns true if the file has the expected kind, false otherwise
fn yaml_file_has_kind(path: &Path, expected_kind: &str) -> bool {
    #[derive(serde::Deserialize)]
    struct KindCheck {
        kind: Option<String>,
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    match serde_yaml::from_str::<KindCheck>(&content) {
        Ok(check) => check.kind.as_deref() == Some(expected_kind),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_agent_registry() {
        let mut registry = AgentRegistry::new();

        let agent = AgentConfig {
            name: "test-agent".to_string(),
            model: "google:gemini-2.5-flash".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            provider: None,
            tools: vec![],
            mcp_servers: vec![],
            memory: None,
            max_context_messages: 10,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            output_schema: None,
            routing: None,
            extra: HashMap::new(),
            budget: None,
        };

        registry.register(agent).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.exists("test-agent"));
        assert!(registry.get("test-agent").is_some());
    }

    #[test]
    fn test_context_registry() {
        let mut registry = ContextRegistry::new();

        let yaml = r#"
apiVersion: agentix.dev/v1
kind: Context
metadata:
  name: test-context
spec:
  namespace: default
"#;
        let context: Context = serde_yaml::from_str(yaml).unwrap();

        registry.register(context).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.exists("test-context"));
    }

    #[test]
    fn test_trigger_registry() {
        let mut registry = TriggerRegistry::new();

        let yaml = r#"
apiVersion: agentix.dev/v1
kind: Trigger
metadata:
  name: test-trigger
spec:
  type: HTTP
  config: {}
"#;
        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();

        registry.register(trigger).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.exists("test-trigger"));
    }

    #[test]
    fn test_resource_manager_load_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directory structure
        std::fs::create_dir_all(root.join("agents")).unwrap();
        std::fs::create_dir_all(root.join("contexts")).unwrap();
        std::fs::create_dir_all(root.join("triggers")).unwrap();

        // Write agent file
        let agent_yaml = r#"
apiVersion: agentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: google:gemini-2.5-flash
"#;
        let mut file = std::fs::File::create(root.join("agents/test.yaml")).unwrap();
        file.write_all(agent_yaml.as_bytes()).unwrap();

        // Write context file
        let context_yaml = r#"
apiVersion: agentix.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
"#;
        let mut file = std::fs::File::create(root.join("contexts/prod.yaml")).unwrap();
        file.write_all(context_yaml.as_bytes()).unwrap();

        // Load all
        let mut manager = ResourceManager::new();
        let summary = manager.load_directory(root).unwrap();

        assert_eq!(summary.agents, 1);
        assert_eq!(summary.contexts, 1);
        assert!(manager.agents.exists("test-agent"));
        assert!(manager.contexts.exists("prod"));
    }
}
