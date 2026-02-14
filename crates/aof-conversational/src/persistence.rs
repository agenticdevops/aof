// File persistence for conversational agent creation
//
// This module handles atomic file writes for agent configurations,
// personality files, skills, and triggers generated through conversation.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

/// Persistence manager for workspace files
#[derive(Debug, Clone)]
pub struct WorkspacePersistence {
    workspace_path: PathBuf,
    skills_path: PathBuf,
}

/// Result of file persistence operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct PersistenceResult {
    pub files_written: Vec<String>,
    pub files_modified: Vec<String>,
}

impl WorkspacePersistence {
    /// Create new workspace persistence manager
    pub fn new(workspace_path: PathBuf, skills_path: PathBuf) -> Self {
        Self {
            workspace_path,
            skills_path,
        }
    }

    /// Persist files from specialist output to workspace
    ///
    /// File paths in the HashMap determine the action:
    /// - "workspace/AGENTS.md" -> append to existing agents
    /// - "workspace/SOUL.md" -> append personality section
    /// - "skills/{name}/SKILL.md" -> create new skill directory + file
    /// - "triggers.yaml" -> append schedule entry
    pub async fn persist_files(&self, files: &HashMap<String, String>) -> Result<PersistenceResult> {
        self.ensure_workspace().await?;

        let mut files_written = Vec::new();
        let mut files_modified = Vec::new();

        for (path, content) in files {
            debug!("Persisting file: {}", path);

            if path == "workspace/AGENTS.md" || path.ends_with("/AGENTS.md") {
                self.append_agent(content).await
                    .context("Failed to append agent")?;
                files_modified.push(path.clone());
            } else if path == "workspace/SOUL.md" || path.ends_with("/SOUL.md") {
                self.append_soul(content).await
                    .context("Failed to append soul")?;
                files_modified.push(path.clone());
            } else if path.starts_with("skills/") && path.ends_with("/SKILL.md") {
                // Extract skill name from path: skills/{name}/SKILL.md
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() >= 2 {
                    let skill_name = parts[1];
                    self.create_skill(skill_name, content).await
                        .context(format!("Failed to create skill: {}", skill_name))?;
                    files_written.push(path.clone());
                }
            } else if path == "triggers.yaml" || path.ends_with("/triggers.yaml") {
                self.append_trigger(content).await
                    .context("Failed to append trigger")?;
                files_modified.push(path.clone());
            } else {
                warn!("Unknown file path pattern: {}", path);
            }
        }

        info!(
            "Persistence complete: {} written, {} modified",
            files_written.len(),
            files_modified.len()
        );

        Ok(PersistenceResult {
            files_written,
            files_modified,
        })
    }

    /// Append agent entry to existing AGENTS.md (don't overwrite)
    async fn append_agent(&self, content: &str) -> Result<()> {
        let agents_path = self.workspace_path.join("AGENTS.md");

        // Read existing content if file exists
        let existing = if agents_path.exists() {
            fs::read_to_string(&agents_path).await
                .context("Failed to read existing AGENTS.md")?
        } else {
            String::new()
        };

        // Append new agent with separator
        let updated = if existing.is_empty() {
            content.to_string()
        } else {
            format!("{}\n\n{}", existing, content)
        };

        // Write atomically via temp file
        self.atomic_write(&agents_path, &updated).await
            .context("Failed to write AGENTS.md")?;

        info!("Appended agent to AGENTS.md");
        Ok(())
    }

    /// Append soul section to existing SOUL.md (don't overwrite)
    async fn append_soul(&self, content: &str) -> Result<()> {
        let soul_path = self.workspace_path.join("SOUL.md");

        // Read existing content if file exists
        let existing = if soul_path.exists() {
            fs::read_to_string(&soul_path).await
                .context("Failed to read existing SOUL.md")?
        } else {
            String::new()
        };

        // Append new soul section with separator
        let updated = if existing.is_empty() {
            content.to_string()
        } else {
            format!("{}\n\n---\n\n{}", existing, content)
        };

        // Write atomically via temp file
        self.atomic_write(&soul_path, &updated).await
            .context("Failed to write SOUL.md")?;

        info!("Appended soul section to SOUL.md");
        Ok(())
    }

    /// Create new skill file: skills/{name}/SKILL.md
    async fn create_skill(&self, name: &str, content: &str) -> Result<()> {
        let skill_dir = self.skills_path.join(name);
        let skill_file = skill_dir.join("SKILL.md");

        // Create directory if it doesn't exist
        fs::create_dir_all(&skill_dir).await
            .context(format!("Failed to create skill directory: {}", name))?;

        // Write skill file atomically
        self.atomic_write(&skill_file, content).await
            .context(format!("Failed to write SKILL.md for: {}", name))?;

        info!("Created skill: {}", name);
        Ok(())
    }

    /// Append schedule to triggers.yaml
    async fn append_trigger(&self, content: &str) -> Result<()> {
        let triggers_path = self.workspace_path.join("triggers.yaml");

        // Read existing content if file exists
        let existing = if triggers_path.exists() {
            fs::read_to_string(&triggers_path).await
                .context("Failed to read existing triggers.yaml")?
        } else {
            // Create empty YAML array
            "schedules: []\n".to_string()
        };

        // Parse existing YAML to append properly
        // For simplicity, we'll append to the schedules array
        let updated = if existing.contains("schedules:") {
            // Append to existing schedules
            format!("{}\n{}", existing, content)
        } else {
            // Create new schedules section
            format!("schedules:\n{}", content)
        };

        // Write atomically via temp file
        self.atomic_write(&triggers_path, &updated).await
            .context("Failed to write triggers.yaml")?;

        info!("Appended trigger to triggers.yaml");
        Ok(())
    }

    /// Create workspace files if they don't exist (first-time setup)
    async fn ensure_workspace(&self) -> Result<()> {
        // Create workspace directory
        if !self.workspace_path.exists() {
            fs::create_dir_all(&self.workspace_path).await
                .context("Failed to create workspace directory")?;
            debug!("Created workspace directory: {:?}", self.workspace_path);
        }

        // Create empty AGENTS.md if missing
        let agents_path = self.workspace_path.join("AGENTS.md");
        if !agents_path.exists() {
            fs::write(&agents_path, "# Agents\n\nThis file contains agent configurations.\n\n")
                .await
                .context("Failed to create AGENTS.md")?;
            debug!("Created empty AGENTS.md");
        }

        // Create empty SOUL.md if missing
        let soul_path = self.workspace_path.join("SOUL.md");
        if !soul_path.exists() {
            fs::write(&soul_path, "# Agent Personalities\n\nThis file contains agent personality configurations.\n\n")
                .await
                .context("Failed to create SOUL.md")?;
            debug!("Created empty SOUL.md");
        }

        // Create skills directory
        if !self.skills_path.exists() {
            fs::create_dir_all(&self.skills_path).await
                .context("Failed to create skills directory")?;
            debug!("Created skills directory: {:?}", self.skills_path);
        }

        Ok(())
    }

    /// Atomic write: write to temp file, then rename
    ///
    /// Prevents partial writes on crash or errors
    async fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
        // Create temp file in same directory
        let parent = path.parent()
            .ok_or_else(|| anyhow!("Path has no parent directory"))?;
        let temp_path = parent.join(format!(
            ".{}.tmp",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("temp")
        ));

        // Write to temp file
        let mut file = fs::File::create(&temp_path).await
            .context("Failed to create temp file")?;
        file.write_all(content.as_bytes()).await
            .context("Failed to write to temp file")?;
        file.sync_all().await
            .context("Failed to sync temp file")?;

        // Atomic rename
        fs::rename(&temp_path, path).await
            .context("Failed to rename temp file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_persistence() -> (WorkspacePersistence, TempDir) {
        let temp = TempDir::new().unwrap();
        let workspace_path = temp.path().join("workspace");
        let skills_path = temp.path().join("skills");

        let persistence = WorkspacePersistence::new(
            workspace_path.clone(),
            skills_path.clone(),
        );

        (persistence, temp)
    }

    #[tokio::test]
    async fn test_persist_new_agent_to_empty_workspace() {
        let (persistence, _temp) = setup_test_persistence().await;

        let mut files = HashMap::new();
        files.insert(
            "workspace/AGENTS.md".to_string(),
            "## Agent: test-agent\nname: test-agent".to_string(),
        );

        let result = persistence.persist_files(&files).await.unwrap();

        assert_eq!(result.files_modified.len(), 1);
        assert!(result.files_modified.contains(&"workspace/AGENTS.md".to_string()));

        // Verify file was created
        let agents_path = persistence.workspace_path.join("AGENTS.md");
        assert!(agents_path.exists());

        let content = fs::read_to_string(&agents_path).await.unwrap();
        assert!(content.contains("test-agent"));
    }

    #[tokio::test]
    async fn test_persist_agent_appends_to_existing() {
        let (persistence, _temp) = setup_test_persistence().await;

        // Create workspace with existing agent
        persistence.ensure_workspace().await.unwrap();
        let agents_path = persistence.workspace_path.join("AGENTS.md");
        fs::write(&agents_path, "## Agent: existing-agent\nname: existing")
            .await
            .unwrap();

        // Add new agent
        let mut files = HashMap::new();
        files.insert(
            "workspace/AGENTS.md".to_string(),
            "## Agent: new-agent\nname: new".to_string(),
        );

        persistence.persist_files(&files).await.unwrap();

        // Verify both agents exist
        let content = fs::read_to_string(&agents_path).await.unwrap();
        assert!(content.contains("existing-agent"));
        assert!(content.contains("new-agent"));
    }

    #[tokio::test]
    async fn test_persist_skill_creates_directory() {
        let (persistence, _temp) = setup_test_persistence().await;

        let mut files = HashMap::new();
        files.insert(
            "skills/test-skill/SKILL.md".to_string(),
            "# Test Skill\nThis is a test skill.".to_string(),
        );

        let result = persistence.persist_files(&files).await.unwrap();

        assert_eq!(result.files_written.len(), 1);
        assert!(result.files_written.contains(&"skills/test-skill/SKILL.md".to_string()));

        // Verify directory and file created
        let skill_dir = persistence.skills_path.join("test-skill");
        assert!(skill_dir.exists());

        let skill_file = skill_dir.join("SKILL.md");
        assert!(skill_file.exists());

        let content = fs::read_to_string(&skill_file).await.unwrap();
        assert!(content.contains("Test Skill"));
    }

    #[tokio::test]
    async fn test_persist_trigger_appends() {
        let (persistence, _temp) = setup_test_persistence().await;

        let mut files = HashMap::new();
        files.insert(
            "triggers.yaml".to_string(),
            "  - name: test-trigger\n    cron: '0 0 * * *'".to_string(),
        );

        let result = persistence.persist_files(&files).await.unwrap();

        assert_eq!(result.files_modified.len(), 1);

        // Verify file created
        let triggers_path = persistence.workspace_path.join("triggers.yaml");
        assert!(triggers_path.exists());

        let content = fs::read_to_string(&triggers_path).await.unwrap();
        assert!(content.contains("test-trigger"));
    }

    #[tokio::test]
    async fn test_ensure_workspace_creates_files() {
        let (persistence, _temp) = setup_test_persistence().await;

        persistence.ensure_workspace().await.unwrap();

        // Verify workspace directory created
        assert!(persistence.workspace_path.exists());

        // Verify AGENTS.md created
        let agents_path = persistence.workspace_path.join("AGENTS.md");
        assert!(agents_path.exists());

        // Verify SOUL.md created
        let soul_path = persistence.workspace_path.join("SOUL.md");
        assert!(soul_path.exists());

        // Verify skills directory created
        assert!(persistence.skills_path.exists());
    }

    #[tokio::test]
    async fn test_atomic_write_safety() {
        let (persistence, _temp) = setup_test_persistence().await;
        persistence.ensure_workspace().await.unwrap();

        let test_path = persistence.workspace_path.join("test.txt");

        // Write initial content
        persistence.atomic_write(&test_path, "initial").await.unwrap();
        assert_eq!(
            fs::read_to_string(&test_path).await.unwrap(),
            "initial"
        );

        // Overwrite with new content
        persistence.atomic_write(&test_path, "updated").await.unwrap();
        assert_eq!(
            fs::read_to_string(&test_path).await.unwrap(),
            "updated"
        );

        // Verify no temp files left behind
        let temp_file = persistence.workspace_path.join(".test.txt.tmp");
        assert!(!temp_file.exists());
    }

    #[tokio::test]
    async fn test_persist_multiple_files() {
        let (persistence, _temp) = setup_test_persistence().await;

        let mut files = HashMap::new();
        files.insert(
            "workspace/AGENTS.md".to_string(),
            "## Agent: multi-test".to_string(),
        );
        files.insert(
            "workspace/SOUL.md".to_string(),
            "## Personality: multi-test".to_string(),
        );
        files.insert(
            "skills/multi/SKILL.md".to_string(),
            "# Multi Skill".to_string(),
        );

        let result = persistence.persist_files(&files).await.unwrap();

        assert_eq!(result.files_modified.len(), 2); // AGENTS.md, SOUL.md
        assert_eq!(result.files_written.len(), 1);  // skill

        // Verify all files created
        assert!(persistence.workspace_path.join("AGENTS.md").exists());
        assert!(persistence.workspace_path.join("SOUL.md").exists());
        assert!(persistence.skills_path.join("multi/SKILL.md").exists());
    }
}
