use std::fmt;

/// Resource types supported by aofctl (kubectl-compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    // Core resources
    Agent,
    Workflow,
    Tool,
    Config,

    // Runtime resources
    Deployment,
    Template,

    // MCP resources
    McpServer,
    McpTool,

    // Execution resources
    Job,
    Task,

    // Storage resources
    Memory,
    State,
}

impl ResourceType {
    /// Get all available resource types
    pub fn all() -> Vec<ResourceType> {
        vec![
            ResourceType::Agent,
            ResourceType::Workflow,
            ResourceType::Tool,
            ResourceType::Config,
            ResourceType::Deployment,
            ResourceType::Template,
            ResourceType::McpServer,
            ResourceType::McpTool,
            ResourceType::Job,
            ResourceType::Task,
            ResourceType::Memory,
            ResourceType::State,
        ]
    }

    /// Get the canonical name for this resource type
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Agent => "agent",
            ResourceType::Workflow => "workflow",
            ResourceType::Tool => "tool",
            ResourceType::Config => "config",
            ResourceType::Deployment => "deployment",
            ResourceType::Template => "template",
            ResourceType::McpServer => "mcpserver",
            ResourceType::McpTool => "mcptool",
            ResourceType::Job => "job",
            ResourceType::Task => "task",
            ResourceType::Memory => "memory",
            ResourceType::State => "state",
        }
    }

    /// Get the plural name for this resource type
    pub fn plural(&self) -> &'static str {
        match self {
            ResourceType::Agent => "agents",
            ResourceType::Workflow => "workflows",
            ResourceType::Tool => "tools",
            ResourceType::Config => "configs",
            ResourceType::Deployment => "deployments",
            ResourceType::Template => "templates",
            ResourceType::McpServer => "mcpservers",
            ResourceType::McpTool => "mcptools",
            ResourceType::Job => "jobs",
            ResourceType::Task => "tasks",
            ResourceType::Memory => "memories",
            ResourceType::State => "states",
        }
    }

    /// Get short names/aliases for this resource type
    pub fn short_names(&self) -> Vec<&'static str> {
        match self {
            ResourceType::Agent => vec!["ag"],
            ResourceType::Workflow => vec!["wf", "workflow"],
            ResourceType::Tool => vec!["t"],
            ResourceType::Config => vec!["cfg"],
            ResourceType::Deployment => vec!["deploy", "dep"],
            ResourceType::Template => vec!["tmpl", "tpl"],
            ResourceType::McpServer => vec!["mcpsrv"],
            ResourceType::McpTool => vec!["mcpt"],
            ResourceType::Job => vec!["j"],
            ResourceType::Task => vec!["tsk"],
            ResourceType::Memory => vec!["mem"],
            ResourceType::State => vec!["st"],
        }
    }

    /// Get API version for this resource type
    pub fn api_version(&self) -> &'static str {
        match self {
            ResourceType::Agent | ResourceType::Workflow | ResourceType::Tool => "v1",
            ResourceType::Config => "v1",
            ResourceType::Deployment | ResourceType::Template => "apps/v1",
            ResourceType::McpServer | ResourceType::McpTool => "mcp/v1",
            ResourceType::Job | ResourceType::Task => "batch/v1",
            ResourceType::Memory | ResourceType::State => "storage/v1",
        }
    }

    /// Check if this resource is namespaced
    pub fn is_namespaced(&self) -> bool {
        match self {
            ResourceType::Config | ResourceType::McpServer => false,
            _ => true,
        }
    }

    /// Get resource kind (for kubectl compatibility)
    pub fn kind(&self) -> &'static str {
        match self {
            ResourceType::Agent => "Agent",
            ResourceType::Workflow => "Workflow",
            ResourceType::Tool => "Tool",
            ResourceType::Config => "Config",
            ResourceType::Deployment => "Deployment",
            ResourceType::Template => "Template",
            ResourceType::McpServer => "McpServer",
            ResourceType::McpTool => "McpTool",
            ResourceType::Job => "Job",
            ResourceType::Task => "Task",
            ResourceType::Memory => "Memory",
            ResourceType::State => "State",
        }
    }

    /// Parse resource type from string (supports name, plural, and short names)
    pub fn from_str(s: &str) -> Option<ResourceType> {
        let s_lower = s.to_lowercase();

        for rt in Self::all() {
            // Check name
            if rt.name() == s_lower {
                return Some(rt);
            }

            // Check plural
            if rt.plural() == s_lower {
                return Some(rt);
            }

            // Check short names
            if rt.short_names().contains(&s_lower.as_str()) {
                return Some(rt);
            }
        }

        None
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_parsing() {
        assert_eq!(ResourceType::from_str("agent"), Some(ResourceType::Agent));
        assert_eq!(ResourceType::from_str("agents"), Some(ResourceType::Agent));
        assert_eq!(ResourceType::from_str("ag"), Some(ResourceType::Agent));
        assert_eq!(ResourceType::from_str("Agent"), Some(ResourceType::Agent));

        assert_eq!(ResourceType::from_str("workflow"), Some(ResourceType::Workflow));
        assert_eq!(ResourceType::from_str("wf"), Some(ResourceType::Workflow));

        assert_eq!(ResourceType::from_str("invalid"), None);
    }

    #[test]
    fn test_resource_properties() {
        let agent = ResourceType::Agent;
        assert_eq!(agent.name(), "agent");
        assert_eq!(agent.plural(), "agents");
        assert_eq!(agent.kind(), "Agent");
        assert_eq!(agent.api_version(), "v1");
        assert!(agent.is_namespaced());
    }
}
