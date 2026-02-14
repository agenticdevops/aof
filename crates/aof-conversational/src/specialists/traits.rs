use crate::types::{IntentClassification, ConversationSession};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Specialist handler trait - converts intents into generated files
#[async_trait]
pub trait Specialist: Send + Sync {
    /// Handle a classified intent and return generated files + response message
    async fn handle(
        &self,
        intent: &IntentClassification,
        session: &ConversationSession,
    ) -> Result<SpecialistOutput>;

    /// Name of this specialist (for logging)
    fn name(&self) -> &str;
}

/// Output from a specialist handler
pub struct SpecialistOutput {
    /// Files to be created/modified (path -> content)
    pub files: HashMap<String, String>,
    /// Response message to user
    pub message: String,
    /// Should user confirm before writing?
    pub requires_confirmation: bool,
}

impl SpecialistOutput {
    /// Create a new specialist output
    pub fn new(files: HashMap<String, String>, message: String, requires_confirmation: bool) -> Self {
        Self {
            files,
            message,
            requires_confirmation,
        }
    }

    /// Create output that requires confirmation
    pub fn with_confirmation(files: HashMap<String, String>, message: String) -> Self {
        Self::new(files, message, true)
    }

    /// Create output that doesn't require confirmation
    pub fn without_confirmation(files: HashMap<String, String>, message: String) -> Self {
        Self::new(files, message, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_output_construction() {
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), "content".to_string());

        let output = SpecialistOutput::with_confirmation(
            files.clone(),
            "Test message".to_string()
        );

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.message, "Test message");
        assert!(output.requires_confirmation);

        let output2 = SpecialistOutput::without_confirmation(
            files,
            "No confirm".to_string()
        );
        assert!(!output2.requires_confirmation);
    }
}
