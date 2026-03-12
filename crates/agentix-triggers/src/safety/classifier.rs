//! Tool Classification - Categorize commands by their action class
//!
//! Classifies tool invocations into action classes:
//! - read: Information retrieval only
//! - write: Creates or modifies resources
//! - delete: Removes or destroys resources
//! - dangerous: Potentially harmful operations

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Action class for a tool invocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionClass {
    /// Read-only operations - information retrieval
    Read,
    /// Write operations - create or modify resources
    Write,
    /// Delete operations - remove or destroy resources
    Delete,
    /// Dangerous operations - potentially harmful
    Dangerous,
}

impl ActionClass {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Read => "read-only operation",
            Self::Write => "write operation",
            Self::Delete => "delete operation",
            Self::Dangerous => "dangerous operation",
        }
    }

    /// Get risk level (0 = safe, 3 = most risky)
    pub fn risk_level(&self) -> u8 {
        match self {
            Self::Read => 0,
            Self::Write => 1,
            Self::Delete => 2,
            Self::Dangerous => 3,
        }
    }
}

impl std::fmt::Display for ActionClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::Dangerous => write!(f, "dangerous"),
        }
    }
}

impl std::str::FromStr for ActionClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "delete" => Ok(Self::Delete),
            "dangerous" => Ok(Self::Dangerous),
            _ => Err(format!("Unknown action class: {}", s)),
        }
    }
}

/// Result of classifying a command
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// The determined action class
    pub class: ActionClass,
    /// The tool name (e.g., "kubectl", "docker")
    pub tool: String,
    /// The specific verb or subcommand matched
    pub verb: Option<String>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Source of classification (tool-specific, pattern, default)
    pub source: ClassificationSource,
}

/// Source of the classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationSource {
    /// Matched a tool-specific verb
    ToolSpecific,
    /// Matched a generic pattern
    GenericPattern,
    /// Default (fail secure)
    Default,
}

/// Tool-specific classification rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRules {
    /// Read verbs for this tool
    #[serde(default)]
    pub read: Vec<String>,
    /// Write verbs for this tool
    #[serde(default)]
    pub write: Vec<String>,
    /// Delete verbs for this tool
    #[serde(default)]
    pub delete: Vec<String>,
    /// Dangerous verbs for this tool
    #[serde(default)]
    pub dangerous: Vec<String>,
}

/// Generic pattern-based classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericPatterns {
    /// Patterns for read operations
    #[serde(default)]
    pub read: Vec<String>,
    /// Patterns for write operations
    #[serde(default)]
    pub write: Vec<String>,
    /// Patterns for delete operations
    #[serde(default)]
    pub delete: Vec<String>,
    /// Patterns for dangerous operations
    #[serde(default)]
    pub dangerous: Vec<String>,
}

/// Complete tool classifications configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolClassifications {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// Kind
    pub kind: String,
    /// Metadata
    pub metadata: ClassificationMetadata,
    /// Spec containing tool rules and patterns
    pub spec: ClassificationSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetadata {
    pub name: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSpec {
    /// Tool-specific classification rules
    #[serde(default)]
    pub tools: HashMap<String, ToolRules>,
    /// Generic pattern-based rules
    #[serde(default)]
    pub generic_patterns: GenericPatterns,
}

impl Default for GenericPatterns {
    fn default() -> Self {
        Self {
            read: vec![
                r"^(get|list|show|describe|status|info|inspect|cat|head|tail|watch|ls|ps|version|check|validate)\b".to_string(),
            ],
            write: vec![
                r"^(create|apply|patch|update|set|add|push|install|upgrade|run|start|exec|deploy|scale|restart|enable|configure)\b".to_string(),
            ],
            delete: vec![
                r"^(delete|destroy|remove|rm|uninstall|terminate|kill|stop|drop|purge|prune|disable)\b".to_string(),
            ],
            dangerous: vec![
                r"\b(--force|-f)\b".to_string(),
                r"\bsudo\b".to_string(),
                r"\brm\s+-rf\b".to_string(),
            ],
        }
    }
}

/// Tool classifier that determines action class for commands
pub struct ToolClassifier {
    /// Tool-specific rules
    tool_rules: HashMap<String, ToolRules>,
    /// Compiled regex patterns for generic matching
    read_patterns: Vec<Regex>,
    write_patterns: Vec<Regex>,
    delete_patterns: Vec<Regex>,
    dangerous_patterns: Vec<Regex>,
}

impl ToolClassifier {
    /// Create a new classifier with default patterns
    pub fn new() -> Self {
        Self::with_patterns(GenericPatterns::default())
    }

    /// Create classifier with custom patterns
    pub fn with_patterns(patterns: GenericPatterns) -> Self {
        Self {
            tool_rules: HashMap::new(),
            read_patterns: Self::compile_patterns(&patterns.read),
            write_patterns: Self::compile_patterns(&patterns.write),
            delete_patterns: Self::compile_patterns(&patterns.delete),
            dangerous_patterns: Self::compile_patterns(&patterns.dangerous),
        }
    }

    /// Load from ToolClassifications config
    pub fn from_config(config: ToolClassifications) -> Self {
        let mut classifier = Self::with_patterns(config.spec.generic_patterns);
        classifier.tool_rules = config.spec.tools;
        classifier
    }

    /// Add tool-specific rules
    pub fn add_tool_rules(&mut self, tool: &str, rules: ToolRules) {
        self.tool_rules.insert(tool.to_string(), rules);
    }

    /// Classify a command
    ///
    /// Command format: "tool subcommand args..."
    /// Examples:
    /// - "kubectl get pods"
    /// - "docker rm container-id"
    /// - "helm install my-chart"
    pub fn classify(&self, command: &str) -> ClassificationResult {
        let command = command.trim();
        if command.is_empty() {
            return ClassificationResult {
                class: ActionClass::Read,
                tool: String::new(),
                verb: None,
                confidence: 1.0,
                source: ClassificationSource::Default,
            };
        }

        // Split command into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        let tool = parts.first().map(|s| *s).unwrap_or("");
        let rest = parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();

        // Check dangerous patterns first (highest priority)
        if self.matches_any(&self.dangerous_patterns, command) {
            return ClassificationResult {
                class: ActionClass::Dangerous,
                tool: tool.to_string(),
                verb: Some(rest.clone()),
                confidence: 0.95,
                source: ClassificationSource::GenericPattern,
            };
        }

        // Check tool-specific rules
        if let Some(rules) = self.tool_rules.get(tool) {
            if let Some(result) = self.check_tool_rules(tool, &rest, rules) {
                return result;
            }
        }

        // Check generic patterns
        if let Some(class) = self.classify_by_patterns(&rest) {
            return ClassificationResult {
                class,
                tool: tool.to_string(),
                verb: Some(rest.clone()),
                confidence: 0.7,
                source: ClassificationSource::GenericPattern,
            };
        }

        // Default: assume write (fail secure)
        ClassificationResult {
            class: ActionClass::Write,
            tool: tool.to_string(),
            verb: Some(rest),
            confidence: 0.3,
            source: ClassificationSource::Default,
        }
    }

    /// Check if command matches tool-specific rules
    fn check_tool_rules(&self, tool: &str, rest: &str, rules: &ToolRules) -> Option<ClassificationResult> {
        // Check dangerous first
        for verb in &rules.dangerous {
            if rest.starts_with(verb) || rest.contains(verb) {
                return Some(ClassificationResult {
                    class: ActionClass::Dangerous,
                    tool: tool.to_string(),
                    verb: Some(verb.clone()),
                    confidence: 0.95,
                    source: ClassificationSource::ToolSpecific,
                });
            }
        }

        // Check delete
        for verb in &rules.delete {
            if rest.starts_with(verb) {
                return Some(ClassificationResult {
                    class: ActionClass::Delete,
                    tool: tool.to_string(),
                    verb: Some(verb.clone()),
                    confidence: 0.9,
                    source: ClassificationSource::ToolSpecific,
                });
            }
        }

        // Check write
        for verb in &rules.write {
            if rest.starts_with(verb) {
                return Some(ClassificationResult {
                    class: ActionClass::Write,
                    tool: tool.to_string(),
                    verb: Some(verb.clone()),
                    confidence: 0.9,
                    source: ClassificationSource::ToolSpecific,
                });
            }
        }

        // Check read
        for verb in &rules.read {
            if rest.starts_with(verb) {
                return Some(ClassificationResult {
                    class: ActionClass::Read,
                    tool: tool.to_string(),
                    verb: Some(verb.clone()),
                    confidence: 0.9,
                    source: ClassificationSource::ToolSpecific,
                });
            }
        }

        None
    }

    /// Classify command using generic patterns
    fn classify_by_patterns(&self, command: &str) -> Option<ActionClass> {
        // Check delete first (higher priority than write)
        if self.matches_any(&self.delete_patterns, command) {
            return Some(ActionClass::Delete);
        }

        // Check write
        if self.matches_any(&self.write_patterns, command) {
            return Some(ActionClass::Write);
        }

        // Check read
        if self.matches_any(&self.read_patterns, command) {
            return Some(ActionClass::Read);
        }

        None
    }

    /// Compile regex patterns, skipping invalid ones
    fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
        patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    }

    /// Check if command matches any pattern
    fn matches_any(&self, patterns: &[Regex], command: &str) -> bool {
        patterns.iter().any(|p| p.is_match(command))
    }
}

impl Default for ToolClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_kubectl_rules() -> ToolRules {
        ToolRules {
            read: vec!["get".into(), "list".into(), "describe".into(), "logs".into()],
            write: vec!["apply".into(), "create".into(), "scale".into()],
            delete: vec!["delete".into()],
            dangerous: vec!["exec".into(), "port-forward".into()],
        }
    }

    #[test]
    fn test_classify_kubectl_get() {
        let mut classifier = ToolClassifier::new();
        classifier.add_tool_rules("kubectl", create_kubectl_rules());

        let result = classifier.classify("kubectl get pods");
        assert_eq!(result.class, ActionClass::Read);
        assert_eq!(result.tool, "kubectl");
        assert_eq!(result.source, ClassificationSource::ToolSpecific);
    }

    #[test]
    fn test_classify_kubectl_delete() {
        let mut classifier = ToolClassifier::new();
        classifier.add_tool_rules("kubectl", create_kubectl_rules());

        let result = classifier.classify("kubectl delete pod my-pod");
        assert_eq!(result.class, ActionClass::Delete);
        assert_eq!(result.tool, "kubectl");
    }

    #[test]
    fn test_classify_kubectl_exec() {
        let mut classifier = ToolClassifier::new();
        classifier.add_tool_rules("kubectl", create_kubectl_rules());

        let result = classifier.classify("kubectl exec -it my-pod -- bash");
        assert_eq!(result.class, ActionClass::Dangerous);
    }

    #[test]
    fn test_classify_generic_rm() {
        let classifier = ToolClassifier::new();
        let result = classifier.classify("rm -rf /tmp/test");
        assert_eq!(result.class, ActionClass::Dangerous);
    }

    #[test]
    fn test_classify_generic_list() {
        let classifier = ToolClassifier::new();
        let result = classifier.classify("some-tool list items");
        assert_eq!(result.class, ActionClass::Read);
    }

    #[test]
    fn test_classify_unknown_defaults_to_write() {
        let classifier = ToolClassifier::new();
        let result = classifier.classify("unknown-tool unknown-command");
        assert_eq!(result.class, ActionClass::Write);
        assert_eq!(result.source, ClassificationSource::Default);
    }

    #[test]
    fn test_action_class_risk_levels() {
        assert_eq!(ActionClass::Read.risk_level(), 0);
        assert_eq!(ActionClass::Write.risk_level(), 1);
        assert_eq!(ActionClass::Delete.risk_level(), 2);
        assert_eq!(ActionClass::Dangerous.risk_level(), 3);
    }
}
