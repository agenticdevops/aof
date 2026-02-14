//! Validation for generated agent content
//!
//! Validates generated AGENTS.md and SOUL.md content before persisting to files.
//! Catches duplicates, hallucinated skills, invalid formats, and conflicting capabilities.

use aof_personas::types::Agent;
use regex::Regex;
use std::sync::OnceLock;

/// Errors that can occur during validation
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),

    #[error("Duplicate agent ID: {0}")]
    DuplicateAgentId(String),

    #[error("Skill not found: {skill}. Similar: {suggestions:?}")]
    SkillNotFound {
        skill: String,
        suggestions: Vec<String>,
    },

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid emoji avatar: {0}")]
    InvalidAvatar(String),

    #[error("Conflicting capabilities: {0}")]
    ConflictingCapabilities(String),

    #[error("Token limit exceeded: {estimate} > {limit}")]
    TokenLimitExceeded { estimate: usize, limit: usize },

    #[error("Invalid agent ID format: {0}. Expected lowercase-hyphenated (e.g., 'k8s-monitor')")]
    InvalidAgentId(String),
}

/// Validate a generated agent against existing agents and available skills
///
/// Returns all validation errors found (not just the first one).
pub fn validate_generated_agent(
    agent: &Agent,
    existing_agents: &[Agent],
    available_skills: &[String],
) -> Result<(), Vec<GenerationError>> {
    let mut errors = Vec::new();

    // 1. Agent ID is non-empty, lowercase-hyphenated
    if agent.id.is_empty() {
        errors.push(GenerationError::MissingField("id".to_string()));
    } else if !is_valid_agent_id(&agent.id) {
        errors.push(GenerationError::InvalidAgentId(agent.id.clone()));
    }

    // 2. Agent ID not already in existing_agents
    if existing_agents.iter().any(|a| a.id == agent.id) {
        errors.push(GenerationError::DuplicateAgentId(agent.id.clone()));
    }

    // 3. Name, role are non-empty
    if agent.name.is_empty() {
        errors.push(GenerationError::MissingField("name".to_string()));
    }
    if agent.role.is_empty() {
        errors.push(GenerationError::MissingField("role".to_string()));
    }

    // 4. Avatar is single emoji
    if let Err(e) = validate_emoji_avatar(&agent.avatar) {
        errors.push(e);
    }

    // 5. All skills exist in available_skills list
    for skill in &agent.skills {
        if !available_skills.contains(skill) {
            let suggestions = find_similar_skills(skill, available_skills);
            errors.push(GenerationError::SkillNotFound {
                skill: skill.clone(),
                suggestions,
            });
        }
    }

    // 6. Can and cannot lists don't conflict
    if let Some(conflict) = find_capability_conflict(&agent.can, &agent.cannot) {
        errors.push(GenerationError::ConflictingCapabilities(conflict));
    }

    // 7. Personality traits non-empty
    if agent.personality_traits.is_empty() {
        errors.push(GenerationError::MissingField("personality_traits".to_string()));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check if agent ID follows lowercase-hyphenated format
fn is_valid_agent_id(id: &str) -> bool {
    static AGENT_ID_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = AGENT_ID_REGEX.get_or_init(|| {
        Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("Invalid agent ID regex")
    });
    regex.is_match(id)
}

/// Validate that avatar is a single emoji character
fn validate_emoji_avatar(avatar: &str) -> Result<(), GenerationError> {
    use unicode_segmentation::UnicodeSegmentation;

    // Count grapheme clusters (visual characters)
    let graphemes: Vec<&str> = avatar.graphemes(true).collect();

    if graphemes.is_empty() {
        return Err(GenerationError::MissingField("avatar".to_string()));
    }

    if graphemes.len() > 1 {
        return Err(GenerationError::InvalidAvatar(format!(
            "Avatar must be a single emoji, got {} characters: {}",
            graphemes.len(),
            avatar
        )));
    }

    // Check if it's actually an emoji (basic check - unicode emoji blocks)
    let first_char = avatar.chars().next().unwrap();
    if !is_emoji_char(first_char) {
        return Err(GenerationError::InvalidAvatar(format!(
            "Avatar must be an emoji, got: {}",
            avatar
        )));
    }

    Ok(())
}

/// Check if a character is in emoji Unicode blocks
fn is_emoji_char(c: char) -> bool {
    matches!(c,
        // Emoticons
        '\u{1F600}'..='\u{1F64F}' |
        // Miscellaneous Symbols and Pictographs
        '\u{1F300}'..='\u{1F5FF}' |
        // Transport and Map Symbols
        '\u{1F680}'..='\u{1F6FF}' |
        // Supplemental Symbols and Pictographs
        '\u{1F900}'..='\u{1F9FF}' |
        // Symbols and Pictographs Extended-A
        '\u{1FA70}'..='\u{1FAFF}' |
        // Dingbats
        '\u{2700}'..='\u{27BF}' |
        // Miscellaneous Symbols
        '\u{2600}'..='\u{26FF}' |
        // Variation Selectors
        '\u{FE00}'..='\u{FE0F}'
    )
}

/// Find similar skills using simple string similarity
pub fn find_similar_skills(skill: &str, available: &[String]) -> Vec<String> {
    let mut similarities: Vec<(String, usize)> = available
        .iter()
        .map(|s| {
            let similarity = calculate_similarity(skill, s);
            (s.clone(), similarity)
        })
        .collect();

    // Sort by similarity (higher is more similar)
    similarities.sort_by(|a, b| b.1.cmp(&a.1));

    // Return top 3 most similar
    similarities
        .into_iter()
        .take(3)
        .filter(|(_, score)| *score > 0)
        .map(|(s, _)| s)
        .collect()
}

/// Calculate simple similarity score (longest common substring length)
fn calculate_similarity(a: &str, b: &str) -> usize {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    // Check for substring match
    if b_lower.contains(&a_lower) || a_lower.contains(&b_lower) {
        return a_lower.len().min(b_lower.len());
    }

    // Count common prefix length
    let common_prefix = a_lower
        .chars()
        .zip(b_lower.chars())
        .take_while(|(ac, bc)| ac == bc)
        .count();

    common_prefix
}

/// Find conflicts between can and cannot capability lists
fn find_capability_conflict(can: &[String], cannot: &[String]) -> Option<String> {
    for can_item in can {
        for cannot_item in cannot {
            // Check if they're the same (case-insensitive)
            if can_item.to_lowercase() == cannot_item.to_lowercase() {
                return Some(format!(
                    "'{}' appears in both 'can' and 'cannot' lists",
                    can_item
                ));
            }

            // Check if one is a substring of the other (likely conflict)
            let can_lower = can_item.to_lowercase();
            let cannot_lower = cannot_item.to_lowercase();

            if can_lower.contains(&cannot_lower) || cannot_lower.contains(&can_lower) {
                return Some(format!(
                    "Potential conflict between '{}' (can) and '{}' (cannot)",
                    can_item, cannot_item
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent(id: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: "Test Agent".to_string(),
            role: "Tester".to_string(),
            avatar: "🧪".to_string(),
            personality_traits: vec!["curious".to_string()],
            can: vec!["test things".to_string()],
            cannot: vec!["break production".to_string()],
            skills: vec!["testing".to_string()],
        }
    }

    #[test]
    fn test_valid_agent_passes() {
        let agent = create_test_agent("test-agent");
        let existing: Vec<Agent> = vec![];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_id_caught() {
        let agent = create_test_agent("test-agent");
        let existing = vec![create_test_agent("test-agent")];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GenerationError::DuplicateAgentId(_))));
    }

    #[test]
    fn test_hallucinated_skill_caught_with_suggestions() {
        let mut agent = create_test_agent("test-agent");
        agent.skills = vec!["k8s-monitoring".to_string()]; // Typo

        let existing: Vec<Agent> = vec![];
        let skills = vec![
            "k8s-diagnostics".to_string(),
            "k8s-monitor".to_string(),
            "metrics-analysis".to_string(),
        ];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        let skill_error = errors
            .iter()
            .find(|e| matches!(e, GenerationError::SkillNotFound { .. }));

        assert!(skill_error.is_some());

        if let Some(GenerationError::SkillNotFound { skill, suggestions }) = skill_error {
            assert_eq!(skill, "k8s-monitoring");
            assert!(!suggestions.is_empty());
            // Should suggest k8s-monitor or k8s-diagnostics
            assert!(suggestions.iter().any(|s| s.starts_with("k8s-")));
        }
    }

    #[test]
    fn test_missing_fields_caught() {
        let mut agent = create_test_agent("test-agent");
        agent.name = String::new();
        agent.role = String::new();
        agent.personality_traits = vec![];

        let existing: Vec<Agent> = vec![];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() >= 3); // name, role, personality_traits
    }

    #[test]
    fn test_invalid_emoji_caught() {
        let mut agent = create_test_agent("test-agent");
        agent.avatar = "ABC".to_string(); // Not an emoji

        let existing: Vec<Agent> = vec![];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GenerationError::InvalidAvatar(_))));
    }

    #[test]
    fn test_conflicting_can_cannot_caught() {
        let mut agent = create_test_agent("test-agent");
        agent.can = vec!["deploy code".to_string()];
        agent.cannot = vec!["deploy to production".to_string()];

        let existing: Vec<Agent> = vec![];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GenerationError::ConflictingCapabilities(_))));
    }

    #[test]
    fn test_invalid_agent_id_format() {
        let mut agent = create_test_agent("TestAgent"); // Should be lowercase-hyphenated
        let existing: Vec<Agent> = vec![];
        let skills = vec!["testing".to_string()];

        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GenerationError::InvalidAgentId(_))));

        // Test various invalid formats
        agent.id = "test_agent".to_string(); // Underscore
        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        agent.id = "test agent".to_string(); // Space
        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());

        agent.id = "Test-Agent".to_string(); // Uppercase
        let result = validate_generated_agent(&agent, &existing, &skills);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_similar_skills() {
        let available = vec![
            "k8s-diagnostics".to_string(),
            "k8s-monitor".to_string(),
            "metrics-analysis".to_string(),
            "log-parser".to_string(),
        ];

        let suggestions = find_similar_skills("k8s-monitoring", &available);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.starts_with("k8s-")));

        let suggestions = find_similar_skills("logging", &available);
        assert!(suggestions.iter().any(|s| s.contains("log")));
    }

    #[test]
    fn test_valid_emoji_detection() {
        assert!(validate_emoji_avatar("🧪").is_ok());
        assert!(validate_emoji_avatar("🤖").is_ok());
        assert!(validate_emoji_avatar("🔍").is_ok());
        assert!(validate_emoji_avatar("⚡").is_ok());
    }

    #[test]
    fn test_multiple_emoji_rejected() {
        let result = validate_emoji_avatar("🧪🤖");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_agent_id_format() {
        assert!(is_valid_agent_id("k8s-monitor"));
        assert!(is_valid_agent_id("test-agent-123"));
        assert!(is_valid_agent_id("simple"));
        assert!(is_valid_agent_id("multi-word-id"));

        assert!(!is_valid_agent_id("Test-Agent")); // Uppercase
        assert!(!is_valid_agent_id("test_agent")); // Underscore
        assert!(!is_valid_agent_id("test agent")); // Space
        assert!(!is_valid_agent_id("test-")); // Trailing hyphen
        assert!(!is_valid_agent_id("-test")); // Leading hyphen
    }
}
