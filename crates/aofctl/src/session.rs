//! Session Management for Agent Conversations
//!
//! This module provides session persistence, allowing conversations to be saved
//! and resumed across multiple invocations.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// A saved agent conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Agent name this session belongs to
    pub agent_name: String,
    /// Model used in this session
    pub model: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// Conversation messages
    pub messages: Vec<SessionMessage>,
    /// Token usage statistics
    pub token_usage: TokenUsage,
    /// Activity log entries
    pub activity_log: Vec<ActivityLogEntry>,
}

/// A message in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    /// Role: user, assistant, error, system
    pub role: String,
    /// Message content
    pub content: String,
    /// When the message was sent
    pub timestamp: DateTime<Utc>,
    /// Token counts for this message (if available)
    pub tokens: Option<MessageTokens>,
}

/// Token counts for a single message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTokens {
    pub input: u32,
    pub output: u32,
}

/// Cumulative token usage for the session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub total_input: u32,
    pub total_output: u32,
}

/// An activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub timestamp: DateTime<Utc>,
    pub activity_type: String,
    pub message: String,
}

impl Session {
    /// Create a new session
    pub fn new(agent_name: &str, model: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_name: agent_name.to_string(),
            model: model.to_string(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            token_usage: TokenUsage::default(),
            activity_log: Vec::new(),
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, role: &str, content: &str, tokens: Option<MessageTokens>) {
        self.messages.push(SessionMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            tokens: tokens.clone(),
        });
        self.updated_at = Utc::now();

        // Update cumulative token usage
        if let Some(t) = tokens {
            self.token_usage.total_input += t.input;
            self.token_usage.total_output += t.output;
        }
    }

    /// Add an activity log entry
    pub fn add_activity(&mut self, activity_type: &str, message: &str) {
        self.activity_log.push(ActivityLogEntry {
            timestamp: Utc::now(),
            activity_type: activity_type.to_string(),
            message: message.to_string(),
        });
        self.updated_at = Utc::now();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> u32 {
        self.token_usage.total_input + self.token_usage.total_output
    }

    /// Convert messages to chat history format (role, content)
    pub fn to_chat_history(&self) -> Vec<(String, String)> {
        self.messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect()
    }
}

/// Session storage manager
pub struct SessionManager {
    /// Base directory for session storage
    base_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let base_dir = Self::get_sessions_dir()?;
        fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    /// Get the sessions directory path
    fn get_sessions_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home.join(".aof").join("sessions"))
    }

    /// Get the directory for a specific agent's sessions
    fn get_agent_dir(&self, agent_name: &str) -> PathBuf {
        self.base_dir.join(sanitize_name(agent_name))
    }

    /// Save a session to disk
    pub fn save(&self, session: &Session) -> Result<PathBuf> {
        let agent_dir = self.get_agent_dir(&session.agent_name);
        fs::create_dir_all(&agent_dir)?;

        let file_path = agent_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)?;
        fs::write(&file_path, json)?;

        // Also update the "latest" symlink/file
        let latest_path = agent_dir.join("latest.json");
        fs::write(&latest_path, serde_json::to_string_pretty(session)?)?;

        Ok(file_path)
    }

    /// Load a specific session by ID
    pub fn load(&self, agent_name: &str, session_id: &str) -> Result<Session> {
        let agent_dir = self.get_agent_dir(agent_name);
        let file_path = agent_dir.join(format!("{}.json", session_id));

        if !file_path.exists() {
            return Err(anyhow!("Session not found: {}", session_id));
        }

        let json = fs::read_to_string(&file_path)?;
        let session: Session = serde_json::from_str(&json)?;
        Ok(session)
    }

    /// Load the latest session for an agent
    pub fn load_latest(&self, agent_name: &str) -> Result<Session> {
        let agent_dir = self.get_agent_dir(agent_name);
        let latest_path = agent_dir.join("latest.json");

        if !latest_path.exists() {
            return Err(anyhow!("No sessions found for agent: {}", agent_name));
        }

        let json = fs::read_to_string(&latest_path)?;
        let session: Session = serde_json::from_str(&json)?;
        Ok(session)
    }

    /// List all sessions for an agent
    pub fn list(&self, agent_name: &str) -> Result<Vec<SessionSummary>> {
        let agent_dir = self.get_agent_dir(agent_name);

        if !agent_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();

        for entry in fs::read_dir(&agent_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip the latest.json file
            if path.file_name().map(|n| n == "latest.json").unwrap_or(false) {
                continue;
            }

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&json) {
                        // Compute values before moving fields
                        let message_count = session.messages.len();
                        let total_tokens = session.total_tokens();
                        sessions.push(SessionSummary {
                            id: session.id,
                            agent_name: session.agent_name,
                            model: session.model,
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            message_count,
                            total_tokens,
                        });
                    }
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// List all agents with sessions
    pub fn list_agents(&self) -> Result<Vec<String>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    agents.push(name.to_string());
                }
            }
        }

        agents.sort();
        Ok(agents)
    }

    /// Delete a specific session
    pub fn delete(&self, agent_name: &str, session_id: &str) -> Result<()> {
        let agent_dir = self.get_agent_dir(agent_name);
        let file_path = agent_dir.join(format!("{}.json", session_id));

        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        // If this was the latest session, remove latest.json too
        let latest_path = agent_dir.join("latest.json");
        if latest_path.exists() {
            if let Ok(json) = fs::read_to_string(&latest_path) {
                if let Ok(session) = serde_json::from_str::<Session>(&json) {
                    if session.id == session_id {
                        fs::remove_file(&latest_path)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Delete all sessions for an agent
    pub fn delete_agent_sessions(&self, agent_name: &str) -> Result<usize> {
        let agent_dir = self.get_agent_dir(agent_name);

        if !agent_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&agent_dir)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                fs::remove_file(entry.path())?;
                count += 1;
            }
        }

        // Try to remove the directory if empty
        let _ = fs::remove_dir(&agent_dir);

        Ok(count)
    }

    /// Check if a session exists
    pub fn exists(&self, agent_name: &str, session_id: &str) -> bool {
        let agent_dir = self.get_agent_dir(agent_name);
        let file_path = agent_dir.join(format!("{}.json", session_id));
        file_path.exists()
    }

    /// Check if latest session exists
    pub fn has_latest(&self, agent_name: &str) -> bool {
        let agent_dir = self.get_agent_dir(agent_name);
        let latest_path = agent_dir.join("latest.json");
        latest_path.exists()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create session manager")
    }
}

/// Summary information about a session (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub agent_name: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub total_tokens: u32,
}

impl SessionSummary {
    /// Format for display
    pub fn format_display(&self) -> String {
        let age = Utc::now().signed_duration_since(self.updated_at);
        let age_str = if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else if age.num_minutes() > 0 {
            format!("{}m ago", age.num_minutes())
        } else {
            "just now".to_string()
        };

        format!(
            "{} | {} msgs | {} tokens | {}",
            &self.id[..8],
            self.message_count,
            self.total_tokens,
            age_str
        )
    }
}

/// Sanitize agent name for use as a directory name
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_manager() -> (SessionManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager {
            base_dir: temp_dir.path().to_path_buf(),
        };
        (manager, temp_dir)
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new("test-agent", "google:gemini-2.5-flash");
        assert_eq!(session.agent_name, "test-agent");
        assert_eq!(session.model, "google:gemini-2.5-flash");
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut session = Session::new("test-agent", "google:gemini-2.5-flash");
        session.add_message("user", "Hello", Some(MessageTokens { input: 10, output: 0 }));
        session.add_message(
            "assistant",
            "Hi there!",
            Some(MessageTokens { input: 0, output: 20 }),
        );

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.token_usage.total_input, 10);
        assert_eq!(session.token_usage.total_output, 20);
    }

    #[test]
    fn test_save_and_load() {
        let (manager, _temp) = test_manager();

        let mut session = Session::new("test-agent", "google:gemini-2.5-flash");
        session.add_message("user", "Hello", None);
        session.add_message("assistant", "Hi!", None);

        let session_id = session.id.clone();

        // Save
        manager.save(&session).unwrap();

        // Load
        let loaded = manager.load("test-agent", &session_id).unwrap();
        assert_eq!(loaded.id, session_id);
        assert_eq!(loaded.messages.len(), 2);
    }

    #[test]
    fn test_load_latest() {
        let (manager, _temp) = test_manager();

        let mut session = Session::new("test-agent", "google:gemini-2.5-flash");
        session.add_message("user", "Test", None);
        manager.save(&session).unwrap();

        let latest = manager.load_latest("test-agent").unwrap();
        assert_eq!(latest.id, session.id);
    }

    #[test]
    fn test_list_sessions() {
        let (manager, _temp) = test_manager();

        // Create multiple sessions
        for i in 0..3 {
            let mut session = Session::new("test-agent", "google:gemini-2.5-flash");
            session.add_message("user", &format!("Message {}", i), None);
            manager.save(&session).unwrap();
        }

        let sessions = manager.list("test-agent").unwrap();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_delete_session() {
        let (manager, _temp) = test_manager();

        let mut session = Session::new("test-agent", "google:gemini-2.5-flash");
        session.add_message("user", "Test", None);
        let session_id = session.id.clone();
        manager.save(&session).unwrap();

        assert!(manager.exists("test-agent", &session_id));
        manager.delete("test-agent", &session_id).unwrap();
        assert!(!manager.exists("test-agent", &session_id));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("my-agent"), "my-agent");
        assert_eq!(sanitize_name("my agent"), "my_agent");
        assert_eq!(sanitize_name("agent@v1"), "agent_v1");
    }
}
