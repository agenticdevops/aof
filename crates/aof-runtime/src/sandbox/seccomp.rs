//! Seccomp profile management for sandbox isolation
//!
//! This module provides custom seccomp profile selection and management
//! for different tool types, enabling stricter syscall filtering than
//! Docker's default profile.

use aof_core::error::{AofError, AofResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Seccomp profile metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeccompProfile {
    /// Profile name (e.g., "default", "kubectl", "docker", "readonly")
    pub name: String,
    /// Path to the seccomp JSON file
    pub path: PathBuf,
    /// Tool types this profile applies to
    pub tool_types: Vec<String>,
}

impl SeccompProfile {
    /// Create a new seccomp profile
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>, tool_types: Vec<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            tool_types,
        }
    }
}

/// Manages seccomp profiles for different tool types
pub struct SeccompProfileManager {
    profiles_dir: PathBuf,
    cache: HashMap<String, SeccompProfile>,
    default_profile: SeccompProfile,
}

impl SeccompProfileManager {
    /// Create a new seccomp profile manager
    ///
    /// # Arguments
    /// * `profiles_dir` - Directory containing seccomp profile JSON files
    ///
    /// # Returns
    /// A new profile manager with all profiles loaded and cached
    pub fn new(profiles_dir: impl Into<PathBuf>) -> AofResult<Self> {
        let profiles_dir = profiles_dir.into();

        if !profiles_dir.exists() {
            return Err(AofError::config(format!(
                "Seccomp profiles directory does not exist: {}",
                profiles_dir.display()
            )));
        }

        let mut cache = HashMap::new();

        // Define profile mappings
        let profiles = vec![
            SeccompProfile::new(
                "kubectl",
                profiles_dir.join("kubectl-profile.json"),
                vec!["kubectl".to_string(), "k9s".to_string()],
            ),
            SeccompProfile::new(
                "docker",
                profiles_dir.join("docker-profile.json"),
                vec!["docker".to_string()],
            ),
            SeccompProfile::new(
                "readonly",
                profiles_dir.join("readonly-profile.json"),
                vec!["cat".to_string(), "grep".to_string(), "ls".to_string()],
            ),
        ];

        // Load profiles into cache
        for profile in profiles {
            if !profile.path.exists() {
                tracing::warn!(
                    "Seccomp profile {} not found at {}, will use default",
                    profile.name,
                    profile.path.display()
                );
                continue;
            }

            for tool_type in &profile.tool_types {
                cache.insert(tool_type.clone(), profile.clone());
            }
        }

        let default_profile = SeccompProfile::new(
            "default",
            profiles_dir.join("default.json"),
            vec!["*".to_string()],
        );

        if !default_profile.path.exists() {
            return Err(AofError::config(format!(
                "Default seccomp profile not found at {}",
                default_profile.path.display()
            )));
        }

        Ok(Self {
            profiles_dir,
            cache,
            default_profile,
        })
    }

    /// Get the appropriate seccomp profile for a tool
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool (e.g., "kubectl", "docker", "cat")
    ///
    /// # Returns
    /// The most specific profile for the tool, falling back to default
    pub fn profile_for_tool(&self, tool_name: &str) -> &SeccompProfile {
        self.cache
            .get(tool_name)
            .unwrap_or(&self.default_profile)
    }

    /// Generate Docker security option for seccomp profile
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool to get security options for
    ///
    /// # Returns
    /// Docker `--security-opt` argument string
    pub fn docker_security_opt(&self, tool_name: &str) -> String {
        let profile = self.profile_for_tool(tool_name);
        format!("seccomp={}", profile.path.display())
    }

    /// Get all loaded profiles
    pub fn profiles(&self) -> Vec<&SeccompProfile> {
        let mut profiles: Vec<&SeccompProfile> = self.cache.values().collect();
        profiles.push(&self.default_profile);
        profiles.sort_by_key(|p| &p.name);
        profiles.dedup_by_key(|p| &p.name);
        profiles
    }

    /// Get profiles directory
    pub fn profiles_dir(&self) -> &Path {
        &self.profiles_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_profiles() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("aof-test-seccomp");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create minimal valid seccomp profiles
        let default_profile = r#"{"defaultAction": "SCMP_ACT_ERRNO", "syscalls": []}"#;
        fs::write(temp_dir.join("default.json"), default_profile).unwrap();
        fs::write(temp_dir.join("kubectl-profile.json"), default_profile).unwrap();
        fs::write(temp_dir.join("docker-profile.json"), default_profile).unwrap();
        fs::write(temp_dir.join("readonly-profile.json"), default_profile).unwrap();

        temp_dir
    }

    #[test]
    fn test_new_seccomp_manager() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();
        assert_eq!(manager.profiles_dir(), profiles_dir.as_path());
    }

    #[test]
    fn test_profile_for_tool_kubectl() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();

        let profile = manager.profile_for_tool("kubectl");
        assert_eq!(profile.name, "kubectl");
        assert!(profile.path.ends_with("kubectl-profile.json"));
    }

    #[test]
    fn test_profile_for_tool_docker() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();

        let profile = manager.profile_for_tool("docker");
        assert_eq!(profile.name, "docker");
        assert!(profile.path.ends_with("docker-profile.json"));
    }

    #[test]
    fn test_profile_for_tool_readonly() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();

        let profile = manager.profile_for_tool("cat");
        assert_eq!(profile.name, "readonly");
        assert!(profile.path.ends_with("readonly-profile.json"));
    }

    #[test]
    fn test_profile_for_tool_default() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();

        let profile = manager.profile_for_tool("unknown-tool");
        assert_eq!(profile.name, "default");
        assert!(profile.path.ends_with("default.json"));
    }

    #[test]
    fn test_docker_security_opt() {
        let profiles_dir = setup_test_profiles();
        let manager = SeccompProfileManager::new(&profiles_dir).unwrap();

        let opt = manager.docker_security_opt("kubectl");
        assert!(opt.starts_with("seccomp="));
        assert!(opt.contains("kubectl-profile.json"));
    }

    #[test]
    fn test_missing_profiles_dir() {
        let result = SeccompProfileManager::new("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_default_profile() {
        let temp_dir = std::env::temp_dir().join("aof-test-seccomp-no-default");
        fs::create_dir_all(&temp_dir).unwrap();

        let result = SeccompProfileManager::new(&temp_dir);
        assert!(result.is_err());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
