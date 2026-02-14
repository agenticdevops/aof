use async_trait::async_trait;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;

use aof_core::Model;
use aof_personas::AgentLoader;

use crate::schedule::{parse_natural_schedule, parse_with_llm, ScheduleError};
use crate::specialists::traits::{Specialist, SpecialistOutput};
use crate::types::{ConversationSession, IntentClassification};

/// Scheduler specialist for configuring agent triggers
pub struct Scheduler {
    model: Arc<dyn Model>,
    workspace_path: PathBuf,
}

impl Scheduler {
    /// Create a new scheduler specialist
    pub fn new(model: Arc<dyn Model>, workspace_path: PathBuf) -> Self {
        Self {
            model,
            workspace_path,
        }
    }

    /// Resolve which agent this schedule is for
    ///
    /// If agent_id is specified in parameters, verify it exists.
    /// If not specified, list available agents for user selection.
    async fn resolve_agent(&self, intent: &IntentClassification) -> Result<Option<String>> {
        // Check if agent_id was specified
        if let Some(agent_id) = intent.parameters.get("agent_id").and_then(|v| v.as_str()) {
            // Verify agent exists
            let agents_path = self.workspace_path.join("AGENTS.md");
            let agents = AgentLoader::load_from_file(agents_path.to_str().unwrap())
                .await
                .map_err(|e| anyhow!("Failed to load agents: {}", e))?;

            if agents.iter().any(|a| a.id == agent_id) {
                Ok(Some(agent_id.to_string()))
            } else {
                Err(anyhow!("Agent '{}' not found in AGENTS.md", agent_id))
            }
        } else {
            // No agent specified - will need to list options
            Ok(None)
        }
    }

    /// Format trigger configuration as YAML
    fn format_trigger_config(&self, agent_id: &str, schedule: &crate::schedule::ParsedSchedule) -> String {
        format!(
            r#"# Trigger configuration for {}
# Add this to your triggers.yaml file

schedules:
  - id: {}-schedule
    agent_id: {}
    trigger:
      type: Schedule
      schedule: "{}"
      timezone: "{}"
    description: "{}"
"#,
            agent_id,
            agent_id,
            agent_id,
            schedule.cron_expression,
            schedule.timezone,
            schedule.description
        )
    }

    /// Format the next runs in human-readable format
    fn format_next_runs(&self, schedule: &crate::schedule::ParsedSchedule) -> String {
        schedule
            .next_runs
            .iter()
            .enumerate()
            .map(|(i, dt)| {
                let formatted = dt.format("%a %b %d, %Y at %I:%M %p %Z");
                format!("{}. {}", i + 1, formatted)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl Specialist for Scheduler {
    async fn handle(
        &self,
        intent: &IntentClassification,
        _session: &ConversationSession,
    ) -> Result<SpecialistOutput> {
        // Extract schedule description
        let schedule_desc = intent
            .parameters
            .get("schedule")
            .or_else(|| intent.parameters.get("description"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No schedule description provided"))?;

        // Try regex-based parsing first (fast path, no LLM call)
        let schedule = match parse_natural_schedule(schedule_desc) {
            Ok(s) => s,
            Err(ScheduleError::UnparsableInput(_)) => {
                // Fall back to LLM parsing for complex patterns
                parse_with_llm(schedule_desc, self.model.as_ref())
                    .await
                    .map_err(|e| match e {
                        ScheduleError::UnparsableInput(msg) => {
                            anyhow!("Could not parse schedule: {}. Try patterns like:\n\
                                 - 'every 30 minutes'\n\
                                 - 'daily at 6am EST'\n\
                                 - 'every weekday at 9am'\n\
                                 - 'business hours'", msg)
                        }
                        ScheduleError::InvalidCron(msg) => {
                            anyhow!("Invalid cron expression: {}", msg)
                        }
                        ScheduleError::InvalidTimezone(msg) => {
                            anyhow!("Invalid timezone: {}", msg)
                        }
                        ScheduleError::LlmError(msg) => {
                            anyhow!("LLM parsing failed: {}", msg)
                        }
                    })?
            }
            Err(e) => {
                return Err(anyhow!("Schedule parsing failed: {}", e))
            }
        };

        // Resolve agent
        let agent_id = match self.resolve_agent(intent).await? {
            Some(id) => id,
            None => {
                // No agent specified - list available agents
                let agents_path = self.workspace_path.join("AGENTS.md");
                let agents = AgentLoader::load_from_file(agents_path.to_str().unwrap())
                    .await
                    .map_err(|e| anyhow!("Failed to load agents: {}", e))?;

                let agent_list = agents
                    .iter()
                    .map(|a| format!("  - {}", a.id))
                    .collect::<Vec<_>>()
                    .join("\n");

                return Ok(SpecialistOutput::without_confirmation(
                    std::collections::HashMap::new(),
                    format!(
                        "Which agent should run on this schedule?\n\nAvailable agents:\n{}",
                        agent_list
                    ),
                ));
            }
        };

        // Generate trigger config
        let trigger_yaml = self.format_trigger_config(&agent_id, &schedule);
        let next_runs = self.format_next_runs(&schedule);

        // Build file map
        let mut files = std::collections::HashMap::new();
        files.insert("triggers.yaml".to_string(), trigger_yaml.clone());

        // Build response message
        let message = format!(
            "Schedule configured for agent '{}':\n\n\
            Cron expression: {}\n\
            Timezone: {}\n\n\
            Next 3 scheduled runs:\n{}\n\n\
            Review the generated triggers.yaml configuration.",
            agent_id, schedule.cron_expression, schedule.timezone, next_runs
        );

        Ok(SpecialistOutput::with_confirmation(files, message))
    }

    fn name(&self) -> &str {
        "scheduler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{AofResult, ModelProvider, ModelRequest, ModelResponse, StopReason, Usage};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::pin::Pin;
    use tempfile::TempDir;

    struct MockModel {
        response: String,
    }

    #[async_trait]
    impl Model for MockModel {
        async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
            Ok(ModelResponse {
                content: self.response.clone(),
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                tool_calls: Vec::new(),
                metadata: HashMap::new(),
            })
        }

        async fn generate_stream(
            &self,
            _request: &ModelRequest,
        ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<aof_core::StreamChunk>> + Send>>>
        {
            unimplemented!()
        }

        fn config(&self) -> &aof_core::ModelConfig {
            unimplemented!()
        }

        fn provider(&self) -> ModelProvider {
            ModelProvider::Anthropic
        }
    }

    fn setup_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let agents_content = r#"agents:
  - id: k8s-monitor
    name: "Kubernetes Monitor"
    role: "Monitor cluster health"
    avatar: "🔍"
"#;
        std::fs::write(temp_dir.path().join("AGENTS.md"), agents_content).unwrap();
        temp_dir
    }

    #[tokio::test]
    async fn test_scheduler_full_flow() {
        let temp_dir = setup_test_workspace();
        let model = Arc::new(MockModel {
            response: String::new(), // Won't be called for simple pattern
        });

        let scheduler = Scheduler::new(model, temp_dir.path().to_path_buf());

        let mut params = HashMap::new();
        params.insert(
            "schedule".to_string(),
            serde_json::Value::String("every 30 minutes".to_string()),
        );
        params.insert(
            "agent_id".to_string(),
            serde_json::Value::String("k8s-monitor".to_string()),
        );

        let intent = IntentClassification {
            intent: crate::types::IntentType::ConfigureSchedule,
            confidence: 0.95,
            parameters: params,
            clarifying_questions: vec![],
        };

        let session = crate::types::ConversationSession::new("test-session".to_string());

        let result = scheduler.handle(&intent, &session).await.unwrap();

        assert!(result.requires_confirmation);
        assert!(result.files.contains_key("triggers.yaml"));
        assert!(result.message.contains("k8s-monitor"));
        assert!(result.message.contains("0 */30 * * * *"));
        assert!(result.message.contains("Next 3 scheduled runs"));
    }

    #[tokio::test]
    async fn test_scheduler_unknown_agent() {
        let temp_dir = setup_test_workspace();
        let model = Arc::new(MockModel {
            response: String::new(),
        });

        let scheduler = Scheduler::new(model, temp_dir.path().to_path_buf());

        let mut params = HashMap::new();
        params.insert(
            "schedule".to_string(),
            serde_json::Value::String("daily at 6am".to_string()),
        );
        params.insert(
            "agent_id".to_string(),
            serde_json::Value::String("unknown-agent".to_string()),
        );

        let intent = IntentClassification {
            intent: crate::types::IntentType::ConfigureSchedule,
            confidence: 0.95,
            parameters: params,
            clarifying_questions: vec![],
        };

        let session = crate::types::ConversationSession::new("test-session".to_string());

        let result = scheduler.handle(&intent, &session).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_scheduler_no_agent_specified() {
        let temp_dir = setup_test_workspace();
        let model = Arc::new(MockModel {
            response: String::new(),
        });

        let scheduler = Scheduler::new(model, temp_dir.path().to_path_buf());

        let mut params = HashMap::new();
        params.insert(
            "schedule".to_string(),
            serde_json::Value::String("business hours".to_string()),
        );
        // No agent_id parameter

        let intent = IntentClassification {
            intent: crate::types::IntentType::ConfigureSchedule,
            confidence: 0.95,
            parameters: params,
            clarifying_questions: vec![],
        };

        let session = crate::types::ConversationSession::new("test-session".to_string());

        let result = scheduler.handle(&intent, &session).await.unwrap();

        assert!(!result.requires_confirmation);
        assert!(result.message.contains("Which agent"));
        assert!(result.message.contains("k8s-monitor"));
    }

    #[tokio::test]
    async fn test_trigger_config_yaml_valid() {
        let temp_dir = setup_test_workspace();
        let model = Arc::new(MockModel {
            response: String::new(),
        });

        let scheduler = Scheduler::new(model, temp_dir.path().to_path_buf());

        let mut params = HashMap::new();
        params.insert(
            "schedule".to_string(),
            serde_json::Value::String("every weekday at 9am".to_string()),
        );
        params.insert(
            "agent_id".to_string(),
            serde_json::Value::String("k8s-monitor".to_string()),
        );

        let intent = IntentClassification {
            intent: crate::types::IntentType::ConfigureSchedule,
            confidence: 0.95,
            parameters: params,
            clarifying_questions: vec![],
        };

        let session = crate::types::ConversationSession::new("test-session".to_string());

        let result = scheduler.handle(&intent, &session).await.unwrap();

        let yaml_content = result.files.get("triggers.yaml").unwrap();

        // Parse YAML to ensure it's valid
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        assert!(parsed.get("schedules").is_some());
    }
}
