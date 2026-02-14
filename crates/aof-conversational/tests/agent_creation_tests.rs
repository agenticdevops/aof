//! Integration tests for agent creation flow
//!
//! Tests the full pipeline: intent -> specialist -> generation -> validation -> preview -> confirm

use aof_conversational::specialists::{AgentCreator, Specialist};
use aof_conversational::types::{ConversationSession, IntentClassification, IntentType};
use aof_core::{AofResult, Model, ModelConfig, ModelProvider, ModelRequest, ModelResponse, StopReason, Usage, StreamChunk};
use aof_skills::SkillRegistry;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

/// Mock Model that returns predetermined responses
struct MockModel {
    responses: Vec<String>,
    current: AtomicUsize,
}

impl MockModel {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            current: AtomicUsize::new(0),
        }
    }

    fn single(response: String) -> Self {
        Self::new(vec![response])
    }
}

#[async_trait]
impl Model for MockModel {
    async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
        let idx = self.current.fetch_add(1, Ordering::SeqCst);
        let content = self
            .responses
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "fallback".to_string());

        Ok(ModelResponse {
            content,
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 200,
            },
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<std::pin::Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>>
    {
        unimplemented!("Stream not needed for tests")
    }

    fn config(&self) -> &ModelConfig {
        unimplemented!("Config not needed for tests")
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Anthropic
    }
}

/// Create a test workspace with AGENTS.md and skills
async fn create_test_workspace() -> (TempDir, Arc<SkillRegistry>) {
    let temp = tempfile::tempdir().unwrap();

    // Create empty AGENTS.md (agents: [])
    let agents_content = "agents: []\n";
    fs::write(temp.path().join("AGENTS.md"), agents_content)
        .await
        .unwrap();

    // Create a mock skill registry
    let registry = Arc::new(SkillRegistry::default_registry());

    (temp, registry)
}

/// Create a basic intent classification for CreateAgent
fn create_agent_intent(agent_type: &str, description: &str) -> IntentClassification {
    let mut parameters = HashMap::new();
    parameters.insert("agent_type".to_string(), serde_json::json!(agent_type));
    parameters.insert("description".to_string(), serde_json::json!(description));
    parameters.insert("skills".to_string(), serde_json::json!([]));

    IntentClassification {
        intent: IntentType::CreateAgent,
        confidence: 0.95,
        parameters,
        clarifying_questions: vec![],
    }
}

#[tokio::test]
async fn test_create_agent_from_description() {
    let (workspace, registry) = create_test_workspace().await;

    // Mock responses: agent YAML, then SOUL.md
    let agent_yaml = r#"id: k8s-monitor
name: Kubernetes Monitor
role: Infrastructure Specialist
avatar: 🔍
personality_traits: [methodical, detail-oriented, proactive]
can: [monitor clusters, analyze metrics, detect anomalies]
cannot: [modify production, delete resources]
skills: []
"#;

    let soul_md = r#"```yaml
id: k8s-monitor
communication_style: formal-technical
tone: calm-professional
values: [reliability, accuracy]
personality_summary: A methodical monitoring agent
boundaries: [Never modify production, Always verify data]
default_intro: Hello, I monitor your Kubernetes clusters
```

## Communication Style

I communicate with precision and clarity, focusing on factual observations.
"#;

    let model = Arc::new(MockModel::new(vec![
        agent_yaml.to_string(),
        soul_md.to_string(),
    ]));

    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("monitor", "K8s monitoring agent");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_ok(), "Agent creation should succeed");

    let output = result.unwrap();
    assert_eq!(output.files.len(), 2, "Should generate 2 files");
    assert!(output.requires_confirmation, "Should require confirmation");
    assert!(output.message.contains("k8s-monitor"), "Message should mention agent ID");

    // Verify AGENTS.md content
    let agents_path = workspace.path().join("AGENTS.md").to_string_lossy().to_string();
    let agents_content = output.files.get(&agents_path).unwrap();
    assert!(agents_content.contains("k8s-monitor"));
    assert!(agents_content.contains("Infrastructure Specialist"));

    // Verify SOUL.md content
    let soul_path = workspace.path().join("SOUL.md").to_string_lossy().to_string();
    let soul_content = output.files.get(&soul_path).unwrap();
    assert!(soul_content.contains("communication_style"));
    assert!(soul_content.contains("Communication Style"));
}

#[tokio::test]
async fn test_agent_yaml_validation_catches_duplicate() {
    let (workspace, registry) = create_test_workspace().await;

    // Create existing agent
    let existing_yaml = r#"agents:
  - id: existing-agent
    name: Existing
    role: Test
    avatar: 🧪
    personality_traits: [test]
    can: [test]
    cannot: [break]
    skills: []
"#;
    fs::write(workspace.path().join("AGENTS.md"), existing_yaml)
        .await
        .unwrap();

    // Mock returns agent with duplicate ID
    let duplicate_yaml = r#"id: existing-agent
name: Duplicate
role: Test
avatar: 🧪
personality_traits: [test]
can: [test]
cannot: [break]
skills: []
"#;

    let model = Arc::new(MockModel::single(duplicate_yaml.to_string()));
    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("test", "A test agent");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_err(), "Should fail on duplicate ID");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Duplicate"), "Error should mention duplicate");
}

#[tokio::test]
async fn test_skill_hallucination_detection() {
    let (workspace, registry) = create_test_workspace().await;

    // Mock returns agent with non-existent skill
    let agent_yaml = r#"id: test-agent
name: Test Agent
role: Tester
avatar: 🧪
personality_traits: [curious]
can: [test things]
cannot: [break things]
skills: [nonexistent-skill, another-fake-skill]
"#;

    let model = Arc::new(MockModel::single(agent_yaml.to_string()));
    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("test", "A test agent");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_err(), "Should fail on hallucinated skills");

    let err = result.unwrap_err();
    // After skill correction attempt, all skills were invalid
    assert!(err.to_string().contains("invalid") || err.to_string().contains("skill"));
}

#[tokio::test]
async fn test_soul_generation_matches_agent() {
    let (workspace, registry) = create_test_workspace().await;

    let agent_yaml = r#"id: test-soul-agent
name: Soul Test
role: Tester
avatar: 🧪
personality_traits: [friendly, helpful]
can: [assist users]
cannot: [harm anyone]
skills: []
"#;

    let soul_md = r#"```yaml
id: test-soul-agent
communication_style: casual-friendly
tone: warm-encouraging
values: [helpfulness]
personality_summary: A friendly testing agent
boundaries: [Always be kind]
default_intro: Hi, I'm here to help
```

## Communication Style

I speak in a warm and encouraging manner.
"#;

    let model = Arc::new(MockModel::new(vec![
        agent_yaml.to_string(),
        soul_md.to_string(),
    ]));

    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("assistant", "A helpful agent");

    let result = creator.handle(&intent, &session).await.unwrap();

    let soul_path = workspace.path().join("SOUL.md").to_string_lossy().to_string();
    let soul_content = result.files.get(&soul_path).unwrap();

    // Verify SOUL.md matches agent ID and personality
    assert!(soul_content.contains("test-soul-agent"));
    assert!(soul_content.contains("friendly"));
    assert!(soul_content.contains("Communication Style"));
}

#[tokio::test]
async fn test_retry_on_invalid_yaml() {
    let (workspace, registry) = create_test_workspace().await;

    // First response: invalid YAML, second: valid
    let invalid_yaml = "this is not yaml at all!!!";
    let valid_yaml = r#"id: retry-agent
name: Retry Test
role: Tester
avatar: 🔄
personality_traits: [persistent]
can: [retry]
cannot: [give up]
skills: []
"#;

    let soul_md = r#"```yaml
id: retry-agent
communication_style: determined
tone: persistent
values: [persistence]
personality_summary: Never gives up
boundaries: [Keep trying]
default_intro: I'll keep trying
```

## Communication Style

I never give up on a task.
"#;

    let model = Arc::new(MockModel::new(vec![
        invalid_yaml.to_string(),
        valid_yaml.to_string(),
        soul_md.to_string(),
    ]));

    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("tester", "An agent that retries");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_ok(), "Should succeed after retry");

    let output = result.unwrap();
    let agents_path = workspace.path().join("AGENTS.md").to_string_lossy().to_string();
    let content = output.files.get(&agents_path).unwrap();
    assert!(content.contains("retry-agent"));
}

#[tokio::test]
async fn test_preview_before_save() {
    let (workspace, registry) = create_test_workspace().await;

    let agent_yaml = r#"id: preview-agent
name: Preview Test
role: Previewer
avatar: 👁️
personality_traits: [careful]
can: [preview]
cannot: [save without confirmation]
skills: []
"#;

    let soul_md = r#"```yaml
id: preview-agent
communication_style: careful
tone: cautious
values: [caution]
personality_summary: Always previews first
boundaries: [Never rush]
default_intro: Let me show you first
```

## Communication Style

I always show you what I'm about to do before doing it.
"#;

    let model = Arc::new(MockModel::new(vec![
        agent_yaml.to_string(),
        soul_md.to_string(),
    ]));

    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("previewer", "Shows preview");

    let output = creator.handle(&intent, &session).await.unwrap();

    // Verify preview contains both files
    assert_eq!(output.files.len(), 2);
    assert!(output.requires_confirmation);

    // Verify both file paths are correct
    let agents_path = workspace.path().join("AGENTS.md").to_string_lossy().to_string();
    let soul_path = workspace.path().join("SOUL.md").to_string_lossy().to_string();

    assert!(output.files.contains_key(&agents_path));
    assert!(output.files.contains_key(&soul_path));
}

#[tokio::test]
async fn test_empty_workspace_first_agent() {
    let temp = tempfile::tempdir().unwrap();
    let registry = Arc::new(SkillRegistry::default_registry());

    // No AGENTS.md file exists

    let agent_yaml = r#"id: first-agent
name: First Agent
role: Pioneer
avatar: 🌟
personality_traits: [pioneering]
can: [start fresh]
cannot: [depend on others]
skills: []
"#;

    let soul_md = r#"```yaml
id: first-agent
communication_style: pioneering
tone: confident
values: [independence]
personality_summary: The first of many
boundaries: [Blaze trails]
default_intro: I'm the first one here
```

## Communication Style

As the first agent, I set the standard.
"#;

    let model = Arc::new(MockModel::new(vec![
        agent_yaml.to_string(),
        soul_md.to_string(),
    ]));

    let creator = AgentCreator::new(model, temp.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("pioneer", "First agent");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_ok(), "Should work with empty workspace");

    let output = result.unwrap();
    assert_eq!(output.files.len(), 2);
}

#[tokio::test]
async fn test_invalid_emoji_caught() {
    let (workspace, registry) = create_test_workspace().await;

    // Agent with invalid emoji (text instead)
    let agent_yaml = r#"id: bad-emoji-agent
name: Bad Emoji
role: Tester
avatar: ABC
personality_traits: [test]
can: [test]
cannot: [break]
skills: []
"#;

    let model = Arc::new(MockModel::single(agent_yaml.to_string()));
    let creator = AgentCreator::new(model, workspace.path().to_path_buf(), registry);
    let session = ConversationSession::new("test-session".to_string());
    let intent = create_agent_intent("test", "Bad emoji agent");

    let result = creator.handle(&intent, &session).await;
    assert!(result.is_err(), "Should fail on invalid emoji");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("emoji") || err.to_string().contains("avatar"));
}
