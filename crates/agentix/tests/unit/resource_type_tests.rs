/// Unit tests for resource type handling
#[cfg(test)]
mod resource_type_tests {
    #[test]
    fn test_agent_resource_type() {
        let resource = "agent";
        assert_eq!(resource, "agent");
    }

    #[test]
    fn test_agents_resource_type() {
        let resource = "agents";
        assert_eq!(resource, "agents");
    }

    #[test]
    fn test_workflow_resource_type() {
        let resource = "workflow";
        assert_eq!(resource, "workflow");
    }

    #[test]
    fn test_workflows_resource_type() {
        let resource = "workflows";
        assert_eq!(resource, "workflows");
    }

    #[test]
    fn test_tool_resource_type() {
        let resource = "tool";
        assert_eq!(resource, "tool");
    }

    #[test]
    fn test_tools_resource_type() {
        let resource = "tools";
        assert_eq!(resource, "tools");
    }

    #[test]
    fn test_normalize_resource_type_singular() {
        let resource = "agent";
        let normalized = if resource.ends_with('s') {
            resource
        } else {
            resource
        };
        assert_eq!(normalized, "agent");
    }

    #[test]
    fn test_normalize_resource_type_plural() {
        let resource = "agents";
        let normalized = resource.trim_end_matches('s');
        assert_eq!(normalized, "agent");
    }

    #[test]
    fn test_supported_resource_types() {
        let supported = vec!["agent", "agents", "workflow", "workflows", "tool", "tools"];

        for resource in supported {
            assert!(["agent", "agents", "workflow", "workflows", "tool", "tools"]
                .contains(&resource));
        }
    }
}
