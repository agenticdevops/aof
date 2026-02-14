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
// TODO: Uncomment as types are implemented in subsequent tasks
// pub use types::{
//     IntentType, IntentClassification, MessageRole, ConversationMessage,
//     ConversationSession, OrchestratorResponse,
// };
// pub use intent::IntentClassifier;
// pub use session::ConversationSessionStore;
// pub use orchestrator::Orchestrator;
// pub use sanitize::{sanitize_user_input, SanitizeError};
