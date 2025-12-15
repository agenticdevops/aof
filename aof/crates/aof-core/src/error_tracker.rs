//! Error Tracking System with RAG (Retrieval-Augmented Generation)
//!
//! Maintains a local knowledge base of errors encountered during development,
//! enabling agents to learn from patterns and avoid recurring mistakes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

/// Error record for tracking and learning
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorRecord {
    /// Unique error identifier (hash of error type + message)
    pub id: String,

    /// Error type (MCP, Tool, Config, etc.)
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Stack trace or context
    pub context: String,

    /// When the error occurred
    pub timestamp: String,

    /// How many times this error has occurred
    pub occurrence_count: usize,

    /// Proposed fix or workaround
    pub solution: Option<String>,

    /// Whether this error has been resolved
    pub resolved: bool,

    /// Tags for categorization (e.g., "mcp", "initialization", "kubernetes")
    pub tags: Vec<String>,

    /// Related files that triggered this error
    pub related_files: Vec<String>,
}

impl ErrorRecord {
    /// Create a new error record
    pub fn new(
        error_type: &str,
        message: &str,
        context: &str,
    ) -> Self {
        // Create a simple hash from message
        let message_hash = message.chars().fold(0u32, |acc, c| {
            acc.wrapping_mul(31).wrapping_add(c as u32)
        });

        let id = format!("{}-{}",
            error_type,
            message_hash
        );

        Self {
            id,
            error_type: error_type.to_string(),
            message: message.to_string(),
            context: context.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            occurrence_count: 1,
            solution: None,
            resolved: false,
            tags: vec![],
            related_files: vec![],
        }
    }

    /// Add a tag for categorization
    pub fn with_tag(mut self, tag: &str) -> Self {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
        self
    }

    /// Add related file
    pub fn with_file(mut self, file: &str) -> Self {
        if !self.related_files.contains(&file.to_string()) {
            self.related_files.push(file.to_string());
        }
        self
    }

    /// Set a solution
    pub fn with_solution(mut self, solution: &str) -> Self {
        self.solution = Some(solution.to_string());
        self
    }

    /// Mark as resolved
    pub fn resolve(mut self) -> Self {
        self.resolved = true;
        self
    }
}

/// Local Error Knowledge Base (RAG system)
pub struct ErrorKnowledgeBase {
    /// Store of error records
    errors: HashMap<String, ErrorRecord>,

    /// Error frequency index for quick lookups
    error_index: HashMap<String, Vec<String>>, // type -> error_ids

    /// Tag index for categorization
    tag_index: HashMap<String, Vec<String>>, // tag -> error_ids
}

impl ErrorKnowledgeBase {
    /// Create a new knowledge base
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            error_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Record an error
    pub fn record(&mut self, error: ErrorRecord) {
        let error_id = error.id.clone();
        let error_type = error.error_type.clone();

        // Increment occurrence count if error already exists
        if let Some(existing) = self.errors.get_mut(&error_id) {
            existing.occurrence_count += 1;
            return;
        }

        // Index by error type
        self.error_index
            .entry(error_type)
            .or_insert_with(Vec::new)
            .push(error_id.clone());

        // Index by tags
        for tag in &error.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(error_id.clone());
        }

        // Store the error
        self.errors.insert(error_id, error);
    }

    /// Find similar errors (for learning from past mistakes)
    pub fn find_similar(&self, error_type: &str, keywords: &[&str]) -> Vec<ErrorRecord> {
        let mut matches = vec![];

        // Get errors of the same type
        if let Some(error_ids) = self.error_index.get(error_type) {
            for error_id in error_ids {
                if let Some(error) = self.errors.get(error_id) {
                    // Check if any keywords match
                    let matches_keywords = keywords.iter().any(|kw| {
                        error.message.contains(kw)
                            || error.context.contains(kw)
                            || error.tags.iter().any(|t| t.contains(kw))
                    });

                    if matches_keywords || keywords.is_empty() {
                        matches.push(error.clone());
                    }
                }
            }
        }

        // Sort by occurrence count (most frequent first)
        matches.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        matches
    }

    /// Get errors by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<ErrorRecord> {
        if let Some(error_ids) = self.tag_index.get(tag) {
            error_ids
                .iter()
                .filter_map(|id| self.errors.get(id).cloned())
                .collect()
        } else {
            vec![]
        }
    }

    /// Get all unresolved errors
    pub fn unresolved(&self) -> Vec<ErrorRecord> {
        self.errors
            .values()
            .filter(|e| !e.resolved)
            .cloned()
            .collect()
    }

    /// Get most frequent errors
    pub fn most_frequent(&self, limit: usize) -> Vec<ErrorRecord> {
        let mut errors: Vec<_> = self.errors.values().cloned().collect();
        errors.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        errors.into_iter().take(limit).collect()
    }

    /// Export to JSON for documentation/learning
    pub fn export_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.errors)
    }

    /// Get statistics about errors
    pub fn stats(&self) -> ErrorStats {
        let total_errors = self.errors.len();
        let total_occurrences = self.errors.values().map(|e| e.occurrence_count).sum();
        let unresolved_count = self.errors.values().filter(|e| !e.resolved).count();
        let with_solutions = self.errors.values().filter(|e| e.solution.is_some()).count();

        ErrorStats {
            total_unique_errors: total_errors,
            total_occurrences,
            unresolved_count,
            with_solutions,
            avg_occurrences: if total_errors > 0 { total_occurrences / total_errors } else { 0 },
        }
    }
}

impl Default for ErrorKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

/// Error statistics for monitoring and learning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_unique_errors: usize,
    pub total_occurrences: usize,
    pub unresolved_count: usize,
    pub with_solutions: usize,
    pub avg_occurrences: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_record_creation() {
        let error = ErrorRecord::new("MCP", "Client not initialized", "runtime.rs:332")
            .with_tag("initialization")
            .with_file("runtime.rs")
            .with_solution("Call client.initialize() after creation");

        assert_eq!(error.error_type, "MCP");
        assert_eq!(error.message, "Client not initialized");
        assert!(error.tags.contains(&"initialization".to_string()));
        assert!(error.solution.is_some());
    }

    #[test]
    fn test_knowledge_base_recording() {
        let mut kb = ErrorKnowledgeBase::new();

        let error1 = ErrorRecord::new("MCP", "Client not initialized", "context1")
            .with_tag("initialization");
        let error2 = ErrorRecord::new("Tool", "Tool not found", "context2")
            .with_tag("execution");

        kb.record(error1);
        kb.record(error2);

        assert_eq!(kb.errors.len(), 2);
    }

    #[test]
    fn test_find_by_tag() {
        let mut kb = ErrorKnowledgeBase::new();

        kb.record(ErrorRecord::new("MCP", "Error 1", "ctx").with_tag("init"));
        kb.record(ErrorRecord::new("MCP", "Error 2", "ctx").with_tag("execution"));
        kb.record(ErrorRecord::new("MCP", "Error 3", "ctx").with_tag("init"));

        let init_errors = kb.find_by_tag("init");
        assert_eq!(init_errors.len(), 2);
    }

    #[test]
    fn test_find_similar() {
        let mut kb = ErrorKnowledgeBase::new();

        kb.record(
            ErrorRecord::new("MCP", "Client not initialized", "runtime").with_tag("init")
        );
        kb.record(
            ErrorRecord::new("Tool", "Tool execution failed", "executor").with_tag("execution")
        );

        let similar = kb.find_similar("MCP", &["initialized"]);
        assert_eq!(similar.len(), 1);
        assert!(similar[0].message.contains("initialized"));
    }
}
