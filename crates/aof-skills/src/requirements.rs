//! Requirements checking and gating for skills.
//!
//! Skills can specify requirements that must be met before they're eligible:
//! - Required binaries in PATH
//! - Required environment variables
//! - Required config file paths
//! - OS restrictions

use std::collections::HashMap;
use std::env;
use std::path::Path;

use crate::types::{Skill, SkillRequirements};

/// Context for checking skill eligibility
#[derive(Debug, Clone, Default)]
pub struct EligibilityContext {
    /// Current operating system
    pub os: String,

    /// Available binaries (cached from PATH lookup)
    pub available_bins: HashMap<String, bool>,

    /// Environment variables that are set
    pub env_vars: HashMap<String, bool>,

    /// Config paths that exist
    pub config_paths: HashMap<String, bool>,
}

impl EligibilityContext {
    /// Create a new context with current system state
    pub fn from_system() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            available_bins: HashMap::new(),
            env_vars: HashMap::new(),
            config_paths: HashMap::new(),
        }
    }

    /// Check if a binary is available in PATH
    pub fn has_binary(&mut self, name: &str) -> bool {
        if let Some(&cached) = self.available_bins.get(name) {
            return cached;
        }

        let available = which::which(name).is_ok();
        self.available_bins.insert(name.to_string(), available);
        available
    }

    /// Check if an environment variable is set
    pub fn has_env(&mut self, name: &str) -> bool {
        if let Some(&cached) = self.env_vars.get(name) {
            return cached;
        }

        let has_var = env::var(name).is_ok();
        self.env_vars.insert(name.to_string(), has_var);
        has_var
    }

    /// Check if a config path exists
    pub fn has_config(&mut self, path: &str) -> bool {
        if let Some(&cached) = self.config_paths.get(path) {
            return cached;
        }

        // Expand ~ to home directory
        let expanded = if path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                home.join(&path[2..])
            } else {
                Path::new(path).to_path_buf()
            }
        } else {
            Path::new(path).to_path_buf()
        };

        let exists = expanded.exists();
        self.config_paths.insert(path.to_string(), exists);
        exists
    }
}

/// Result of checking requirements
#[derive(Debug, Clone)]
pub struct RequirementCheck {
    /// Whether all requirements are met
    pub eligible: bool,

    /// Missing binaries
    pub missing_bins: Vec<String>,

    /// Missing "any_bins" (none of the alternatives available)
    pub missing_any_bins: Vec<String>,

    /// Missing environment variables
    pub missing_env: Vec<String>,

    /// Missing config paths
    pub missing_config: Vec<String>,

    /// OS mismatch (if restricted)
    pub os_mismatch: Option<String>,
}

impl RequirementCheck {
    /// Create a passing check
    pub fn passed() -> Self {
        Self {
            eligible: true,
            missing_bins: vec![],
            missing_any_bins: vec![],
            missing_env: vec![],
            missing_config: vec![],
            os_mismatch: None,
        }
    }

    /// Get a human-readable summary of what's missing
    pub fn summary(&self) -> String {
        if self.eligible {
            return "All requirements met".to_string();
        }

        let mut parts = vec![];

        if !self.missing_bins.is_empty() {
            parts.push(format!("Missing binaries: {}", self.missing_bins.join(", ")));
        }

        if !self.missing_any_bins.is_empty() {
            parts.push(format!(
                "Need one of: {}",
                self.missing_any_bins.join(", ")
            ));
        }

        if !self.missing_env.is_empty() {
            parts.push(format!("Missing env vars: {}", self.missing_env.join(", ")));
        }

        if !self.missing_config.is_empty() {
            parts.push(format!("Missing configs: {}", self.missing_config.join(", ")));
        }

        if let Some(ref os) = self.os_mismatch {
            parts.push(format!("OS mismatch: {}", os));
        }

        parts.join("; ")
    }
}

/// Checker for skill requirements
pub struct RequirementChecker {
    context: EligibilityContext,
}

impl RequirementChecker {
    /// Create a new checker with system context
    pub fn new() -> Self {
        Self {
            context: EligibilityContext::from_system(),
        }
    }

    /// Create a checker with custom context
    pub fn with_context(context: EligibilityContext) -> Self {
        Self { context }
    }

    /// Check if a skill's requirements are met
    pub fn check(&mut self, skill: &Skill) -> RequirementCheck {
        // Skills marked as "always" bypass requirements
        if skill.metadata.always {
            return RequirementCheck::passed();
        }

        let mut check = RequirementCheck {
            eligible: true,
            missing_bins: vec![],
            missing_any_bins: vec![],
            missing_env: vec![],
            missing_config: vec![],
            os_mismatch: None,
        };

        // Check OS restriction
        if let Some(ref allowed_os) = skill.metadata.os {
            if !allowed_os.contains(&self.context.os) {
                check.eligible = false;
                check.os_mismatch = Some(format!(
                    "Current OS '{}' not in allowed list: {:?}",
                    self.context.os, allowed_os
                ));
            }
        }

        // Check required binaries
        self.check_requirements(&skill.metadata.requires, &mut check);

        check
    }

    /// Check requirements and update the check result
    fn check_requirements(&mut self, reqs: &SkillRequirements, check: &mut RequirementCheck) {
        // All bins must be present
        for bin in &reqs.bins {
            if !self.context.has_binary(bin) {
                check.eligible = false;
                check.missing_bins.push(bin.clone());
            }
        }

        // At least one of any_bins must be present
        if !reqs.any_bins.is_empty() {
            let has_any = reqs.any_bins.iter().any(|b| self.context.has_binary(b));
            if !has_any {
                check.eligible = false;
                check.missing_any_bins = reqs.any_bins.clone();
            }
        }

        // All env vars must be set
        for var in &reqs.env {
            if !self.context.has_env(var) {
                check.eligible = false;
                check.missing_env.push(var.clone());
            }
        }

        // All config paths must exist
        for path in &reqs.config {
            if !self.context.has_config(path) {
                check.eligible = false;
                check.missing_config.push(path.clone());
            }
        }
    }

    /// Check multiple skills and return only eligible ones
    pub fn filter_eligible(&mut self, skills: Vec<Skill>) -> Vec<Skill> {
        skills
            .into_iter()
            .filter(|skill| self.check(skill).eligible)
            .collect()
    }
}

impl Default for RequirementChecker {
    fn default() -> Self {
        Self::new()
    }
}

// Helper module for home directory expansion
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillMetadata, SkillSource};

    fn make_skill(name: &str, reqs: SkillRequirements) -> Skill {
        Skill {
            name: name.to_string(),
            description: "Test skill".to_string(),
            homepage: None,
            content: "# Test".to_string(),
            metadata: SkillMetadata {
                requires: reqs,
                ..Default::default()
            },
            source: SkillSource::Bundled,
        }
    }

    #[test]
    fn test_empty_requirements_pass() {
        let skill = make_skill("test", SkillRequirements::default());
        let mut checker = RequirementChecker::new();
        let check = checker.check(&skill);
        assert!(check.eligible);
    }

    #[test]
    fn test_always_skill_bypasses_requirements() {
        let mut skill = make_skill(
            "always-skill",
            SkillRequirements {
                bins: vec!["nonexistent-binary-xyz".to_string()],
                ..Default::default()
            },
        );
        skill.metadata.always = true;

        let mut checker = RequirementChecker::new();
        let check = checker.check(&skill);
        assert!(check.eligible);
    }

    #[test]
    fn test_missing_binary() {
        let skill = make_skill(
            "test",
            SkillRequirements {
                bins: vec!["nonexistent-binary-xyz".to_string()],
                ..Default::default()
            },
        );

        let mut checker = RequirementChecker::new();
        let check = checker.check(&skill);
        assert!(!check.eligible);
        assert!(check.missing_bins.contains(&"nonexistent-binary-xyz".to_string()));
    }

    #[test]
    fn test_check_summary() {
        let check = RequirementCheck {
            eligible: false,
            missing_bins: vec!["kubectl".to_string()],
            missing_any_bins: vec![],
            missing_env: vec!["KUBECONFIG".to_string()],
            missing_config: vec![],
            os_mismatch: None,
        };

        let summary = check.summary();
        assert!(summary.contains("kubectl"));
        assert!(summary.contains("KUBECONFIG"));
    }
}
