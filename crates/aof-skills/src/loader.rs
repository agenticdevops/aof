//! Skill loading from filesystem directories.
//!
//! Skills are loaded from directories containing `SKILL.md` files.
//! The loader supports multiple source directories with precedence ordering.

use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::error::SkillError;
use crate::frontmatter::parse_frontmatter;
use crate::types::{Skill, SkillConfig, SkillSource};
use crate::Result;

/// Loads skills from filesystem directories
pub struct SkillLoader {
    config: SkillConfig,
}

impl SkillLoader {
    /// Create a new skill loader with configuration
    pub fn new(config: SkillConfig) -> Self {
        Self { config }
    }

    /// Create a loader with default configuration
    pub fn default_loader() -> Self {
        Self::new(SkillConfig::default())
    }

    /// Load all skills from configured sources
    ///
    /// Skills are loaded in precedence order:
    /// 1. Workspace (highest priority)
    /// 2. Enterprise registry
    /// 3. Public registry
    /// 4. Bundled (lowest priority)
    pub async fn load_all(&self) -> Result<Vec<Skill>> {
        let mut all_skills = Vec::new();

        // Load bundled skills first (lowest precedence)
        for dir in &self.config.bundled_dirs {
            if dir.exists() {
                let skills = self.load_from_directory(dir, SkillSource::Bundled).await?;
                all_skills.extend(skills);
            }
        }

        // Load workspace skills last (highest precedence)
        if let Some(ref workspace_dir) = self.config.workspace_dir {
            if workspace_dir.exists() {
                let skills = self
                    .load_from_directory(
                        workspace_dir,
                        SkillSource::Workspace {
                            path: workspace_dir.clone(),
                        },
                    )
                    .await?;
                all_skills.extend(skills);
            }
        }

        // Deduplicate by name, keeping highest precedence
        let deduped = Self::deduplicate_by_precedence(all_skills);

        info!("Loaded {} skills", deduped.len());
        Ok(deduped)
    }

    /// Load skills from a single directory
    pub async fn load_from_directory(
        &self,
        dir: &Path,
        source: SkillSource,
    ) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();

        // Find all SKILL.md files
        let pattern = dir.join("**/SKILL.md");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| SkillError::invalid_skill(dir, "Invalid path encoding"))?;

        for entry in glob::glob(pattern_str)? {
            match entry {
                Ok(path) => {
                    match self.load_skill_file(&path, source.clone()).await {
                        Ok(skill) => {
                            debug!("Loaded skill '{}' from {:?}", skill.name, path);
                            skills.push(skill);
                        }
                        Err(e) => {
                            warn!("Failed to load skill from {:?}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Glob error: {}", e);
                }
            }
        }

        Ok(skills)
    }

    /// Load a single skill from a SKILL.md file
    pub async fn load_skill_file(&self, path: &Path, source: SkillSource) -> Result<Skill> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| SkillError::read_error(path, e))?;

        let parsed = parse_frontmatter(&content).map_err(|e| match e {
            SkillError::FrontmatterError { message, .. } => {
                SkillError::frontmatter_error(path, message)
            }
            other => other,
        })?;

        Ok(Skill {
            name: parsed.frontmatter.name,
            description: parsed.frontmatter.description,
            homepage: parsed.frontmatter.homepage,
            content: parsed.content,
            metadata: parsed.frontmatter.metadata,
            source,
        })
    }

    /// Deduplicate skills by name, keeping the highest precedence source
    fn deduplicate_by_precedence(skills: Vec<Skill>) -> Vec<Skill> {
        use std::collections::HashMap;

        let mut by_name: HashMap<String, Skill> = HashMap::new();

        for skill in skills {
            let name = skill.name.clone();
            if let Some(existing) = by_name.get(&name) {
                // Keep the one with higher precedence
                if skill.source.precedence() > existing.source.precedence() {
                    by_name.insert(name, skill);
                }
            } else {
                by_name.insert(name, skill);
            }
        }

        let mut result: Vec<Skill> = by_name.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
}

/// Build a skill prompt section for model consumption
///
/// Formats skills as XML for injection into agent prompts
pub fn build_skills_prompt(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }

    let mut output = String::from("<available-skills>\n");

    for skill in skills {
        output.push_str(&format!(
            "<skill name=\"{}\">\n<description>{}</description>\n",
            skill.name, skill.description
        ));

        if !skill.metadata.tags.is_empty() {
            output.push_str(&format!(
                "<tags>{}</tags>\n",
                skill.metadata.tags.join(", ")
            ));
        }

        output.push_str("<instructions>\n");
        output.push_str(&skill.content);
        output.push_str("\n</instructions>\n");
        output.push_str("</skill>\n");
    }

    output.push_str("</available-skills>\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SkillMetadata;
    use std::path::PathBuf;

    fn make_skill(name: &str, source: SkillSource) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("Description for {}", name),
            homepage: None,
            content: format!("# {}\n\nInstructions...", name),
            metadata: SkillMetadata::default(),
            source,
        }
    }

    #[test]
    fn test_deduplicate_keeps_highest_precedence() {
        let skills = vec![
            make_skill("test-skill", SkillSource::Bundled),
            make_skill(
                "test-skill",
                SkillSource::Workspace {
                    path: PathBuf::from("/workspace"),
                },
            ),
            make_skill(
                "test-skill",
                SkillSource::EnterpriseRegistry {
                    org: "acme".to_string(),
                    version: "1.0".to_string(),
                },
            ),
        ];

        let deduped = SkillLoader::deduplicate_by_precedence(skills);

        assert_eq!(deduped.len(), 1);
        assert!(matches!(deduped[0].source, SkillSource::Workspace { .. }));
    }

    #[test]
    fn test_build_skills_prompt_empty() {
        let prompt = build_skills_prompt(&[]);
        assert!(prompt.is_empty());
    }

    #[test]
    fn test_build_skills_prompt() {
        let mut skill = make_skill("k8s-debug", SkillSource::Bundled);
        skill.metadata.tags = vec!["kubernetes".to_string(), "debugging".to_string()];

        let prompt = build_skills_prompt(&[skill]);

        assert!(prompt.contains("<available-skills>"));
        assert!(prompt.contains("<skill name=\"k8s-debug\">"));
        assert!(prompt.contains("<description>"));
        assert!(prompt.contains("<tags>kubernetes, debugging</tags>"));
        assert!(prompt.contains("<instructions>"));
        assert!(prompt.contains("</available-skills>"));
    }
}
