use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Intent types recognized by the classification engine
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentType {
    /// Create a new agent with specific capabilities
    CreateAgent,
    /// Build a squad of coordinating agents
    BuildSquad,
    /// Configure scheduling and triggers
    ConfigureSchedule,
    /// Teach a new skill to the system
    TeachSkill,
    /// Intent could not be determined
    Unknown,
}

impl fmt::Display for IntentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntentType::CreateAgent => write!(f, "create_agent"),
            IntentType::BuildSquad => write!(f, "build_squad"),
            IntentType::ConfigureSchedule => write!(f, "configure_schedule"),
            IntentType::TeachSkill => write!(f, "teach_skill"),
            IntentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    /// The identified intent
    pub intent: IntentType,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Extracted parameters from user message
    pub parameters: HashMap<String, serde_json::Value>,
    /// Questions to clarify the intent
    pub clarifying_questions: Vec<String>,
}

/// Role of a message in the conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the assistant/system
    Assistant,
    /// System message (internal)
    System,
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
}

/// A conversation session tracking message history and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// Unique session identifier
    pub session_id: String,
    /// Message history in chronological order
    pub messages: Vec<ConversationMessage>,
    /// Current classified intent (if any)
    pub current_intent: Option<IntentClassification>,
    /// Files pending user confirmation (path -> content)
    pub pending_files: HashMap<String, String>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl ConversationSession {
    /// Create a new conversation session with the given ID
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            messages: Vec::new(),
            current_intent: None,
            pending_files: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Response from the orchestrator to user input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestratorResponse {
    /// System needs clarification from the user
    ClarifyingQuestions {
        /// Questions to ask the user
        questions: Vec<String>,
        /// Partially identified intent
        partial_intent: IntentType,
    },
    /// Specialist has produced a result
    SpecialistResult {
        /// Intent that was handled
        intent: IntentType,
        /// Generated files (path -> content)
        files: HashMap<String, String>,
        /// Message to display to user
        message: String,
    },
    /// Error occurred during processing
    Error {
        /// Error message
        message: String,
    },
    /// User confirmation of pending files
    Confirmation {
        /// Session ID
        session_id: String,
        /// Files to write (path -> content)
        files: HashMap<String, String>,
        /// Summary of what will be written
        summary: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_type_display() {
        assert_eq!(IntentType::CreateAgent.to_string(), "create_agent");
        assert_eq!(IntentType::BuildSquad.to_string(), "build_squad");
        assert_eq!(IntentType::ConfigureSchedule.to_string(), "configure_schedule");
        assert_eq!(IntentType::TeachSkill.to_string(), "teach_skill");
        assert_eq!(IntentType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_intent_type_serialization() {
        let intent = IntentType::CreateAgent;
        let json = serde_json::to_string(&intent).unwrap();
        assert_eq!(json, "\"create_agent\"");

        let deserialized: IntentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, IntentType::CreateAgent);
    }

    #[test]
    fn test_conversation_session_new() {
        let session = ConversationSession::new("test-123".to_string());
        assert_eq!(session.session_id, "test-123");
        assert!(session.messages.is_empty());
        assert!(session.current_intent.is_none());
        assert!(session.pending_files.is_empty());
        assert!(session.created_at <= Utc::now());
        assert_eq!(session.created_at, session.updated_at);
    }

    #[test]
    fn test_orchestrator_response_serialization() {
        let response = OrchestratorResponse::ClarifyingQuestions {
            questions: vec!["What kind of agent?".to_string()],
            partial_intent: IntentType::CreateAgent,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"type\":\"clarifying_questions\""));
        assert!(json.contains("What kind of agent?"));
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");
    }
}
