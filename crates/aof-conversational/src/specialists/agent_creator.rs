//! AgentCreator specialist - generates AGENTS.md + SOUL.md from natural language

use async_trait::async_trait;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, warn};

use aof_core::{Model, ModelRequest, RequestMessage};
use aof_core::model::MessageRole;
use aof_personas::loader::AgentLoader;
use aof_personas::types::Agent;
use aof_skills::SkillRegistry;

use crate::generation::{
    build_agent_generation_prompt, build_soul_generation_prompt, format_agent_yaml,
    format_soul_markdown, parse_agent_yaml, parse_soul_markdown,
};
use crate::specialists::traits::{Specialist, SpecialistOutput};
use crate::types::{ConversationSession, IntentClassification};
use crate::validation::{validate_generated_agent, GenerationError};

/// Maximum retries for LLM generation
const MAX_GENERATION_RETRIES: usize = 2;

/// AgentCreator specialist - converts agent descriptions into validated YAML + personality
pub struct AgentCreator {
    model: Arc<dyn Model>,
    workspace_path: PathBuf,
    skill_registry: Arc<SkillRegistry>,
}

impl AgentCreator {
    /// Create a new AgentCreator
    pub fn new(
        model: Arc<dyn Model>,
        workspace_path: PathBuf,
        skill_registry: Arc<SkillRegistry>,
    ) -> Self {
        Self {
            model,
            workspace_path,
            skill_registry,
        }
    }

    /// Load context: existing agents and available skills
    async fn load_context(&self) -> Result<(Vec<Agent>, Vec<String>)> {
        // Load existing agents
        let agents_path = self.workspace_path.join("AGENTS.md");
        let existing_agents = if agents_path.exists() {
            AgentLoader::load_from_file(&agents_path.to_string_lossy()).await?
        } else {
            debug!("No existing AGENTS.md, starting fresh");
            Vec::new()
        };

        // Load available skills from registry
        let skills = self.skill_registry.all().await;
        let skill_names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

        debug!(
            "Loaded context: {} existing agents, {} available skills",
            existing_agents.len(),
            skill_names.len()
        );

        Ok((existing_agents, skill_names))
    }

    /// Generate agent YAML from description
    async fn generate_agent(
        &self,
        intent: &IntentClassification,
        available_skills: &[String],
    ) -> Result<Agent> {
        // Extract parameters
        let agent_type = intent
            .parameters
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("assistant");

        let skills: Vec<String> = intent
            .parameters
            .get("skills")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let description = intent
            .parameters
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Build prompt with available skills to prevent hallucination
        let prompt =
            build_agent_generation_prompt(agent_type, &skills, description, available_skills);

        // Try generation with retries
        for attempt in 1..=MAX_GENERATION_RETRIES {
            debug!("Agent generation attempt {}/{}", attempt, MAX_GENERATION_RETRIES);

            let request = ModelRequest {
                messages: vec![RequestMessage {
                    role: MessageRole::User,
                    content: prompt.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                }],
                system: Some("You are an expert at creating agent configurations. Output ONLY valid YAML with no additional explanation.".to_string()),
                tools: vec![],
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false,
                extra: HashMap::new(),
            };

            let response = self.model.generate(&request).await?;

            // Try to parse the generated YAML
            match parse_agent_yaml(&response.content) {
                Ok(agent) => {
                    debug!("Successfully parsed agent YAML: {}", agent.id);
                    return Ok(agent);
                }
                Err(e) => {
                    warn!("Failed to parse YAML on attempt {}: {}", attempt, e);
                    if attempt == MAX_GENERATION_RETRIES {
                        return Err(anyhow!("Failed to generate valid YAML after {} attempts: {}", MAX_GENERATION_RETRIES, e));
                    }
                }
            }
        }

        Err(anyhow!("Failed to generate agent after retries"))
    }

    /// Generate SOUL.md personality for the agent
    async fn generate_soul(&self, agent: &Agent, description: &str) -> Result<String> {
        let prompt = build_soul_generation_prompt(
            &agent.id,
            &agent.role,
            &agent.personality_traits,
            description,
        );

        let request = ModelRequest {
            messages: vec![RequestMessage {
                role: MessageRole::User,
                content: prompt,
                tool_calls: None,
                tool_call_id: None,
            }],
            system: Some("You are an expert at creating agent personalities. Follow the SOUL.md format exactly.".to_string()),
            tools: vec![],
            temperature: Some(0.8),
            max_tokens: Some(3000),
            stream: false,
            extra: HashMap::new(),
        };

        let response = self.model.generate(&request).await?;

        // Parse and validate the SOUL content
        let soul = parse_soul_markdown(&response.content, &agent.id)?;

        // Format back to markdown
        let formatted = format_soul_markdown(&soul)?;

        Ok(formatted)
    }

    /// Validate generated agent with retry on skill hallucination
    async fn validate_with_retry(
        &self,
        mut agent: Agent,
        existing_agents: &[Agent],
        available_skills: &[String],
    ) -> Result<Agent> {
        match validate_generated_agent(&agent, existing_agents, available_skills) {
            Ok(()) => Ok(agent),
            Err(errors) => {
                // Check if we have skill hallucination errors
                let has_skill_errors = errors
                    .iter()
                    .any(|e| matches!(e, GenerationError::SkillNotFound { .. }));

                if has_skill_errors {
                    warn!("Skill hallucination detected, attempting fix");

                    // Remove hallucinated skills and use only valid ones
                    agent.skills.retain(|s| available_skills.contains(s));

                    // If we removed all skills, this is an error
                    if agent.skills.is_empty() {
                        return Err(anyhow!(
                            "All generated skills were invalid. Available skills: {:?}",
                            available_skills
                        ));
                    }

                    // Validate again with corrected skills
                    validate_generated_agent(&agent, existing_agents, available_skills)
                        .map_err(|errors| {
                            anyhow!(
                                "Validation failed after skill correction: {}",
                                format_validation_errors(&errors)
                            )
                        })?;

                    Ok(agent)
                } else {
                    // Other validation errors - fail immediately
                    Err(anyhow!(
                        "Agent validation failed: {}",
                        format_validation_errors(&errors)
                    ))
                }
            }
        }
    }
}

#[async_trait]
impl Specialist for AgentCreator {
    async fn handle(
        &self,
        intent: &IntentClassification,
        _session: &ConversationSession,
    ) -> Result<SpecialistOutput> {
        debug!("AgentCreator handling intent: {:?}", intent.intent);

        // Load context
        let (existing_agents, available_skills) = self.load_context().await?;

        // Generate agent YAML
        let agent = self.generate_agent(intent, &available_skills).await?;

        // Validate and fix if needed
        let validated_agent = self
            .validate_with_retry(agent, &existing_agents, &available_skills)
            .await?;

        // Generate SOUL.md personality
        let description = intent
            .parameters
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let soul_content = self.generate_soul(&validated_agent, description).await?;

        // Format agent YAML
        let agent_yaml = format_agent_yaml(&validated_agent)?;

        // Build file map
        let mut files = HashMap::new();
        files.insert(
            self.workspace_path
                .join("AGENTS.md")
                .to_string_lossy()
                .to_string(),
            agent_yaml,
        );
        files.insert(
            self.workspace_path
                .join("SOUL.md")
                .to_string_lossy()
                .to_string(),
            soul_content,
        );

        // Build response message
        let message = format!(
            "I've created a new agent configuration:\n\n\
            **Agent:** {} ({})\n\
            **Role:** {}\n\
            **Skills:** {}\n\
            **Personality:** {}\n\n\
            Review the files below and confirm to add this agent to your workspace.",
            validated_agent.name,
            validated_agent.id,
            validated_agent.role,
            validated_agent.skills.join(", "),
            validated_agent.personality_traits.join(", ")
        );

        Ok(SpecialistOutput::with_confirmation(files, message))
    }

    fn name(&self) -> &str {
        "agent_creator"
    }
}

/// Format validation errors for user-friendly display
fn format_validation_errors(errors: &[GenerationError]) -> String {
    errors
        .iter()
        .map(|e| format!("- {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{ModelResponse, StopReason, Usage, AofResult, ModelConfig, ModelProvider, StreamChunk};

    /// Mock Model for testing
    struct MockModel {
        responses: Vec<String>,
        current: std::sync::atomic::AtomicUsize,
    }

    impl MockModel {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                current: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl Model for MockModel {
        async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
            let idx = self
                .current
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let content = self.responses.get(idx).cloned().unwrap_or_default();

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

    #[tokio::test]
    async fn test_format_validation_errors() {
        let errors = vec![
            GenerationError::MissingField("name".to_string()),
            GenerationError::InvalidAgentId("TestAgent".to_string()),
        ];

        let formatted = format_validation_errors(&errors);
        assert!(formatted.contains("Missing required field: name"));
        assert!(formatted.contains("Invalid agent ID format"));
    }
}
