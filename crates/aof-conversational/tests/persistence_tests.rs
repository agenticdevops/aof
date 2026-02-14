// Integration tests for file persistence module
//
// Tests atomic file writes, append behavior, and workspace creation.

use aof_conversational::WorkspacePersistence;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_persist_agent_creates_workspace_if_missing() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path);

    let mut files = HashMap::new();
    files.insert(
        "workspace/AGENTS.md".to_string(),
        "- id: test-agent\n  name: Test Agent".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    assert_eq!(result.files_modified.len(), 1);
    assert_eq!(result.files_written.len(), 0);

    // Verify workspace directory created
    assert!(workspace_path.exists());

    // Verify AGENTS.md contains content
    let agents_content = fs::read_to_string(workspace_path.join("AGENTS.md"))
        .await
        .unwrap();
    assert!(agents_content.contains("test-agent"));
}

#[tokio::test]
async fn test_persist_agent_appends_to_existing() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    // Create workspace with existing agent
    fs::create_dir_all(&workspace_path).await.unwrap();
    fs::write(
        workspace_path.join("AGENTS.md"),
        "- id: agent-1\n  name: First Agent",
    )
    .await
    .unwrap();

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path);

    let mut files = HashMap::new();
    files.insert(
        "workspace/AGENTS.md".to_string(),
        "- id: agent-2\n  name: Second Agent".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    assert_eq!(result.files_modified.len(), 1);

    // Verify both agents exist
    let agents_content = fs::read_to_string(workspace_path.join("AGENTS.md"))
        .await
        .unwrap();
    assert!(agents_content.contains("agent-1"));
    assert!(agents_content.contains("agent-2"));
}

#[tokio::test]
async fn test_persist_soul_appends_section() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    // Create workspace with existing soul
    fs::create_dir_all(&workspace_path).await.unwrap();
    fs::write(
        workspace_path.join("SOUL.md"),
        "# agent-1\nFirst personality",
    )
    .await
    .unwrap();

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path);

    let mut files = HashMap::new();
    files.insert(
        "workspace/SOUL.md".to_string(),
        "# agent-2\nSecond personality".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    assert_eq!(result.files_modified.len(), 1);

    // Verify both souls exist with separator
    let soul_content = fs::read_to_string(workspace_path.join("SOUL.md"))
        .await
        .unwrap();
    assert!(soul_content.contains("agent-1"));
    assert!(soul_content.contains("agent-2"));
    assert!(soul_content.contains("---")); // Separator
}

#[tokio::test]
async fn test_persist_skill_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    let persistence = WorkspacePersistence::new(workspace_path, skills_path.clone());

    let mut files = HashMap::new();
    files.insert(
        "skills/restart-service/SKILL.md".to_string(),
        "# Restart Service\nHow to restart a service".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    assert_eq!(result.files_written.len(), 1);
    assert_eq!(result.files_modified.len(), 0);

    // Verify skill directory and file created
    let skill_path = skills_path.join("restart-service").join("SKILL.md");
    assert!(skill_path.exists());

    let skill_content = fs::read_to_string(skill_path).await.unwrap();
    assert!(skill_content.contains("Restart Service"));
}

#[tokio::test]
async fn test_persist_trigger_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path);

    let mut files = HashMap::new();
    files.insert(
        "triggers.yaml".to_string(),
        "  - cron: \"0 2 * * *\"\n    agent: backup-agent".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    assert_eq!(result.files_modified.len(), 1);

    // Verify triggers.yaml exists
    let triggers_path = workspace_path.join("triggers.yaml");
    assert!(triggers_path.exists());

    let triggers_content = fs::read_to_string(triggers_path).await.unwrap();
    assert!(triggers_content.contains("backup-agent"));
}

#[tokio::test]
async fn test_persist_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path.clone());

    let mut files = HashMap::new();
    files.insert(
        "workspace/AGENTS.md".to_string(),
        "- id: test-agent\n  name: Test Agent".to_string(),
    );
    files.insert(
        "workspace/SOUL.md".to_string(),
        "# test-agent\nTest personality".to_string(),
    );
    files.insert(
        "skills/deploy/SKILL.md".to_string(),
        "# Deploy\nDeployment skill".to_string(),
    );

    let result = persistence.persist_files(&files).await.unwrap();

    // 2 modified (AGENTS.md, SOUL.md), 1 written (skill)
    assert_eq!(result.files_modified.len(), 2);
    assert_eq!(result.files_written.len(), 1);

    // Verify all files exist
    assert!(workspace_path.join("AGENTS.md").exists());
    assert!(workspace_path.join("SOUL.md").exists());
    assert!(skills_path.join("deploy").join("SKILL.md").exists());
}

#[tokio::test]
async fn test_atomic_write_safety() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    let skills_path = temp_dir.path().join("skills");

    // Create workspace with existing content
    fs::create_dir_all(&workspace_path).await.unwrap();
    fs::write(
        workspace_path.join("AGENTS.md"),
        "- id: original\n  name: Original Agent",
    )
    .await
    .unwrap();

    let persistence = WorkspacePersistence::new(workspace_path.clone(), skills_path);

    let mut files = HashMap::new();
    files.insert(
        "workspace/AGENTS.md".to_string(),
        "- id: new-agent\n  name: New Agent".to_string(),
    );

    persistence.persist_files(&files).await.unwrap();

    // Verify original content preserved (appended, not overwritten)
    let agents_content = fs::read_to_string(workspace_path.join("AGENTS.md"))
        .await
        .unwrap();
    assert!(agents_content.contains("original"));
    assert!(agents_content.contains("new-agent"));
}
