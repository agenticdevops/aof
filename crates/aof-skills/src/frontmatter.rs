//! YAML frontmatter parsing for SKILL.md files.
//!
//! Frontmatter is delimited by `---` markers at the start of the file:
//!
//! ```markdown
//! ---
//! name: k8s-debug
//! description: "Kubernetes pod debugging and troubleshooting"
//! metadata:
//!   emoji: "🐳"
//!   requires:
//!     bins: ["kubectl"]
//! ---
//!
//! # Kubernetes Debug Skill
//!
//! Instructions here...
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::error::SkillError;
use crate::types::SkillMetadata;

/// Regex to extract frontmatter between --- delimiters
static FRONTMATTER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---\r?\n(.*)$").expect("Invalid frontmatter regex")
});

/// Parsed frontmatter from a SKILL.md file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    /// Skill name (required)
    pub name: String,

    /// Description (required)
    pub description: String,

    /// Optional homepage URL
    pub homepage: Option<String>,

    /// Skill metadata
    #[serde(default)]
    pub metadata: SkillMetadata,
}

/// Result of parsing a SKILL.md file
#[derive(Debug, Clone)]
pub struct ParsedSkill {
    /// Parsed frontmatter
    pub frontmatter: SkillFrontmatter,

    /// Markdown content after frontmatter
    pub content: String,
}

/// Parse frontmatter and content from a SKILL.md file
///
/// # Arguments
/// * `text` - The full text content of the SKILL.md file
///
/// # Returns
/// * `Ok(ParsedSkill)` - Parsed frontmatter and content
/// * `Err(SkillError)` - If parsing fails
pub fn parse_frontmatter(text: &str) -> Result<ParsedSkill, SkillError> {
    let captures = FRONTMATTER_REGEX
        .captures(text)
        .ok_or_else(|| SkillError::frontmatter_error("<unknown>", "No frontmatter found"))?;

    let yaml_content = captures.get(1).map(|m| m.as_str()).unwrap_or("");
    let markdown_content = captures.get(2).map(|m| m.as_str()).unwrap_or("");

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_content)
        .map_err(|e| SkillError::frontmatter_error("<unknown>", format!("YAML parse error: {}", e)))?;

    Ok(ParsedSkill {
        frontmatter,
        content: markdown_content.to_string(),
    })
}

/// Check if text has valid frontmatter delimiters
pub fn has_frontmatter(text: &str) -> bool {
    FRONTMATTER_REGEX.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let text = r#"---
name: test-skill
description: "A test skill"
---

# Test Skill

Some content here.
"#;

        let result = parse_frontmatter(text).unwrap();
        assert_eq!(result.frontmatter.name, "test-skill");
        assert_eq!(result.frontmatter.description, "A test skill");
        assert!(result.content.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_frontmatter_with_metadata() {
        let text = r#"---
name: k8s-debug
description: "Kubernetes debugging"
homepage: "https://example.com"
metadata:
  emoji: "🐳"
  requires:
    bins:
      - kubectl
      - jq
    env:
      - KUBECONFIG
  tags:
    - kubernetes
    - debugging
---

# K8s Debug

Content...
"#;

        let result = parse_frontmatter(text).unwrap();
        assert_eq!(result.frontmatter.name, "k8s-debug");
        assert_eq!(result.frontmatter.metadata.emoji, Some("🐳".to_string()));
        assert_eq!(result.frontmatter.metadata.requires.bins, vec!["kubectl", "jq"]);
        assert_eq!(result.frontmatter.metadata.requires.env, vec!["KUBECONFIG"]);
        assert_eq!(result.frontmatter.metadata.tags, vec!["kubernetes", "debugging"]);
    }

    #[test]
    fn test_parse_frontmatter_no_delimiters() {
        let text = "# Just markdown\n\nNo frontmatter here.";
        assert!(parse_frontmatter(text).is_err());
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\nname: test\n---\ncontent"));
        assert!(!has_frontmatter("# Just markdown"));
        assert!(!has_frontmatter("---\nincomplete"));
    }

    #[test]
    fn test_parse_frontmatter_with_install_specs() {
        let text = r#"---
name: postgres-ops
description: "PostgreSQL operations"
metadata:
  requires:
    bins:
      - pg_dump
      - psql
  install:
    - id: brew
      kind: brew
      package: postgresql
      bins:
        - pg_dump
        - psql
---

# PostgreSQL Ops
"#;

        let result = parse_frontmatter(text).unwrap();
        assert_eq!(result.frontmatter.metadata.install.len(), 1);
        assert_eq!(result.frontmatter.metadata.install[0].id, "brew");
        assert_eq!(result.frontmatter.metadata.install[0].package, "postgresql");
    }
}
