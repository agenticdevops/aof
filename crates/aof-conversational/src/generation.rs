//! YAML generation and parsing utilities for agent creation
//!
//! Handles conversion between natural language descriptions and structured
//! AGENTS.md/SOUL.md content via LLM generation and validation.

use aof_personas::types::{Agent, Soul, SoulFrontmatter};
use anyhow::Result;
use serde_path_to_error;

/// Errors that can occur during generation
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),

    #[error("Failed to parse YAML: {0}")]
    YamlParseFailed(String),

    #[error("Failed to format YAML: {0}")]
    YamlFormatFailed(String),

    #[error("Invalid markdown structure: {0}")]
    InvalidMarkdown(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Generate AGENTS.md YAML entry from Claude's response
///
/// Handles both raw YAML and code-fenced YAML (strips markdown fences).
/// Uses serde_path_to_error for precise error messages on parse failures.
pub fn parse_agent_yaml(raw_yaml: &str) -> Result<Agent, GenerationError> {
    // Strip markdown code fences if present
    let yaml = strip_code_fences(raw_yaml);

    // Parse with serde_path_to_error for precise error reporting
    let deserializer = serde_yaml::Deserializer::from_str(&yaml);
    serde_path_to_error::deserialize(deserializer)
        .map_err(|e| GenerationError::YamlParseFailed(format!("Field: {}\nError: {}", e.path(), e.inner())))
}

/// Generate SOUL.md content from Claude's response
///
/// Expects markdown with YAML frontmatter followed by prose sections.
pub fn parse_soul_markdown(raw_md: &str, expected_agent_id: &str) -> Result<Soul, GenerationError> {
    let (frontmatter_str, prose) = extract_frontmatter(raw_md)?;

    // Parse frontmatter as SoulFrontmatter first
    let deserializer = serde_yaml::Deserializer::from_str(&frontmatter_str);
    let frontmatter: SoulFrontmatter = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| GenerationError::YamlParseFailed(format!("Field: {}\nError: {}", e.path(), e.inner())))?;

    // Verify agent ID matches
    if frontmatter.id != expected_agent_id {
        return Err(GenerationError::InvalidMarkdown(
            format!("SOUL.md agent ID '{}' doesn't match expected '{}'", frontmatter.id, expected_agent_id)
        ));
    }

    // Convert to Soul and add communication guide prose
    let mut soul: Soul = frontmatter.into();
    soul.communication_guide = prose.trim().to_string();

    Ok(soul)
}

/// Format Agent as YAML string ready for AGENTS.md insertion
pub fn format_agent_yaml(agent: &Agent) -> Result<String, GenerationError> {
    serde_yaml::to_string(agent)
        .map_err(|e| GenerationError::YamlFormatFailed(e.to_string()))
}

/// Format Soul as Markdown string ready for SOUL.md insertion
pub fn format_soul_markdown(soul: &Soul) -> Result<String, GenerationError> {
    let mut output = String::new();

    // Write YAML frontmatter
    output.push_str("```yaml\n");
    output.push_str(&format!("id: {}\n", soul.id));
    output.push_str(&format!("communication_style: {}\n", soul.communication_style));
    output.push_str(&format!("tone: {}\n", soul.tone));

    output.push_str("values:\n");
    for value in &soul.values {
        output.push_str(&format!("  - {}\n", value));
    }

    output.push_str(&format!("personality_summary: {}\n", soul.personality_summary));

    output.push_str("boundaries:\n");
    for boundary in &soul.boundaries {
        output.push_str(&format!("  - {}\n", boundary));
    }

    output.push_str(&format!("default_intro: {}\n", soul.default_intro));
    output.push_str("```\n\n");

    // Write communication guide prose
    output.push_str("## Communication Style\n\n");
    output.push_str(&soul.communication_guide);
    output.push('\n');

    Ok(output)
}

/// Build the agent generation prompt for Claude
///
/// Includes available skills list to prevent hallucination.
pub fn build_agent_generation_prompt(
    agent_type: &str,
    skills: &[String],
    description: &str,
    available_skills: &[String],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("Generate an AGENTS.md YAML entry for the following agent.\n\n");
    prompt.push_str("Requirements:\n");
    prompt.push_str("1. Output ONLY valid YAML (no markdown fences, no explanation)\n");
    prompt.push_str("2. Use lowercase-hyphenated format for 'id' (e.g., 'k8s-monitor')\n");
    prompt.push_str("3. Include fields: id, name, role, avatar, personality_traits, can, cannot, skills\n");
    prompt.push_str("4. avatar must be a single emoji character\n");
    prompt.push_str("5. personality_traits should have 3-5 adjectives\n");
    prompt.push_str("6. can should list 3-4 capabilities\n");
    prompt.push_str("7. cannot should list 2-3 boundaries\n");
    prompt.push_str("8. skills must ONLY use skills from this available list:\n\n");

    for skill in available_skills {
        prompt.push_str(&format!("   - {}\n", skill));
    }

    prompt.push_str("\nAgent description:\n");
    prompt.push_str(&format!("Type: {}\n", agent_type));

    if !skills.is_empty() {
        prompt.push_str(&format!("Requested skills: {}\n", skills.join(", ")));
    }

    prompt.push_str(&format!("Description: {}\n", description));
    prompt.push_str("\nGenerate the YAML now:\n");

    prompt
}

/// Build the personality generation prompt for Claude
pub fn build_soul_generation_prompt(
    agent_id: &str,
    agent_role: &str,
    personality_traits: &[String],
    description: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("Generate a SOUL.md personality section for the following agent.\n\n");
    prompt.push_str("Output format:\n");
    prompt.push_str("1. YAML frontmatter block with fields: id, communication_style, tone, values, personality_summary, boundaries, default_intro\n");
    prompt.push_str("2. Followed by a '## Communication Style' markdown section with 2-3 paragraphs of prose\n\n");

    prompt.push_str("Agent details:\n");
    prompt.push_str(&format!("ID: {}\n", agent_id));
    prompt.push_str(&format!("Role: {}\n", agent_role));
    prompt.push_str(&format!("Personality traits: {}\n", personality_traits.join(", ")));
    prompt.push_str(&format!("Description: {}\n", description));

    prompt.push_str("\nGenerate the SOUL.md content now:\n");

    prompt
}

// Internal helper: Strip markdown code fences from YAML
fn strip_code_fences(input: &str) -> String {
    let trimmed = input.trim();

    // Check for code fence patterns: ```yaml, ```, or just the content
    if trimmed.starts_with("```yaml") {
        // Remove opening ```yaml and closing ```
        let without_opening = trimmed.strip_prefix("```yaml").unwrap().trim_start();
        without_opening.strip_suffix("```").unwrap_or(without_opening).trim().to_string()
    } else if trimmed.starts_with("```") {
        // Remove opening ``` and closing ```
        let without_opening = trimmed.strip_prefix("```").unwrap().trim_start();
        without_opening.strip_suffix("```").unwrap_or(without_opening).trim().to_string()
    } else {
        trimmed.to_string()
    }
}

// Internal helper: Extract YAML frontmatter and prose from markdown
fn extract_frontmatter(markdown: &str) -> Result<(String, String), GenerationError> {
    let trimmed = markdown.trim();

    // Look for ```yaml block
    if !trimmed.starts_with("```yaml") && !trimmed.starts_with("```") {
        return Err(GenerationError::InvalidMarkdown(
            "SOUL.md must start with YAML frontmatter (```yaml block)".to_string()
        ));
    }

    // Find the closing ```
    let start_marker = if trimmed.starts_with("```yaml") { 7 } else { 3 };
    let after_opening = &trimmed[start_marker..];

    if let Some(end_pos) = after_opening.find("```") {
        let frontmatter = after_opening[..end_pos].trim();
        let prose = after_opening[end_pos + 3..].trim();
        Ok((frontmatter.to_string(), prose.to_string()))
    } else {
        Err(GenerationError::InvalidMarkdown(
            "YAML frontmatter not properly closed with ```".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_agent_yaml() {
        let yaml = r#"
id: k8s-monitor
name: Kubernetes Monitor
role: Infrastructure Specialist
avatar: 🔍
personality_traits: [methodical, detail-oriented, proactive]
can: [monitor clusters, analyze metrics, detect anomalies]
cannot: [modify production, delete resources]
skills: [k8s-diagnostics, metrics-analysis]
"#;
        let agent = parse_agent_yaml(yaml).unwrap();
        assert_eq!(agent.id, "k8s-monitor");
        assert_eq!(agent.name, "Kubernetes Monitor");
        assert_eq!(agent.personality_traits.len(), 3);
        assert_eq!(agent.skills.len(), 2);
    }

    #[test]
    fn test_parse_yaml_with_code_fences() {
        let yaml = r#"```yaml
id: test-agent
name: Test Agent
role: Tester
avatar: 🧪
personality_traits: [curious]
can: [test things]
cannot: [break things]
skills: [testing]
```"#;
        let agent = parse_agent_yaml(yaml).unwrap();
        assert_eq!(agent.id, "test-agent");
    }

    #[test]
    fn test_format_agent_roundtrip() {
        let agent = Agent {
            id: "test".to_string(),
            name: "Test".to_string(),
            role: "Tester".to_string(),
            avatar: "🧪".to_string(),
            personality_traits: vec!["curious".to_string()],
            can: vec!["test".to_string()],
            cannot: vec!["break".to_string()],
            skills: vec!["testing".to_string()],
        };

        let yaml = format_agent_yaml(&agent).unwrap();
        let parsed = parse_agent_yaml(&yaml).unwrap();

        assert_eq!(parsed.id, agent.id);
        assert_eq!(parsed.name, agent.name);
        assert_eq!(parsed.skills, agent.skills);
    }

    #[test]
    fn test_build_prompt_includes_available_skills() {
        let available = vec!["skill1".to_string(), "skill2".to_string(), "skill3".to_string()];
        let prompt = build_agent_generation_prompt(
            "monitor",
            &["skill1".to_string()],
            "A test agent",
            &available
        );

        assert!(prompt.contains("skill1"));
        assert!(prompt.contains("skill2"));
        assert!(prompt.contains("skill3"));
        assert!(prompt.contains("available list"));
    }

    #[test]
    fn test_build_soul_generation_prompt() {
        let prompt = build_soul_generation_prompt(
            "test-agent",
            "Tester",
            &["curious".to_string(), "methodical".to_string()],
            "A testing agent"
        );

        assert!(prompt.contains("test-agent"));
        assert!(prompt.contains("Tester"));
        assert!(prompt.contains("curious"));
        assert!(prompt.contains("Communication Style"));
    }

    #[test]
    fn test_strip_code_fences() {
        assert_eq!(strip_code_fences("```yaml\ntest\n```"), "test");
        assert_eq!(strip_code_fences("```\ntest\n```"), "test");
        assert_eq!(strip_code_fences("test"), "test");
        assert_eq!(strip_code_fences("  ```yaml\n  test\n  ```  "), "test");
    }

    #[test]
    fn test_extract_frontmatter() {
        let md = "```yaml\nid: test\n```\n\n## Communication Style\n\nSome prose here.";
        let (frontmatter, prose) = extract_frontmatter(md).unwrap();

        assert_eq!(frontmatter, "id: test");
        assert!(prose.contains("Communication Style"));
        assert!(prose.contains("Some prose here"));
    }

    #[test]
    fn test_parse_soul_markdown() {
        let md = r#"```yaml
id: test-agent
communication_style: formal-technical
tone: calm-professional
values: [reliability, accuracy]
personality_summary: A methodical testing agent
boundaries: [Never skip tests, Always verify]
default_intro: Hello, I am your testing agent
```

## Communication Style

I communicate with precision and clarity, focusing on factual observations
and systematic analysis. My tone remains calm and professional even when
reporting critical issues.
"#;
        let soul = parse_soul_markdown(md, "test-agent").unwrap();
        assert_eq!(soul.id, "test-agent");
        assert_eq!(soul.communication_style, "formal-technical");
        assert_eq!(soul.values.len(), 2);
        assert!(soul.communication_guide.contains("precision"));
    }

    #[test]
    fn test_format_soul_markdown() {
        let soul = Soul {
            id: "test".to_string(),
            communication_style: "casual".to_string(),
            tone: "friendly".to_string(),
            values: vec!["honesty".to_string()],
            personality_summary: "A friendly agent".to_string(),
            boundaries: vec!["Be helpful".to_string()],
            default_intro: "Hi there!".to_string(),
            communication_guide: "I speak in a friendly manner.".to_string(),
        };

        let md = format_soul_markdown(&soul).unwrap();
        assert!(md.contains("```yaml"));
        assert!(md.contains("id: test"));
        assert!(md.contains("## Communication Style"));
        assert!(md.contains("friendly manner"));
    }
}
