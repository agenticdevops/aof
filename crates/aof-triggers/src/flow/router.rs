//! FlowRouter - Simple flow lookup by name
//!
//! After the architecture simplification:
//! - Triggers contain command bindings that route to agents/fleets/flows
//! - AgentFlows no longer contain embedded triggers
//! - Routing decisions are made at the Trigger level, not the Flow level
//!
//! This router now provides simple flow lookup from a registry.

use std::sync::Arc;

use aof_core::AgentFlow;

use super::registry::FlowRegistry;

/// Match result containing a flow
#[derive(Debug, Clone)]
pub struct FlowMatch {
    /// The matched flow
    pub flow: Arc<AgentFlow>,

    /// Match score (higher = better match)
    pub score: u32,

    /// Why this flow matched
    pub reason: MatchReason,
}

/// Reason for flow match
#[derive(Debug, Clone)]
pub enum MatchReason {
    /// Matched via command binding in Trigger
    CommandBinding(String),

    /// Matched as explicit default flow
    ExplicitDefault,

    /// Direct flow lookup by name
    DirectLookup,
}

/// FlowRouter provides flow lookup from a registry
pub struct FlowRouter {
    /// Flow registry
    registry: Arc<FlowRegistry>,

    /// Default flow name (fallback)
    default_flow: Option<String>,
}

impl FlowRouter {
    /// Create a new router with a registry
    pub fn new(registry: Arc<FlowRegistry>) -> Self {
        Self {
            registry,
            default_flow: None,
        }
    }

    /// Create router with a default flow
    pub fn with_default(registry: Arc<FlowRegistry>, default_flow: impl Into<String>) -> Self {
        Self {
            registry,
            default_flow: Some(default_flow.into()),
        }
    }

    /// Set the default flow
    pub fn set_default(&mut self, flow_name: impl Into<String>) {
        self.default_flow = Some(flow_name.into());
    }

    /// Get a flow by name, falling back to default if not found
    pub fn get_flow(&self, name: &str) -> Option<Arc<AgentFlow>> {
        self.registry.get(name).or_else(|| {
            self.default_flow
                .as_ref()
                .and_then(|default| self.registry.get(default))
        })
    }

    /// Get the default flow
    pub fn get_default(&self) -> Option<Arc<AgentFlow>> {
        self.default_flow
            .as_ref()
            .and_then(|name| self.registry.get(name))
    }

    /// Get the registry
    pub fn registry(&self) -> &Arc<FlowRegistry> {
        &self.registry
    }

    /// List all flow names
    pub fn list_flows(&self) -> Vec<String> {
        self.registry.list_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::AgentFlow;

    #[test]
    fn test_router_new() {
        let registry = Arc::new(FlowRegistry::new());
        let router = FlowRouter::new(registry);
        assert!(router.default_flow.is_none());
    }

    #[test]
    fn test_router_get_flow() {
        let registry = Arc::new(FlowRegistry::new());

        let flow: AgentFlow = serde_yaml::from_str(
            r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: test-flow
spec:
  nodes:
    - id: process
      type: End
  connections:
    - from: start
      to: process
"#,
        )
        .unwrap();

        registry.register(flow);

        let router = FlowRouter::new(registry);
        let retrieved = router.get_flow("test-flow");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "test-flow");
    }

    #[test]
    fn test_router_with_default() {
        let registry = Arc::new(FlowRegistry::new());

        let flow: AgentFlow = serde_yaml::from_str(
            r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: default-flow
spec:
  nodes:
    - id: process
      type: End
  connections:
    - from: start
      to: process
"#,
        )
        .unwrap();

        registry.register(flow);

        let router = FlowRouter::with_default(registry, "default-flow");

        // Non-existent flow should fallback to default
        let result = router.get_flow("non-existent");
        assert!(result.is_some());
        assert_eq!(result.unwrap().metadata.name, "default-flow");
    }

    #[test]
    fn test_router_list_flows() {
        let registry = Arc::new(FlowRegistry::new());

        for i in 0..3 {
            let flow: AgentFlow = serde_yaml::from_str(&format!(
                r#"
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: flow-{}
spec:
  nodes:
    - id: process
      type: End
  connections:
    - from: start
      to: process
"#,
                i
            ))
            .unwrap();
            registry.register(flow);
        }

        let router = FlowRouter::new(registry);
        let names = router.list_flows();
        assert_eq!(names.len(), 3);
    }
}
