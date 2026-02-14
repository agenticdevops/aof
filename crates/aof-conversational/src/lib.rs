// aof-conversational - Conversational agent configuration interface
//
// This crate provides intent classification, orchestration, and conversational
// interfaces for creating and managing AOF agents through natural language.

pub mod types;
pub mod intent;
pub mod session;
pub mod orchestrator;
pub mod sanitize;

// Re-export key types for convenience
pub use types::{
    IntentType, IntentClassification, MessageRole, ConversationMessage,
    ConversationSession, OrchestratorResponse,
};
// TODO: Uncomment as implementations are completed in subsequent tasks
// pub use intent::IntentClassifier;
// pub use session::ConversationSessionStore;
// pub use orchestrator::Orchestrator;
// pub use sanitize::{sanitize_user_input, SanitizeError};
