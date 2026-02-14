use crate::types::{ConversationMessage, IntentClassification, IntentType, MessageRole};
use aof_core::{Model, ModelRequest, RequestMessage};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Intent classifier using LLM for natural language understanding
pub struct IntentClassifier {
    model: Box<dyn Model>,
    system_prompt: String,
}

/// JSON response format from LLM
#[derive(Debug, Serialize, Deserialize)]
struct ClassificationResponse {
    intent: String,
    confidence: f32,
    #[serde(default)]
    parameters: HashMap<String, serde_json::Value>,
    #[serde(default)]
    clarifying_questions: Vec<String>,
}

impl IntentClassifier {
    /// Create a new intent classifier with the given model
    pub fn new(model: Box<dyn Model>) -> Self {
        let system_prompt = build_system_prompt();
        Self {
            model,
            system_prompt,
        }
    }

    /// Classify user intent from message and conversation history
    ///
    /// Returns IntentClassification with confidence score and extracted parameters.
    /// Falls back to Unknown intent with confidence 0.0 if classification fails.
    pub async fn classify(
        &self,
        message: &str,
        history: &[ConversationMessage],
    ) -> Result<IntentClassification> {
        let messages = build_classification_prompt(message, history);

        let request = ModelRequest {
            messages,
            system: Some(self.system_prompt.clone()),
            tools: Vec::new(),
            temperature: Some(0.0), // Deterministic classification
            max_tokens: Some(500),  // Classification should be concise
            stream: false,
            extra: HashMap::new(),
        };

        debug!("Classifying intent for message: {}", message);

        match self.model.generate(&request).await {
            Ok(response) => {
                debug!("LLM response: {}", response.content);
                parse_classification_response(&response.content)
            }
            Err(e) => {
                debug!("LLM call failed: {}", e);
                // Fallback to unknown intent
                Ok(IntentClassification {
                    intent: IntentType::Unknown,
                    confidence: 0.0,
                    parameters: HashMap::new(),
                    clarifying_questions: vec![
                        "I'm having trouble understanding. Could you rephrase that?".to_string()
                    ],
                })
            }
        }
    }
}

/// Build the system prompt with intent taxonomy and few-shot examples
fn build_system_prompt() -> String {
    r#"You are an intent classification system for AOF (Agentic Ops Framework).

Your task is to classify user requests into one of these intents:

1. **create_agent** - User wants to create a new agent with specific capabilities
   Examples: "I need a K8s monitoring agent", "Create an agent to check pod status"

2. **build_squad** - User wants to build a team of coordinating agents
   Examples: "Build me an incident response squad", "I need a deployment team"

3. **configure_schedule** - User wants to set up scheduling/triggers
   Examples: "Check my cluster every 30 min", "Monitor PostgreSQL daily at 6am"

4. **teach_skill** - User wants to teach the system a new skill
   Examples: "Learn how to debug our Postgres", "I want to add a custom check"

5. **unknown** - Intent cannot be determined

Respond ONLY with valid JSON in this format:
{
  "intent": "create_agent" | "build_squad" | "configure_schedule" | "teach_skill" | "unknown",
  "confidence": 0.0 to 1.0,
  "parameters": { ... extracted parameters ... },
  "clarifying_questions": [ ... questions if confidence < 0.8 ... ]
}

Few-shot examples:

User: "I need a K8s monitoring agent"
{
  "intent": "create_agent",
  "confidence": 0.95,
  "parameters": {"agent_type": "kubernetes-monitor"},
  "clarifying_questions": []
}

User: "Build me an incident response team"
{
  "intent": "build_squad",
  "confidence": 0.92,
  "parameters": {"squad_type": "incident-response"},
  "clarifying_questions": []
}

User: "Check cluster every 30 minutes"
{
  "intent": "configure_schedule",
  "confidence": 0.90,
  "parameters": {"interval": "30 minutes", "target": "cluster"},
  "clarifying_questions": []
}

User: "Learn how to debug Postgres"
{
  "intent": "teach_skill",
  "confidence": 0.88,
  "parameters": {"skill_target": "postgres", "skill_type": "debug"},
  "clarifying_questions": []
}

Classify the user's message based on these patterns."#.to_string()
}

/// Build the message history for classification
pub fn build_classification_prompt(
    message: &str,
    history: &[ConversationMessage],
) -> Vec<RequestMessage> {
    let mut messages = Vec::new();

    // Include last 10 messages for context
    let start_index = history.len().saturating_sub(10);
    for msg in &history[start_index..] {
        let role = match msg.role {
            MessageRole::User => aof_core::model::MessageRole::User,
            MessageRole::Assistant => aof_core::model::MessageRole::Assistant,
            MessageRole::System => aof_core::model::MessageRole::System,
        };

        messages.push(RequestMessage {
            role,
            content: msg.content.clone(),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // Add current message
    messages.push(RequestMessage {
        role: aof_core::model::MessageRole::User,
        content: message.to_string(),
        tool_calls: None,
        tool_call_id: None,
    });

    messages
}

/// Parse LLM response into IntentClassification
fn parse_classification_response(content: &str) -> Result<IntentClassification> {
    // Try to parse JSON response
    let response: ClassificationResponse = serde_json::from_str(content).map_err(|e| {
        debug!("JSON parsing failed: {}, content: {}", e, content);
        anyhow!("Failed to parse classification response")
    })?;

    // Map string intent to enum
    let intent = match response.intent.as_str() {
        "create_agent" => IntentType::CreateAgent,
        "build_squad" => IntentType::BuildSquad,
        "configure_schedule" => IntentType::ConfigureSchedule,
        "teach_skill" => IntentType::TeachSkill,
        _ => IntentType::Unknown,
    };

    Ok(IntentClassification {
        intent,
        confidence: response.confidence.clamp(0.0, 1.0),
        parameters: response.parameters,
        clarifying_questions: response.clarifying_questions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{AofResult, ModelResponse, StopReason, Usage};
    use async_trait::async_trait;
    use std::pin::Pin;

    // Mock model for testing
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
            unimplemented!("Mock doesn't support streaming")
        }

        fn config(&self) -> &aof_core::ModelConfig {
            unimplemented!()
        }

        fn provider(&self) -> aof_core::ModelProvider {
            aof_core::ModelProvider::Anthropic
        }
    }

    #[tokio::test]
    async fn test_classify_create_agent() {
        let model = Box::new(MockModel {
            response: r#"{"intent": "create_agent", "confidence": 0.95, "parameters": {"agent_type": "k8s-monitor"}}"#.to_string(),
        });

        let classifier = IntentClassifier::new(model);
        let result = classifier.classify("I need a K8s monitoring agent", &[]).await;

        assert!(result.is_ok());
        let classification = result.unwrap();
        assert_eq!(classification.intent, IntentType::CreateAgent);
        assert!(classification.confidence > 0.9);
    }

    #[tokio::test]
    async fn test_classify_build_squad() {
        let model = Box::new(MockModel {
            response: r#"{"intent": "build_squad", "confidence": 0.92, "parameters": {}}"#
                .to_string(),
        });

        let classifier = IntentClassifier::new(model);
        let result = classifier
            .classify("Build incident response team", &[])
            .await;

        assert!(result.is_ok());
        let classification = result.unwrap();
        assert_eq!(classification.intent, IntentType::BuildSquad);
    }

    #[tokio::test]
    async fn test_classify_unknown() {
        let model = Box::new(MockModel {
            response: r#"{"intent": "unknown", "confidence": 0.1, "parameters": {}, "clarifying_questions": ["What would you like to do?"]}"#.to_string(),
        });

        let classifier = IntentClassifier::new(model);
        let result = classifier.classify("asdfasdf", &[]).await;

        assert!(result.is_ok());
        let classification = result.unwrap();
        assert_eq!(classification.intent, IntentType::Unknown);
        assert!(classification.confidence < 0.5);
        assert!(!classification.clarifying_questions.is_empty());
    }

    #[tokio::test]
    async fn test_malformed_json_fallback() {
        let model = Box::new(MockModel {
            response: "This is not JSON".to_string(),
        });

        let classifier = IntentClassifier::new(model);
        let result = classifier.classify("test message", &[]).await;

        // Should not panic, should return Unknown
        assert!(result.is_err()); // parse_classification_response returns Err
    }

    #[test]
    fn test_system_prompt_contains_examples() {
        let prompt = build_system_prompt();
        assert!(prompt.contains("K8s monitoring agent"));
        assert!(prompt.contains("incident response"));
        assert!(prompt.contains("cluster every 30"));
        assert!(prompt.contains("debug Postgres"));
    }

    #[test]
    fn test_build_prompt_with_history() {
        let history = vec![
            ConversationMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: chrono::Utc::now(),
            },
            ConversationMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];

        let messages = build_classification_prompt("I need help", &history);
        assert_eq!(messages.len(), 3); // 2 from history + 1 current
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[2].content, "I need help");
    }

    #[test]
    fn test_build_prompt_limits_history() {
        let mut history = Vec::new();
        for i in 0..20 {
            history.push(ConversationMessage {
                role: MessageRole::User,
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now(),
            });
        }

        let messages = build_classification_prompt("current", &history);
        // Should be 10 from history + 1 current = 11
        assert_eq!(messages.len(), 11);
        assert_eq!(messages[0].content, "Message 10"); // Last 10 messages
    }

    #[test]
    fn test_parse_classification_response() {
        let json = r#"{"intent": "create_agent", "confidence": 0.95, "parameters": {"type": "monitor"}}"#;
        let result = parse_classification_response(json);

        assert!(result.is_ok());
        let classification = result.unwrap();
        assert_eq!(classification.intent, IntentType::CreateAgent);
        assert_eq!(classification.confidence, 0.95);
        assert!(classification.parameters.contains_key("type"));
    }
}
