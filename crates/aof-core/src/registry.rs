// AOF Core - Resource Registries
//
// Unified registries for loading, indexing, and resolving all AOF resource types.
// Each registry provides:
// - Directory loading (load all resources from a path)
// - Name-based lookup
// - Type-safe access to resources

use crate::agent::AgentConfig;
use crate::agentflow::AgentFlow;
use crate::binding::FlowBinding;
use crate::context::Context;
use crate::error::{AofError, AofResult};
use crate::trigger::Trigger;

use std::collections::HashMap;
use std::path::Path;

/// Common trait for all resource registries
pub trait Registry<T> {
    /// Load all resources from a directory
    fn load_directory(&mut self, path: &Path) -> AofResult<usize>;

    /// Get a resource by name
    fn get(&self, name: &str) -> Option<&T>;

    /// Get all resources
    fn get_all(&self) -> Vec<&T>;

    /// Register a resource
    fn register(&mut self, resource: T) -> AofResult<()>;

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
    fn load_directory(&mut self, path: &Path) -> AofResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
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

    fn register(&mut self, resource: AgentConfig) -> AofResult<()> {
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
    fn load_directory(&mut self, path: &Path) -> AofResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
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

    fn register(&mut self, resource: Context) -> AofResult<()> {
        resource.validate().map_err(|e| AofError::Config(e))?;
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
    fn load_directory(&mut self, path: &Path) -> AofResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
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

    fn register(&mut self, resource: Trigger) -> AofResult<()> {
        resource.validate().map_err(|e| AofError::Config(e))?;
        let name = resource.metadata.name.clone();
        self.triggers.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.triggers.len()
    }
}

// ============================================================================
// Flow Registry
// ============================================================================

/// Registry for AgentFlow resources
#[derive(Debug, Default)]
pub struct FlowRegistry {
    flows: HashMap<String, AgentFlow>,
}

impl FlowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get flow names
    pub fn names(&self) -> Vec<&str> {
        self.flows.keys().map(|s| s.as_str()).collect()
    }
}

impl Registry<AgentFlow> for FlowRegistry {
    fn load_directory(&mut self, path: &Path) -> AofResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                match load_yaml_file::<AgentFlow>(&file_path) {
                    Ok(flow) => {
                        if let Err(e) = flow.validate() {
                            tracing::warn!("Invalid flow in {:?}: {}", file_path, e);
                            continue;
                        }
                        let name = flow.metadata.name.clone();
                        self.flows.insert(name.clone(), flow);
                        tracing::debug!("Loaded flow: {}", name);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load flow from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn get(&self, name: &str) -> Option<&AgentFlow> {
        self.flows.get(name)
    }

    fn get_all(&self) -> Vec<&AgentFlow> {
        self.flows.values().collect()
    }

    fn register(&mut self, resource: AgentFlow) -> AofResult<()> {
        resource.validate().map_err(|e| AofError::Config(e))?;
        let name = resource.metadata.name.clone();
        self.flows.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.flows.len()
    }
}

// ============================================================================
// Binding Registry
// ============================================================================

/// Registry for FlowBinding resources
#[derive(Debug, Default)]
pub struct BindingRegistry {
    bindings: HashMap<String, FlowBinding>,
}

impl BindingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get binding names
    pub fn names(&self) -> Vec<&str> {
        self.bindings.keys().map(|s| s.as_str()).collect()
    }

    /// Get all bindings for a specific trigger
    pub fn get_bindings_for_trigger(&self, trigger_name: &str) -> Vec<&FlowBinding> {
        self.bindings
            .values()
            .filter(|b| b.spec.trigger == trigger_name && b.spec.enabled)
            .collect()
    }

    /// Get all bindings for a specific context
    pub fn get_bindings_for_context(&self, context_name: &str) -> Vec<&FlowBinding> {
        self.bindings
            .values()
            .filter(|b| b.spec.context.as_deref() == Some(context_name) && b.spec.enabled)
            .collect()
    }

    /// Get all enabled bindings
    pub fn get_enabled(&self) -> Vec<&FlowBinding> {
        self.bindings.values().filter(|b| b.spec.enabled).collect()
    }

    /// Find best matching binding for a message
    pub fn find_best_match(
        &self,
        trigger_name: &str,
        channel: Option<&str>,
        user: Option<&str>,
        text: Option<&str>,
    ) -> Option<&FlowBinding> {
        let bindings = self.get_bindings_for_trigger(trigger_name);

        bindings
            .into_iter()
            .filter(|b| b.matches(channel, user, text))
            .max_by_key(|b| b.match_score(channel, user, text))
    }
}

impl Registry<FlowBinding> for BindingRegistry {
    fn load_directory(&mut self, path: &Path) -> AofResult<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                match load_yaml_file::<FlowBinding>(&file_path) {
                    Ok(binding) => {
                        if let Err(e) = binding.validate() {
                            tracing::warn!("Invalid binding in {:?}: {}", file_path, e);
                            continue;
                        }
                        let name = binding.metadata.name.clone();
                        self.bindings.insert(name.clone(), binding);
                        tracing::debug!("Loaded binding: {}", name);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load binding from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn get(&self, name: &str) -> Option<&FlowBinding> {
        self.bindings.get(name)
    }

    fn get_all(&self) -> Vec<&FlowBinding> {
        self.bindings.values().collect()
    }

    fn register(&mut self, resource: FlowBinding) -> AofResult<()> {
        resource.validate().map_err(|e| AofError::Config(e))?;
        let name = resource.metadata.name.clone();
        self.bindings.insert(name, resource);
        Ok(())
    }

    fn count(&self) -> usize {
        self.bindings.len()
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
    pub flows: FlowRegistry,
    pub bindings: BindingRegistry,
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
    /// ├── triggers/
    /// ├── flows/
    /// └── bindings/
    /// ```
    pub fn load_directory(&mut self, root: &Path) -> AofResult<ResourceLoadSummary> {
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

        // Load flows
        let flows_dir = root.join("flows");
        if flows_dir.exists() {
            summary.flows = self.flows.load_directory(&flows_dir)?;
        }

        // Load bindings
        let bindings_dir = root.join("bindings");
        if bindings_dir.exists() {
            summary.bindings = self.bindings.load_directory(&bindings_dir)?;
        }

        Ok(summary)
    }

    /// Validate all cross-references between resources
    pub fn validate_references(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Validate binding references
        for binding in self.bindings.get_all() {
            // Check trigger reference
            if !self.triggers.exists(&binding.spec.trigger) {
                errors.push(ValidationError {
                    resource_type: "FlowBinding".to_string(),
                    resource_name: binding.metadata.name.clone(),
                    field: "trigger".to_string(),
                    message: format!("Referenced trigger '{}' not found", binding.spec.trigger),
                });
            }

            // Check context reference (if specified)
            if let Some(ref context_name) = binding.spec.context {
                if !self.contexts.exists(context_name) {
                    errors.push(ValidationError {
                        resource_type: "FlowBinding".to_string(),
                        resource_name: binding.metadata.name.clone(),
                        field: "context".to_string(),
                        message: format!("Referenced context '{}' not found", context_name),
                    });
                }
            }

            // Check flow reference
            if !binding.spec.flow.is_empty() && !self.flows.exists(&binding.spec.flow) {
                errors.push(ValidationError {
                    resource_type: "FlowBinding".to_string(),
                    resource_name: binding.metadata.name.clone(),
                    field: "flow".to_string(),
                    message: format!("Referenced flow '{}' not found", binding.spec.flow),
                });
            }

            // Check agent reference (if specified)
            if let Some(ref agent_name) = binding.spec.agent {
                if !self.agents.exists(agent_name) {
                    errors.push(ValidationError {
                        resource_type: "FlowBinding".to_string(),
                        resource_name: binding.metadata.name.clone(),
                        field: "agent".to_string(),
                        message: format!("Referenced agent '{}' not found", agent_name),
                    });
                }
            }
        }

        errors
    }

    /// Get summary of loaded resources
    pub fn summary(&self) -> ResourceLoadSummary {
        ResourceLoadSummary {
            agents: self.agents.count(),
            contexts: self.contexts.count(),
            triggers: self.triggers.count(),
            flows: self.flows.count(),
            bindings: self.bindings.count(),
        }
    }
}

/// Summary of loaded resources
#[derive(Debug, Default, Clone)]
pub struct ResourceLoadSummary {
    pub agents: usize,
    pub contexts: usize,
    pub triggers: usize,
    pub flows: usize,
    pub bindings: usize,
}

impl ResourceLoadSummary {
    pub fn total(&self) -> usize {
        self.agents + self.contexts + self.triggers + self.flows + self.bindings
    }
}

impl std::fmt::Display for ResourceLoadSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loaded {} resources: {} agents, {} contexts, {} triggers, {} flows, {} bindings",
            self.total(),
            self.agents,
            self.contexts,
            self.triggers,
            self.flows,
            self.bindings
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
fn load_yaml_file<T: serde::de::DeserializeOwned>(path: &Path) -> AofResult<T> {
    let content = std::fs::read_to_string(path)?;
    let resource: T = serde_yaml::from_str(&content)?;
    Ok(resource)
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
            extra: HashMap::new(),
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
apiVersion: aof.dev/v1
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
apiVersion: aof.dev/v1
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
    fn test_binding_registry_find_best_match() {
        let mut registry = BindingRegistry::new();

        // General binding
        let yaml1 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: general
spec:
  trigger: slack
  flow: general-flow
"#;
        // Specific binding for kubectl
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: kubectl-specific
spec:
  trigger: slack
  flow: k8s-flow
  match:
    patterns: [kubectl]
    channels: [production]
"#;

        let binding1: FlowBinding = serde_yaml::from_str(yaml1).unwrap();
        let binding2: FlowBinding = serde_yaml::from_str(yaml2).unwrap();

        registry.register(binding1).unwrap();
        registry.register(binding2).unwrap();

        // Test that more specific binding wins
        let best = registry.find_best_match(
            "slack",
            Some("production"),
            None,
            Some("kubectl get pods"),
        );

        assert!(best.is_some());
        assert_eq!(best.unwrap().metadata.name, "kubectl-specific");
    }

    #[test]
    fn test_resource_manager_load_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directory structure
        std::fs::create_dir_all(root.join("agents")).unwrap();
        std::fs::create_dir_all(root.join("contexts")).unwrap();
        std::fs::create_dir_all(root.join("triggers")).unwrap();
        std::fs::create_dir_all(root.join("bindings")).unwrap();

        // Write agent file
        let agent_yaml = r#"
apiVersion: aof.dev/v1
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
apiVersion: aof.dev/v1
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

    #[test]
    fn test_validate_references() {
        let mut manager = ResourceManager::new();

        // Add a trigger
        let trigger_yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-trigger
spec:
  type: HTTP
  config: {}
"#;
        let trigger: Trigger = serde_yaml::from_str(trigger_yaml).unwrap();
        manager.triggers.register(trigger).unwrap();

        // Add a binding referencing non-existent resources
        let binding_yaml = r#"
apiVersion: aof.dev/v1
kind: FlowBinding
metadata:
  name: bad-binding
spec:
  trigger: slack-trigger
  context: non-existent-context
  flow: non-existent-flow
"#;
        let binding: FlowBinding = serde_yaml::from_str(binding_yaml).unwrap();
        manager.bindings.register(binding).unwrap();

        let errors = manager.validate_references();
        assert_eq!(errors.len(), 2); // context and flow not found
    }
}
