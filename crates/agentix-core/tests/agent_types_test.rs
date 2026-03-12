//! TDD integration tests for AgentManifest, AgentDefinition, DirectoryLoader, and WorkspaceConfig.
//!
//! These tests implement the full spec from:
//! - docs/spec/agent-yaml-v1.md
//! - docs/spec/agent-directory-structure.md
//! - docs/spec/workspace-config.md

use agentix_core::{
    AgentDefinition, AgentFormat, AgentLoader, AgentManifest, DirectoryLoader, WorkspaceConfig,
};
use std::io::Write;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn minimal_agent_yaml() -> &'static str {
    r#"
spec_version: "0.1.0"
name: my-agent
"#
}

// ---------------------------------------------------------------------------
// AgentManifest deserialization tests
// ---------------------------------------------------------------------------

#[test]
fn test_minimal_manifest_yaml() {
    let yaml = r#"
spec_version: "0.1.0"
name: my-agent
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    assert_eq!(manifest.spec_version, "0.1.0");
    assert_eq!(manifest.name, "my-agent");
    assert!(manifest.model.is_none());
    assert!(manifest.version.is_none());
}

#[test]
fn test_full_manifest_yaml() {
    let yaml = r#"
spec_version: "0.1.0"
name: db-optimizer
version: "1.2.0"
description: "Analyzes slow PostgreSQL queries"
model:
  preferred: anthropic/claude-sonnet-4-6
extends: https://github.com/org/base-agent.git
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    assert_eq!(manifest.spec_version, "0.1.0");
    assert_eq!(manifest.name, "db-optimizer");
    assert_eq!(manifest.version.as_deref(), Some("1.2.0"));
    assert_eq!(
        manifest.description.as_deref(),
        Some("Analyzes slow PostgreSQL queries")
    );
    assert_eq!(
        manifest.model.as_ref().and_then(|m| m.preferred.as_deref()),
        Some("anthropic/claude-sonnet-4-6")
    );
    assert_eq!(
        manifest.extends.as_deref(),
        Some("https://github.com/org/base-agent.git")
    );
}

#[test]
fn test_manifest_with_dependencies() {
    let yaml = r#"
spec_version: "0.1.0"
name: coordinator
dependencies:
  - name: fact-checker
    source: https://github.com/org/fact-checker.git
    version: "^1.0.0"
    mount: agents/fact-checker
  - name: summarizer
    source: ./local-agents/summarizer
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    assert_eq!(manifest.dependencies.len(), 2);
    assert_eq!(manifest.dependencies[0].name, "fact-checker");
    assert_eq!(
        manifest.dependencies[0].source,
        "https://github.com/org/fact-checker.git"
    );
    assert_eq!(manifest.dependencies[0].version.as_deref(), Some("^1.0.0"));
    assert_eq!(
        manifest.dependencies[0].mount.as_deref(),
        Some("agents/fact-checker")
    );
    assert_eq!(manifest.dependencies[1].name, "summarizer");
}

#[test]
fn test_manifest_invalid_name_uppercase() {
    let yaml = r#"
spec_version: "0.1.0"
name: My-Agent
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    let err = manifest.validate().expect_err("validation should fail for uppercase name");
    let msg = err.to_string();
    assert!(
        msg.contains("name") || msg.contains("Name") || msg.contains("lowercase"),
        "Error should mention name: {msg}"
    );
}

#[test]
fn test_manifest_invalid_name_too_long() {
    // 64 characters — exceeds 63 char limit
    let long_name = "a".repeat(64);
    let yaml = format!("spec_version: \"0.1.0\"\nname: {long_name}\n");
    let manifest = AgentManifest::from_yaml(&yaml).expect("parse should succeed");
    let err = manifest.validate().expect_err("validation should fail for 64-char name");
    let msg = err.to_string();
    assert!(
        msg.contains("name") || msg.contains("Name") || msg.contains("63"),
        "Error should mention name length: {msg}"
    );
}

#[test]
fn test_manifest_model_format_validated() {
    let yaml = r#"
spec_version: "0.1.0"
name: my-agent
model:
  preferred: claude-sonnet-4-6
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    let err = manifest.validate().expect_err("validation should fail for model without /");
    let msg = err.to_string();
    assert!(
        msg.contains("model") || msg.contains("provider"),
        "Error should mention model format: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AgentDefinition assembly tests (filesystem-based)
// ---------------------------------------------------------------------------

#[test]
fn test_load_minimal_agent_directory() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a helpful assistant.");

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    assert_eq!(definition.name, "my-agent");
    assert!(!definition.soul_content.is_empty());
    assert!(definition.rules_content.is_none());
}

#[test]
fn test_load_agent_with_rules() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a helpful assistant.");
    write_file(&dir, "RULES.md", "Never execute destructive commands.");

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    assert!(definition.rules_content.is_some());
    assert!(!definition.rules_content.unwrap().is_empty());
}

#[test]
fn test_load_agent_with_skills() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a helpful assistant.");
    write_file(
        &dir,
        "skills/postgres-tuning/SKILL.md",
        "When analyzing queries, always run EXPLAIN ANALYZE first.",
    );

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    assert_eq!(definition.skills.len(), 1);
    assert_eq!(definition.skills[0].name, "postgres-tuning");
    assert!(!definition.skills[0].content.is_empty());
}

#[test]
fn test_load_agent_with_tools() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a helpful assistant.");
    write_file(
        &dir,
        "tools/kubectl.yaml",
        r#"
name: kubectl
description: Run kubectl commands
type: cli
command: kubectl
"#,
    );

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    assert_eq!(definition.tools.len(), 1);
    assert_eq!(definition.tools[0].name, "kubectl");
}

#[test]
fn test_load_missing_soul_md_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    // No SOUL.md

    let result = DirectoryLoader::load(dir.path());
    assert!(
        result.is_err(),
        "Loading without SOUL.md should return an error"
    );
}

#[test]
fn test_resolved_system_prompt_composition() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a database expert.");
    write_file(&dir, "RULES.md", "Never drop tables without approval.");
    write_file(
        &dir,
        "skills/postgres-tuning/SKILL.md",
        "Always use EXPLAIN ANALYZE.",
    );

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    let prompt = definition.resolved_system_prompt();

    // SOUL content should appear first
    assert!(prompt.contains("You are a database expert."), "Prompt missing SOUL content");
    // RULES content with header
    assert!(
        prompt.contains("## Constraints"),
        "Prompt missing Constraints header"
    );
    assert!(
        prompt.contains("Never drop tables without approval."),
        "Prompt missing RULES content"
    );
    // Skill content with header
    assert!(
        prompt.contains("## Skill: postgres-tuning"),
        "Prompt missing skill header"
    );
    assert!(
        prompt.contains("Always use EXPLAIN ANALYZE."),
        "Prompt missing skill content"
    );

    // Check ordering: SOUL before RULES, RULES before skills
    let soul_pos = prompt.find("You are a database expert.").unwrap();
    let rules_pos = prompt.find("## Constraints").unwrap();
    let skill_pos = prompt.find("## Skill: postgres-tuning").unwrap();
    assert!(soul_pos < rules_pos, "SOUL should appear before RULES");
    assert!(rules_pos < skill_pos, "RULES should appear before skills");
}

#[test]
fn test_load_sub_agents_directory() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "agent.yaml", minimal_agent_yaml());
    write_file(&dir, "SOUL.md", "You are a coordinator.");
    write_file(
        &dir,
        "agents/sub-agent/agent.yaml",
        r#"
spec_version: "0.1.0"
name: sub-agent
"#,
    );
    write_file(
        &dir,
        "agents/sub-agent/SOUL.md",
        "You are a specialist.",
    );

    let definition = DirectoryLoader::load(dir.path()).expect("load should succeed");
    assert_eq!(definition.sub_agents.len(), 1, "Should have one sub-agent");
    assert_eq!(definition.sub_agents[0].name, "sub-agent");
}

#[test]
fn test_directory_loader_missing_agent_yaml_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir, "SOUL.md", "You are a helpful assistant.");
    // No agent.yaml

    let result = DirectoryLoader::load(dir.path());
    assert!(
        result.is_err(),
        "Loading without agent.yaml should return an error"
    );
}

// ---------------------------------------------------------------------------
// Backward compat flat YAML tests
// ---------------------------------------------------------------------------

#[test]
fn test_load_flat_yaml_agent() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: my-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: |
    You are a helpful assistant.
"#;
    let dir = tempfile::tempdir().unwrap();
    let yaml_file = dir.path().join("my-agent.yaml");
    std::fs::write(&yaml_file, yaml).unwrap();

    let definition = AgentLoader::load(&yaml_file).expect("flat YAML load should succeed");
    assert_eq!(definition.name, "my-agent");
    assert_eq!(
        definition.model_preferred.as_deref(),
        Some("anthropic/claude-sonnet-4-6")
    );
    assert!(!definition.soul_content.is_empty());
    assert!(definition.soul_content.contains("You are a helpful assistant."));
}

#[test]
fn test_autodetect_format_directory_vs_flat() {
    let dir = tempfile::tempdir().unwrap();
    // Directory path → Directory format
    let dir_format = AgentLoader::detect_format(dir.path());
    assert!(
        matches!(dir_format, AgentFormat::Directory),
        "Directory path should detect as Directory format"
    );

    // .yaml file path → FlatYaml format
    let yaml_file = dir.path().join("agent.yaml");
    std::fs::write(&yaml_file, "").unwrap();
    let flat_format = AgentLoader::detect_format(&yaml_file);
    assert!(
        matches!(flat_format, AgentFormat::FlatYaml),
        ".yaml file path should detect as FlatYaml format"
    );
}

// ---------------------------------------------------------------------------
// WorkspaceConfig tests
// ---------------------------------------------------------------------------

#[test]
fn test_workspace_config_deserialize() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-project
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 15
    timeout: "10m"
    mode: autonomous
  providers:
    anthropic:
      api_key: "sk-test-key"
  gateway:
    host: "127.0.0.1"
    port: 7777
  agents_dir: "./agents"
"#;
    let config = WorkspaceConfig::from_yaml(yaml).expect("workspace parse should succeed");
    assert_eq!(config.metadata.name, "my-project");
    assert_eq!(
        config.spec.defaults.model.as_deref(),
        Some("anthropic/claude-sonnet-4-6")
    );
    assert_eq!(config.spec.defaults.max_iterations, Some(15));
    assert_eq!(config.spec.defaults.timeout.as_deref(), Some("10m"));
    assert_eq!(config.spec.gateway.host, "127.0.0.1");
    assert_eq!(config.spec.gateway.port, 7777);
    assert_eq!(config.spec.agents_dir, "./agents");
    assert!(config.spec.providers.contains_key("anthropic"));
}

#[test]
fn test_workspace_defaults_applied_to_definition() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  defaults:
    max_iterations: 20
    mode: autonomous
"#;
    let workspace = WorkspaceConfig::from_yaml(yaml).expect("workspace parse should succeed");

    let mut definition = AgentDefinition {
        name: "test-agent".to_string(),
        version: None,
        description: None,
        model_preferred: None,
        soul_content: "You are helpful.".to_string(),
        rules_content: None,
        skills: vec![],
        tools: vec![],
        sub_agents: vec![],
        max_iterations: 10,
        timeout_secs: 300,
        mode: agentix_core::AgentMode::Manual,
    };

    definition.apply_workspace_defaults(&workspace);
    assert_eq!(definition.max_iterations, 20, "max_iterations should be overridden");
    assert!(
        matches!(definition.mode, agentix_core::AgentMode::Autonomous),
        "mode should be Autonomous"
    );
}

#[test]
fn test_env_var_expansion() {
    // Set a test env var
    std::env::set_var("TEST_AGENTIX_API_KEY", "sk-expanded-key");

    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  providers:
    anthropic:
      api_key: "${TEST_AGENTIX_API_KEY}"
"#;
    let mut config = WorkspaceConfig::from_yaml(yaml).expect("workspace parse should succeed");
    config.expand_env_vars().expect("env var expansion should succeed");

    let api_key = config
        .spec
        .providers
        .get("anthropic")
        .and_then(|p| p.api_key.as_deref());
    assert_eq!(api_key, Some("sk-expanded-key"));

    // Clean up
    std::env::remove_var("TEST_AGENTIX_API_KEY");
}

#[test]
fn test_serde_path_to_error_on_bad_yaml() {
    // port should be an integer, not a string
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  gateway:
    host: "127.0.0.1"
    port: "not-a-number"
"#;
    let result = WorkspaceConfig::from_yaml(yaml);
    assert!(result.is_err(), "Invalid port type should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("port") || err.contains("spec.gateway"),
        "Error should mention the field path, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Error message tests
// ---------------------------------------------------------------------------

#[test]
fn test_validation_error_shows_field() {
    let yaml = r#"
spec_version: "0.1.0"
name: Invalid-Name
"#;
    let manifest = AgentManifest::from_yaml(yaml).expect("parse should succeed");
    let err = manifest.validate().expect_err("validation should fail");
    let msg = err.to_string();
    // The error message should mention the field
    assert!(
        msg.contains("name") || msg.contains("Name"),
        "Validation error should mention the 'name' field, got: {msg}"
    );
}
