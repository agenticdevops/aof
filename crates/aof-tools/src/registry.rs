//! Tool Registry - Central registration and discovery for tools
//!
//! The registry provides a way to organize, discover, and execute tools.
//! It supports both individual tool registration and bulk category registration.

use aof_core::{AofError, AofResult, Tool, ToolDefinition, ToolExecutor, ToolInput, ToolResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Tool category for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// File system operations
    File,
    /// Shell command execution
    Shell,
    /// Kubernetes operations
    Kubectl,
    /// Docker container operations
    Docker,
    /// Git repository operations
    Git,
    /// Terraform IaC operations
    Terraform,
    /// HTTP request operations
    Http,
    /// Custom user-defined tools
    Custom(String),
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::File => write!(f, "file"),
            ToolCategory::Shell => write!(f, "shell"),
            ToolCategory::Kubectl => write!(f, "kubectl"),
            ToolCategory::Docker => write!(f, "docker"),
            ToolCategory::Git => write!(f, "git"),
            ToolCategory::Terraform => write!(f, "terraform"),
            ToolCategory::Http => write!(f, "http"),
            ToolCategory::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Tool registry for managing available tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    categories: HashMap<ToolCategory, Vec<String>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with all default tools enabled
    #[cfg(feature = "all")]
    pub fn with_all_defaults() -> Self {
        let mut registry = Self::new();

        #[cfg(feature = "file")]
        registry.register_category(crate::tools::file::FileTools::all());

        #[cfg(feature = "shell")]
        registry.register(crate::tools::shell::ShellTool::new());

        #[cfg(feature = "kubectl")]
        registry.register_category(crate::tools::kubectl::KubectlTools::all());

        #[cfg(feature = "docker")]
        registry.register_category(crate::tools::docker::DockerTools::all());

        #[cfg(feature = "git")]
        registry.register_category(crate::tools::git::GitTools::all());

        #[cfg(feature = "terraform")]
        registry.register_category(crate::tools::terraform::TerraformTools::all());

        registry
    }

    /// Register a single tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> &mut Self {
        let name = tool.config().name.clone();
        info!(tool = %name, "Registering tool");
        self.tools.insert(name, Arc::new(tool));
        self
    }

    /// Register a tool with a specific category
    pub fn register_with_category<T: Tool + 'static>(
        &mut self,
        tool: T,
        category: ToolCategory,
    ) -> &mut Self {
        let name = tool.config().name.clone();
        self.tools.insert(name.clone(), Arc::new(tool));
        self.categories
            .entry(category)
            .or_default()
            .push(name);
        self
    }

    /// Register multiple tools from a category
    pub fn register_category(&mut self, tools: Vec<Box<dyn Tool>>) -> &mut Self {
        for tool in tools {
            let name = tool.config().name.clone();
            self.tools.insert(name, Arc::from(tool));
        }
        self
    }

    /// Register tools with category tracking
    pub fn register_category_with_name(
        &mut self,
        category: ToolCategory,
        tools: Vec<Box<dyn Tool>>,
    ) -> &mut Self {
        for tool in tools {
            let name = tool.config().name.clone();
            self.categories
                .entry(category.clone())
                .or_default()
                .push(name.clone());
            self.tools.insert(name, Arc::from(tool));
        }
        self
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// List all tool names
    pub fn list_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// List tool definitions
    pub fn list_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// List tools by category
    pub fn list_by_category(&self, category: &ToolCategory) -> Vec<Arc<dyn Tool>> {
        self.categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| self.tools.get(n).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get tool count
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Convert registry into a tool executor
    pub fn into_executor(self) -> BuiltinToolExecutor {
        BuiltinToolExecutor::new(self)
    }

    /// Create executor reference without consuming registry
    pub fn as_executor(&self) -> BuiltinToolExecutor {
        BuiltinToolExecutor {
            tools: self.tools.clone(),
        }
    }
}

/// Built-in tool executor that wraps the registry
pub struct BuiltinToolExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl BuiltinToolExecutor {
    /// Create from registry
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            tools: registry.tools,
        }
    }

    /// Create from a list of tools
    pub fn from_tools(tools: Vec<Box<dyn Tool>>) -> Self {
        let mut map = HashMap::new();
        for tool in tools {
            let name = tool.config().name.clone();
            map.insert(name, Arc::from(tool));
        }
        Self { tools: map }
    }
}

#[async_trait]
impl ToolExecutor for BuiltinToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        let tool = self.tools.get(name).ok_or_else(|| {
            AofError::tool(format!("Tool not found: {}", name))
        })?;

        debug!(tool = %name, "Executing built-in tool");
        let start = std::time::Instant::now();

        match tool.execute(input).await {
            Ok(result) => {
                let elapsed = start.elapsed().as_millis() as u64;
                debug!(tool = %name, elapsed_ms = %elapsed, success = %result.success, "Tool execution complete");
                Ok(result.with_execution_time(elapsed))
            }
            Err(e) => {
                warn!(tool = %name, error = %e, "Tool execution failed");
                Err(e)
            }
        }
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
}

/// Composite executor that combines multiple executors
/// Useful for combining built-in tools with MCP tools
pub struct CompositeToolExecutor {
    executors: Vec<Box<dyn ToolExecutor>>,
}

impl CompositeToolExecutor {
    /// Create new composite executor
    pub fn new() -> Self {
        Self { executors: vec![] }
    }

    /// Add an executor
    pub fn add_executor<E: ToolExecutor + 'static>(mut self, executor: E) -> Self {
        self.executors.push(Box::new(executor));
        self
    }

    /// Add a boxed executor
    pub fn add_boxed(mut self, executor: Box<dyn ToolExecutor>) -> Self {
        self.executors.push(executor);
        self
    }
}

impl Default for CompositeToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for CompositeToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        // Try each executor until one can handle the tool
        for executor in &self.executors {
            if executor.get_tool(name).is_some() {
                return executor.execute_tool(name, input).await;
            }
        }
        Err(AofError::tool(format!("Tool not found in any executor: {}", name)))
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for executor in &self.executors {
            for tool in executor.list_tools() {
                if seen.insert(tool.name.clone()) {
                    tools.push(tool);
                }
            }
        }
        tools
    }

    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        for executor in &self.executors {
            if let Some(tool) = executor.get_tool(name) {
                return Some(tool);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::ToolConfig;

    struct MockTool {
        config: ToolConfig,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                config: ToolConfig {
                    name: name.to_string(),
                    description: format!("Mock tool: {}", name),
                    parameters: serde_json::json!({}),
                    tool_type: aof_core::ToolType::Custom,
                    timeout_secs: 30,
                    extra: HashMap::new(),
                },
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        async fn execute(&self, _input: ToolInput) -> AofResult<ToolResult> {
            Ok(ToolResult::success(serde_json::json!({"mock": true})))
        }

        fn config(&self) -> &ToolConfig {
            &self.config
        }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("test_tool"));

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test_tool").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list_names() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("tool1"));
        registry.register(MockTool::new("tool2"));

        let names = registry.list_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"tool1".to_string()));
        assert!(names.contains(&"tool2".to_string()));
    }

    #[tokio::test]
    async fn test_executor_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("test_tool"));

        let executor = registry.into_executor();
        let input = ToolInput::new(serde_json::json!({}));

        let result = executor.execute_tool("test_tool", input).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_executor_tool_not_found() {
        let registry = ToolRegistry::new();
        let executor = registry.into_executor();
        let input = ToolInput::new(serde_json::json!({}));

        let result = executor.execute_tool("nonexistent", input).await;
        assert!(result.is_err());
    }
}
