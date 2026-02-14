//! Session state persistence using aof-memory FileBackend
//!
//! Provides save/restore functionality for SessionState, allowing agent coordination
//! state to survive daemon restarts.

use aof_core::{AofError, AofResult, Memory, SessionState};
use aof_memory::SimpleMemory;
use std::path::PathBuf;

/// Session state persistence manager
///
/// Uses SimpleMemory with FileBackend to store session state as JSON.
/// Each session is stored with its session_id as the key.
pub struct SessionPersistence {
    memory: SimpleMemory,
}

impl SessionPersistence {
    /// Create a new session persistence manager
    ///
    /// Stores session state in `persist_dir/session-state.json`
    ///
    /// # Arguments
    /// * `persist_dir` - Directory where session state file will be created
    pub async fn new(persist_dir: PathBuf) -> AofResult<Self> {
        // Create file backend at persist_dir/session-state.json
        let memory = SimpleMemory::file(persist_dir.join("session-state.json")).await?;
        Ok(Self { memory })
    }

    /// Save a session state
    ///
    /// Serializes SessionState to JSON and stores it with session_id as key.
    ///
    /// # Arguments
    /// * `state` - The session state to save
    pub async fn save_session(&self, state: &SessionState) -> AofResult<()> {
        let value = serde_json::to_value(state)
            .map_err(|e| AofError::memory(format!("Failed to serialize session state: {}", e)))?;

        self.memory
            .store(&state.session_id, value)
            .await?;

        Ok(())
    }

    /// Restore a session state by session ID
    ///
    /// Returns None if the session doesn't exist.
    ///
    /// # Arguments
    /// * `session_id` - The session ID to restore
    pub async fn restore_session(&self, session_id: &str) -> AofResult<Option<SessionState>> {
        let entry = self.memory.retrieve(session_id).await?;

        match entry {
            Some(value) => {
                let state: SessionState = serde_json::from_value(value)
                    .map_err(|e| AofError::memory(format!("Failed to deserialize session state: {}", e)))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// List all session IDs
    ///
    /// Returns a vector of session IDs currently stored.
    pub async fn list_sessions(&self) -> AofResult<Vec<String>> {
        let keys = self.memory.list_keys().await?;
        Ok(keys)
    }

    /// Delete a session
    ///
    /// Removes the session state from storage.
    ///
    /// # Arguments
    /// * `session_id` - The session ID to delete
    pub async fn delete_session(&self, session_id: &str) -> AofResult<()> {
        self.memory.delete(session_id).await
    }

    /// Clear all sessions
    ///
    /// Removes all stored session state.
    pub async fn clear_all(&self) -> AofResult<()> {
        self.memory.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{AgentState, AgentStatus, TaskInfo};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_restore_session() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = SessionPersistence::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let mut state = SessionState::new("session-123");
        state.update_agent(
            "agent-1".to_string(),
            AgentState::new("agent-1", AgentStatus::Running),
        );
        state.add_task(TaskInfo::new("task-1", "Process data"));

        // Save session
        persistence.save_session(&state).await.unwrap();

        // Restore session
        let restored = persistence
            .restore_session("session-123")
            .await
            .unwrap()
            .expect("Session should exist");

        assert_eq!(restored.session_id, "session-123");
        assert_eq!(restored.agent_states.len(), 1);
        assert_eq!(restored.task_queue.len(), 1);
        assert_eq!(
            restored.agent_states.get("agent-1").unwrap().status,
            AgentStatus::Running
        );
    }

    #[tokio::test]
    async fn test_restore_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = SessionPersistence::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let result = persistence.restore_session("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = SessionPersistence::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Save multiple sessions
        persistence
            .save_session(&SessionState::new("session-1"))
            .await
            .unwrap();
        persistence
            .save_session(&SessionState::new("session-2"))
            .await
            .unwrap();
        persistence
            .save_session(&SessionState::new("session-3"))
            .await
            .unwrap();

        let sessions = persistence.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 3);
        assert!(sessions.contains(&"session-1".to_string()));
        assert!(sessions.contains(&"session-2".to_string()));
        assert!(sessions.contains(&"session-3".to_string()));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = SessionPersistence::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Save and then delete
        persistence
            .save_session(&SessionState::new("session-123"))
            .await
            .unwrap();

        persistence.delete_session("session-123").await.unwrap();

        let result = persistence.restore_session("session-123").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let persist_path = temp_dir.path().to_path_buf();

        // Save with first instance
        {
            let persistence = SessionPersistence::new(persist_path.clone()).await.unwrap();
            let mut state = SessionState::new("session-persistent");
            state.update_agent(
                "agent-1".to_string(),
                AgentState::new("agent-1", AgentStatus::Running),
            );
            persistence.save_session(&state).await.unwrap();
        }

        // Restore with second instance
        {
            let persistence = SessionPersistence::new(persist_path).await.unwrap();
            let restored = persistence
                .restore_session("session-persistent")
                .await
                .unwrap()
                .expect("Session should exist");

            assert_eq!(restored.session_id, "session-persistent");
            assert_eq!(restored.agent_states.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_clear_all_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = SessionPersistence::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Save multiple sessions
        persistence
            .save_session(&SessionState::new("session-1"))
            .await
            .unwrap();
        persistence
            .save_session(&SessionState::new("session-2"))
            .await
            .unwrap();

        persistence.clear_all().await.unwrap();

        let sessions = persistence.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 0);
    }
}
