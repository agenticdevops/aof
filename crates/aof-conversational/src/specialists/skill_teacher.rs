use super::traits::{Specialist, SpecialistOutput};
use crate::types::{IntentClassification, ConversationSession};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Skill generation errors
#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Missing name in frontmatter")]
    MissingName,
    #[error("Missing description in frontmatter")]
    MissingDescription,
    #[error("No steps found (need at least 2 ## sections)")]
    TooFewSteps,
    #[error("No code examples found (need at least 1 code block)")]
    NoCodeExamples,
    #[error("No validation criteria found")]
    NoValidationCriteria,
}

/// Skill teaching specialist
pub struct SkillTeacher {
    skills_path: PathBuf,
}

impl SkillTeacher {
    pub fn new(skills_path: PathBuf) -> Self {
        Self { skills_path }
    }

    /// Check if a skill with this name already exists
    fn check_duplicate(&self, skill_name: &str) -> Result<Option<PathBuf>> {
        let skill_dir = self.skills_path.join(skill_name);
        let skill_file = skill_dir.join("SKILL.md");
        
        if skill_file.exists() {
            Ok(Some(skill_file))
        } else {
            Ok(None)
        }
    }

    /// Generate skill name from description
    fn skill_name_from_description(&self, description: &str) -> String {
        // Simple kebab-case conversion
        description
            .to_lowercase()
            .split_whitespace()
            .take(3) // Take first 3 words
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    /// Generate SKILL.md content (MVP: template-based, not Claude)
    fn generate_skill_content(&self, skill_name: &str, description: &str) -> String {
        format!(
            r#"---
name: {}
description: {}
metadata:
  emoji: 🎯
  version: "1.0.0"
  tags: []
  requires: []
---

# {}

{}

## Steps

### 1. Preparation

Prepare your environment and gather necessary information.

```bash
# Example command
echo "Placeholder - customize this step"
```

### 2. Execution

Execute the main task.

```bash
# Example execution
echo "Placeholder - implement actual skill steps"
```

### 3. Verification

Verify the results.

```bash
# Example verification
echo "Placeholder - add verification steps"
```

## Common Issues

- **Issue 1**: Description of common issue
  - **Solution**: How to resolve it

- **Issue 2**: Another common issue
  - **Solution**: Resolution steps

## Validation

To verify this skill works correctly:

1. Run the preparation steps
2. Execute the main task
3. Verify expected outcomes match actual results
4. Check for any error messages or warnings

## Notes

This is a generated skill template. Customize the steps, commands, and validation criteria based on your specific use case.
"#,
            skill_name, description, skill_name, description
        )
    }

    /// Validate generated SKILL.md content
    fn validate_skill_content(&self, content: &str) -> Result<(), Vec<SkillError>> {
        let mut errors = Vec::new();

        // Check for YAML frontmatter
        if !content.starts_with("---") {
            errors.push(SkillError::MissingName);
            errors.push(SkillError::MissingDescription);
        } else {
            // Basic frontmatter validation
            if !content.contains("name:") {
                errors.push(SkillError::MissingName);
            }
            if !content.contains("description:") {
                errors.push(SkillError::MissingDescription);
            }
        }

        // Count ## headers (steps)
        let header_count = content.matches("\n## ").count();
        if header_count < 2 {
            errors.push(SkillError::TooFewSteps);
        }

        // Check for code blocks
        if !content.contains("```") {
            errors.push(SkillError::NoCodeExamples);
        }

        // Check for validation section
        if !content.contains("## Validation") && !content.contains("## Verification") {
            errors.push(SkillError::NoValidationCriteria);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[async_trait]
impl Specialist for SkillTeacher {
    async fn handle(
        &self,
        intent: &IntentClassification,
        _session: &ConversationSession,
    ) -> Result<SpecialistOutput> {
        // Extract skill name or description
        let skill_name = intent
            .parameters
            .get("skill_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        let description = intent
            .parameters
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing skill description"))?;

        // Derive skill name if not provided
        let name = skill_name.unwrap_or_else(|| self.skill_name_from_description(description));

        // Check for duplicates
        if let Some(existing_path) = self.check_duplicate(&name)? {
            let message = format!(
                "Skill '{}' already exists at {:?}.\n\n\
                Would you like to:\n\
                1. Update the existing skill\n\
                2. Create a new variant with a different name",
                name, existing_path
            );
            return Ok(SpecialistOutput::new(HashMap::new(), message, false));
        }

        // Generate skill content
        let content = self.generate_skill_content(&name, description);

        // Validate
        if let Err(validation_errors) = self.validate_skill_content(&content) {
            let error_list = validation_errors
                .iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(anyhow!(
                "Generated skill content has validation errors:\n{}",
                error_list
            ));
        }

        // Create files map
        let mut files = HashMap::new();
        let skill_dir = format!("{}/{}", self.skills_path.display(), name);
        files.insert(format!("{}/SKILL.md", skill_dir), content);

        let message = format!(
            "Created skill: {}\n\n\
            Description: {}\n\n\
            Location: {}/SKILL.md\n\n\
            Review the generated SKILL.md and customize the steps, commands, and validation criteria.",
            name, description, skill_dir
        );

        Ok(SpecialistOutput::with_confirmation(files, message))
    }

    fn name(&self) -> &str {
        "skill_teacher"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_name_from_description() {
        let teacher = SkillTeacher::new(PathBuf::from("/tmp"));
        
        assert_eq!(
            teacher.skill_name_from_description("debug Postgres connections"),
            "debug-postgres-connections"
        );
        
        assert_eq!(
            teacher.skill_name_from_description("Monitor Kubernetes pods health"),
            "monitor-kubernetes-pods"
        );
    }

    #[test]
    fn test_validate_skill_content_valid() {
        let teacher = SkillTeacher::new(PathBuf::from("/tmp"));
        let content = teacher.generate_skill_content("test-skill", "Test description");
        
        assert!(teacher.validate_skill_content(&content).is_ok());
    }

    #[test]
    fn test_validate_skill_content_missing_frontmatter() {
        let teacher = SkillTeacher::new(PathBuf::from("/tmp"));
        let content = "# Skill\n\nNo frontmatter";
        
        let result = teacher.validate_skill_content(content);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SkillError::MissingName)));
    }

    #[test]
    fn test_validate_skill_content_no_code_blocks() {
        let teacher = SkillTeacher::new(PathBuf::from("/tmp"));
        let content = "---\nname: test\ndescription: test\n---\n## Step 1\nNo code\n## Validation\nCheck";
        
        let result = teacher.validate_skill_content(content);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SkillError::NoCodeExamples)));
    }
}
