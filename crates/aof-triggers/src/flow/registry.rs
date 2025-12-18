//! FlowRegistry - Loads and manages AgentFlow configurations
//!
//! The FlowRegistry provides:
//! - Loading flows from YAML files in a directory
//! - Hot-reload with file watching (optional)
//! - Flow lookup by name and trigger matching

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aof_core::{AgentFlow, AofResult, AofError, TriggerType};
use dashmap::DashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// FlowRegistry manages AgentFlow configurations
pub struct FlowRegistry {
    /// Flows by name
    flows: DashMap<String, Arc<AgentFlow>>,

    /// Flows directory (for reloading)
    flows_dir: Option<PathBuf>,

    /// File watcher state (for hot-reload)
    #[allow(dead_code)]
    watch_enabled: bool,
}

impl FlowRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            flows: DashMap::new(),
            flows_dir: None,
            watch_enabled: false,
        }
    }

    /// Create registry and load flows from a directory
    pub async fn from_directory(dir: impl AsRef<Path>) -> AofResult<Self> {
        let dir = dir.as_ref();
        let mut registry = Self::new();
        registry.flows_dir = Some(dir.to_path_buf());
        registry.load_directory(dir).await?;
        Ok(registry)
    }

    /// Load all flow files from a directory
    pub async fn load_directory(&self, dir: impl AsRef<Path>) -> AofResult<usize> {
        let dir = dir.as_ref();

        if !dir.exists() {
            return Err(AofError::Config(format!(
                "Flows directory does not exist: {}",
                dir.display()
            )));
        }

        let mut loaded = 0;

        // Read directory entries
        let entries = std::fs::read_dir(dir).map_err(|e| {
            AofError::Config(format!("Failed to read flows directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Only load .yaml and .yml files
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    match self.load_file(&path).await {
                        Ok(flow_name) => {
                            info!("Loaded flow: {} from {}", flow_name, path.display());
                            loaded += 1;
                        }
                        Err(e) => {
                            warn!("Failed to load flow from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Loaded {} flows from {}", loaded, dir.display());
        Ok(loaded)
    }

    /// Load a single flow file
    pub async fn load_file(&self, path: impl AsRef<Path>) -> AofResult<String> {
        let path = path.as_ref();

        let content = std::fs::read_to_string(path).map_err(|e| {
            AofError::Config(format!("Failed to read flow file {}: {}", path.display(), e))
        })?;

        let flow: AgentFlow = serde_yaml::from_str(&content).map_err(|e| {
            AofError::Config(format!("Failed to parse flow file {}: {}", path.display(), e))
        })?;

        // Validate the flow
        flow.validate().map_err(|e| {
            AofError::Config(format!("Flow validation failed for {}: {}", path.display(), e))
        })?;

        let name = flow.metadata.name.clone();
        self.flows.insert(name.clone(), Arc::new(flow));

        Ok(name)
    }

    /// Register a flow directly (for testing or programmatic use)
    pub fn register(&self, flow: AgentFlow) -> String {
        let name = flow.metadata.name.clone();
        self.flows.insert(name.clone(), Arc::new(flow));
        name
    }

    /// Get a flow by name
    pub fn get(&self, name: &str) -> Option<Arc<AgentFlow>> {
        self.flows.get(name).map(|r| r.value().clone())
    }

    /// Remove a flow
    pub fn remove(&self, name: &str) -> Option<Arc<AgentFlow>> {
        self.flows.remove(name).map(|(_, v)| v)
    }

    /// List all flow names
    pub fn list(&self) -> Vec<String> {
        self.flows.iter().map(|r| r.key().clone()).collect()
    }

    /// Get all flows
    pub fn all(&self) -> Vec<Arc<AgentFlow>> {
        self.flows.iter().map(|r| r.value().clone()).collect()
    }

    /// Get flows by trigger type
    pub fn by_trigger_type(&self, trigger_type: TriggerType) -> Vec<Arc<AgentFlow>> {
        self.flows
            .iter()
            .filter(|r| r.value().spec.trigger.trigger_type == trigger_type)
            .map(|r| r.value().clone())
            .collect()
    }

    /// Get flows for a specific platform (Slack, Discord, etc.)
    pub fn by_platform(&self, platform: &str) -> Vec<Arc<AgentFlow>> {
        let trigger_type = match platform.to_lowercase().as_str() {
            "slack" => TriggerType::Slack,
            "discord" => TriggerType::Discord,
            "telegram" => TriggerType::Telegram,
            "whatsapp" => TriggerType::WhatsApp,
            "http" | "webhook" => TriggerType::HTTP,
            "schedule" | "cron" => TriggerType::Schedule,
            _ => return vec![],
        };

        self.by_trigger_type(trigger_type)
    }

    /// Number of registered flows
    pub fn len(&self) -> usize {
        self.flows.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    /// Reload all flows from the configured directory
    pub async fn reload(&self) -> AofResult<usize> {
        if let Some(ref dir) = self.flows_dir {
            // Clear existing flows
            self.flows.clear();
            // Reload from directory
            self.load_directory(dir).await
        } else {
            Err(AofError::Config("No flows directory configured".to_string()))
        }
    }
}

impl Default for FlowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = FlowRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_register_get() {
        let registry = FlowRegistry::new();

        let flow: AgentFlow = serde_yaml::from_str(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: test-flow
spec:
  trigger:
    type: HTTP
  nodes:
    - id: process
      type: End
"#).unwrap();

        let name = registry.register(flow);
        assert_eq!(name, "test-flow");

        let retrieved = registry.get("test-flow");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "test-flow");
    }

    #[test]
    fn test_registry_list() {
        let registry = FlowRegistry::new();

        for i in 0..3 {
            let flow: AgentFlow = serde_yaml::from_str(&format!(r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: flow-{}
spec:
  trigger:
    type: HTTP
  nodes:
    - id: process
      type: End
"#, i)).unwrap();
            registry.register(flow);
        }

        let names = registry.list();
        assert_eq!(names.len(), 3);
    }
}
