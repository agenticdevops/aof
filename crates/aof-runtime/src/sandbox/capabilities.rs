//! Linux capability management for sandbox isolation
//!
//! This module provides capability dropping and allowlisting for different
//! tool types. Default is --cap-drop=ALL for maximum isolation.

use serde::{Deserialize, Serialize};

/// Capability configuration for a tool
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Drop all capabilities by default
    pub drop_all: bool,
    /// Capabilities to add back (allowlist)
    pub add: Vec<String>,
}

impl CapabilityConfig {
    /// Create a new capability configuration
    pub fn new(drop_all: bool, add: Vec<String>) -> Self {
        Self { drop_all, add }
    }

    /// Get the appropriate capability configuration for a tool
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool (e.g., "kubectl", "docker")
    ///
    /// # Returns
    /// Capability configuration for the tool with appropriate allowlist
    pub fn for_tool(tool_name: &str) -> Self {
        match tool_name {
            // kubectl uses kubeconfig file, no special capabilities needed
            "kubectl" | "k9s" => Self::new(true, vec![]),

            // docker communicates via socket mount, no capabilities needed
            "docker" => Self::new(true, vec![]),

            // Tools that might need port binding below 1024
            "nc" | "socat" | "ncat" => Self::new(
                true,
                vec!["CAP_NET_BIND_SERVICE".to_string()],
            ),

            // Default: drop all capabilities
            _ => Self::new(true, vec![]),
        }
    }

    /// Generate Docker capability arguments
    ///
    /// # Returns
    /// Vector of Docker CLI arguments for capability configuration
    pub fn docker_cap_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.drop_all {
            args.push("--cap-drop=ALL".to_string());
        }

        for cap in &self.add {
            args.push(format!("--cap-add={}", cap));
        }

        args
    }

    /// Check if configuration allows a specific capability
    pub fn allows(&self, capability: &str) -> bool {
        if !self.drop_all {
            return true; // All capabilities allowed
        }

        self.add.iter().any(|c| c == capability)
    }

    /// Get number of capabilities in allowlist
    pub fn allowlist_count(&self) -> usize {
        self.add.len()
    }
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self::new(true, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_drops_all() {
        let config = CapabilityConfig::default();
        assert!(config.drop_all);
        assert!(config.add.is_empty());
    }

    #[test]
    fn test_kubectl_no_capabilities() {
        let config = CapabilityConfig::for_tool("kubectl");
        assert!(config.drop_all);
        assert_eq!(config.allowlist_count(), 0);
    }

    #[test]
    fn test_docker_no_capabilities() {
        let config = CapabilityConfig::for_tool("docker");
        assert!(config.drop_all);
        assert_eq!(config.allowlist_count(), 0);
    }

    #[test]
    fn test_nc_gets_net_bind_service() {
        let config = CapabilityConfig::for_tool("nc");
        assert!(config.drop_all);
        assert_eq!(config.allowlist_count(), 1);
        assert!(config.allows("CAP_NET_BIND_SERVICE"));
    }

    #[test]
    fn test_unknown_tool_drops_all() {
        let config = CapabilityConfig::for_tool("unknown-tool");
        assert!(config.drop_all);
        assert_eq!(config.allowlist_count(), 0);
    }

    #[test]
    fn test_docker_cap_args_drop_all() {
        let config = CapabilityConfig::default();
        let args = config.docker_cap_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "--cap-drop=ALL");
    }

    #[test]
    fn test_docker_cap_args_with_allowlist() {
        let config = CapabilityConfig::new(
            true,
            vec!["CAP_NET_BIND_SERVICE".to_string(), "CAP_SYS_ADMIN".to_string()],
        );
        let args = config.docker_cap_args();
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "--cap-drop=ALL");
        assert_eq!(args[1], "--cap-add=CAP_NET_BIND_SERVICE");
        assert_eq!(args[2], "--cap-add=CAP_SYS_ADMIN");
    }

    #[test]
    fn test_allows_capability() {
        let config = CapabilityConfig::new(
            true,
            vec!["CAP_NET_BIND_SERVICE".to_string()],
        );
        assert!(config.allows("CAP_NET_BIND_SERVICE"));
        assert!(!config.allows("CAP_SYS_ADMIN"));
    }

    #[test]
    fn test_allows_all_when_not_dropped() {
        let config = CapabilityConfig::new(false, vec![]);
        assert!(config.allows("CAP_SYS_ADMIN"));
        assert!(config.allows("CAP_NET_BIND_SERVICE"));
    }
}
