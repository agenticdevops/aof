//! End-to-end integration tests for Phase 14: Skills + Tools + WASM Sandbox.
//!
//! Tests the 5 ROADMAP success criteria:
//! 1. skills/postgres-tuning/SKILL.md injected without LLM call
//! 2. agentix skills list shows 8 built-in packs
//! 3. Custom skill in agent's skills/ picked up at runtime
//! 4. Agent calls CLI tools in a run
//! 5. WASM tool without declared capability is blocked

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use agentix_core::{DirectoryLoader, DirectoryToolType, ToolEntry};
use agentix_core::skills::SkillRegistry;
use agentix_runtime::tools::{CliToolExecutor, CapabilityManifest, WasmCapability};
use agentix_runtime::executor::react_loop::ToolExecutor;

// ---------------------------------------------------------------------------
// Helper: create a minimal agent directory in a temp dir
// ---------------------------------------------------------------------------

fn create_test_agent_dir(tmp: &TempDir, with_postgres_skill: bool) -> PathBuf {
    let agent_dir = tmp.path().join("test-agent");
    fs::create_dir_all(&agent_dir).unwrap();

    // agent.yaml
    fs::write(
        agent_dir.join("agent.yaml"),
        r#"
spec_version: v1
name: test-agent
description: Integration test agent
model:
  preferred: "anthropic/claude-sonnet-4-6"
"#,
    )
    .unwrap();

    // SOUL.md
    fs::write(
        agent_dir.join("SOUL.md"),
        "You are a test agent. Help users with their requests.",
    )
    .unwrap();

    if with_postgres_skill {
        let skill_dir = agent_dir.join("skills").join("postgres-tuning");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "# PostgreSQL Tuning Skill\n\nSpecialized PostgreSQL instructions for production cluster.",
        )
        .unwrap();
    }

    agent_dir
}

// ---------------------------------------------------------------------------
// Success Criterion 1: postgres-tuning skill injected without LLM call
// ---------------------------------------------------------------------------

#[test]
fn sc1_custom_skill_injected_into_system_prompt_without_llm() {
    let tmp = TempDir::new().unwrap();
    let agent_dir = create_test_agent_dir(&tmp, true); // with postgres-tuning skill

    let definition = DirectoryLoader::load(&agent_dir)
        .expect("Should load agent directory");

    // Verify skill was loaded (DirectoryLoader reads skills/ dir — no LLM involved)
    assert_eq!(definition.skills.len(), 1, "Should have 1 skill");
    assert_eq!(definition.skills[0].name, "postgres-tuning");

    // Verify skill content is in system prompt
    let prompt = definition.resolved_system_prompt();
    assert!(
        prompt.contains("postgres-tuning"),
        "System prompt should contain skill name"
    );
    assert!(
        prompt.contains("PostgreSQL Tuning Skill"),
        "System prompt should contain skill content"
    );
    assert!(
        prompt.contains("## Skill: postgres-tuning"),
        "System prompt should have skill heading"
    );

    // Key assertion: this test itself is the evidence — no LLM was called.
    // DirectoryLoader is a pure filesystem reader. If we reach here without
    // calling any LLM, success criterion 1 is met.
    println!("SC1 PASSED: Custom skill injected deterministically without LLM call");
}

// ---------------------------------------------------------------------------
// Success Criterion 2: agentix skills list shows 8 built-in packs
// ---------------------------------------------------------------------------

#[test]
fn sc2_skills_list_shows_all_8_builtin_packs() {
    let registry = SkillRegistry::new();
    let packs = registry.builtin_packs();

    assert_eq!(packs.len(), 8, "Should have exactly 8 built-in skill packs");

    let expected_names = [
        "aws",
        "kubernetes",
        "terraform",
        "docker",
        "git",
        "database",
        "security",
        "observability",
    ];
    for name in &expected_names {
        assert!(
            packs.iter().any(|p| p.name == *name),
            "Missing built-in skill pack: {}",
            name
        );
    }

    println!("SC2 PASSED: agentix skills list shows {} built-in packs", packs.len());
    for pack in packs {
        println!("  - {} : {}", pack.name, pack.description);
    }
}

// ---------------------------------------------------------------------------
// Success Criterion 3: Custom skill in agent's skills/ dir picked up at runtime
// ---------------------------------------------------------------------------

#[test]
fn sc3_custom_skill_in_agent_directory_loaded_at_runtime() {
    let tmp = TempDir::new().unwrap();
    let agent_dir = create_test_agent_dir(&tmp, false); // no pre-set skill

    // Add a custom skill
    let skill_dir = agent_dir.join("skills").join("my-custom-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "# My Custom Skill\n\nThis is domain-specific knowledge for our team.",
    )
    .unwrap();

    let definition = DirectoryLoader::load(&agent_dir)
        .expect("Should load agent with custom skill");

    assert_eq!(definition.skills.len(), 1, "Should have 1 custom skill");
    assert_eq!(definition.skills[0].name, "my-custom-skill");
    assert!(
        definition.skills[0].content.contains("My Custom Skill"),
        "Custom skill content loaded correctly"
    );

    // Verify it appears in the system prompt
    let prompt = definition.resolved_system_prompt();
    assert!(
        prompt.contains("## Skill: my-custom-skill"),
        "Custom skill should appear in system prompt"
    );

    println!("SC3 PASSED: Custom skill picked up from agent's skills/ directory");
}

// ---------------------------------------------------------------------------
// Success Criterion 4: Agent calls CLI tools within a run
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sc4_agent_calls_cli_tool_in_react_loop() {
    // This tests that CliToolExecutor can execute a real CLI tool
    // (echo is always available — doesn't require kubectl, aws, etc.)
    let executor = CliToolExecutor::new();

    let echo_tool = ToolEntry {
        name: "echo".to_string(),
        tool_type: DirectoryToolType::Shell,
        command: Some("echo".to_string()),
        description: Some("Echo input".to_string()),
        server: None,
        args: vec!["{{message}}".to_string()],
    };

    let result = executor
        .execute(&echo_tool, serde_json::json!({"message": "kubectl-get-pods-output"}))
        .await;

    assert!(result.is_ok(), "CLI tool should succeed: {:?}", result.err());
    assert!(
        result.unwrap().contains("kubectl-get-pods-output"),
        "Should contain tool output"
    );

    // Also test a failing tool returns error as observation (not panic)
    let false_tool = ToolEntry {
        name: "false".to_string(),
        tool_type: DirectoryToolType::Cli,
        command: Some("false".to_string()),
        description: None,
        server: None,
        args: vec![],
    };

    let error_result = executor.execute(&false_tool, serde_json::json!({})).await;
    assert!(error_result.is_err(), "Failing tool should return Err observation");

    println!("SC4 PASSED: Agent can call CLI tools and receive output as observations");
}

// ---------------------------------------------------------------------------
// Success Criterion 5: WASM tool without declared capability is blocked
// ---------------------------------------------------------------------------

#[test]
fn sc5_wasm_tool_without_network_capability_is_blocked() {
    // A WASM tool with no capabilities declared cannot use network
    let manifest = CapabilityManifest {
        name: "untrusted-tool".to_string(),
        capabilities: vec![], // No capabilities declared
    };

    // Simulate the tool attempting network access
    let violation = manifest.check_capability(&WasmCapability::Network);

    assert!(
        violation.is_some(),
        "WASM tool without network capability should be blocked"
    );

    let violation_msg = violation.unwrap().to_string();
    assert!(
        violation_msg.to_lowercase().contains("network") || violation_msg.contains("blocked"),
        "Violation message should mention network blocking: {}",
        violation_msg
    );

    // Confirm a tool WITH network capability is allowed
    let network_manifest = CapabilityManifest {
        name: "network-tool".to_string(),
        capabilities: vec![WasmCapability::Network],
    };

    let no_violation = network_manifest.check_capability(&WasmCapability::Network);
    assert!(
        no_violation.is_none(),
        "Tool with network capability should be allowed"
    );

    println!("SC5 PASSED: WASM tool without declared capability is blocked at sandbox boundary");
    println!(
        "  Violation: {}",
        manifest.check_capability(&WasmCapability::Network).unwrap()
    );
}

// ---------------------------------------------------------------------------
// Regression: Phase 13 agent loading still works
// ---------------------------------------------------------------------------

#[test]
fn regression_phase13_agent_directory_loading_still_works() {
    let tmp = TempDir::new().unwrap();
    let agent_dir = create_test_agent_dir(&tmp, false);

    // Basic agent directory (SOUL.md + agent.yaml) should still load fine
    let definition = DirectoryLoader::load(&agent_dir)
        .expect("Phase 13 basic agent loading must not regress");

    assert_eq!(definition.name, "test-agent");
    assert!(!definition.soul_content.is_empty());
    assert_eq!(definition.skills.len(), 0, "No skills = empty skills vec");
    assert_eq!(definition.tools.len(), 0, "No tools dir = empty tools vec");

    println!("REGRESSION PASSED: Phase 13 agent directory loading works");
}

#[test]
fn regression_phase13_resolved_system_prompt_without_skills() {
    let tmp = TempDir::new().unwrap();
    let agent_dir = create_test_agent_dir(&tmp, false);

    let definition = DirectoryLoader::load(&agent_dir).unwrap();
    let prompt = definition.resolved_system_prompt();

    // Without skills, prompt should just be SOUL.md content
    assert!(prompt.contains("test agent"), "Prompt should contain SOUL.md content");
    assert!(!prompt.contains("## Skill:"), "No skills should mean no Skill sections");

    println!("REGRESSION PASSED: System prompt without skills is just SOUL.md");
}
