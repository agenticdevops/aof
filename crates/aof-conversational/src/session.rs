use crate::types::{ConversationMessage, ConversationSession};
use anyhow::{anyhow, Result};
use chrono::Utc;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session entry with last activity tracking
struct SessionEntry {
    session: ConversationSession,
    last_activity: Instant,
}

/// Conversation session store with LRU cache and TTL expiry
pub struct ConversationSessionStore {
    sessions: Arc<RwLock<LruCache<String, SessionEntry>>>,
    ttl: Duration,
}

impl ConversationSessionStore {
    /// Create a new session store
    ///
    /// # Arguments
    ///
    /// * `max_sessions` - Maximum number of sessions to keep in cache
    /// * `ttl` - Time-to-live for inactive sessions
    pub fn new(max_sessions: usize, ttl: Duration) -> Self {
        let capacity = NonZeroUsize::new(max_sessions).expect("max_sessions must be > 0");
        Self {
            sessions: Arc::new(RwLock::new(LruCache::new(capacity))),
            ttl,
        }
    }

    /// Create a new session and return its ID
    pub async fn create(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = ConversationSession::new(session_id.clone());
        let entry = SessionEntry {
            session,
            last_activity: Instant::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.put(session_id.clone(), entry);

        session_id
    }

    /// Get a session by ID
    ///
    /// Returns None if session doesn't exist or has expired
    pub async fn get(&self, session_id: &str) -> Option<ConversationSession> {
        let mut sessions = self.sessions.write().await;

        if let Some(entry) = sessions.get(session_id) {
            // Check TTL
            if entry.last_activity.elapsed() > self.ttl {
                // Expired - remove it
                sessions.pop(session_id);
                return None;
            }

            Some(entry.session.clone())
        } else {
            None
        }
    }

    /// Update an existing session
    ///
    /// Refreshes the last_activity timestamp
    pub async fn update(&self, session: ConversationSession) {
        let mut sessions = self.sessions.write().await;

        let entry = SessionEntry {
            session,
            last_activity: Instant::now(),
        };

        sessions.put(entry.session.session_id.clone(), entry);
    }

    /// Add a message to a session
    pub async fn add_message(
        &self,
        session_id: &str,
        message: ConversationMessage,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(entry) = sessions.get_mut(session_id) {
            // Update session
            entry.session.messages.push(message);
            entry.session.updated_at = Utc::now();
            entry.last_activity = Instant::now();
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Set pending files for a session
    pub async fn set_pending_files(
        &self,
        session_id: &str,
        files: HashMap<String, String>,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(entry) = sessions.get_mut(session_id) {
            entry.session.pending_files = files;
            entry.session.updated_at = Utc::now();
            entry.last_activity = Instant::now();
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Clean up expired sessions
    ///
    /// Returns the number of sessions removed
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut expired = Vec::new();

        // Collect expired session IDs
        // Note: LRU iterator doesn't support modification during iteration
        for (id, entry) in sessions.iter() {
            if entry.last_activity.elapsed() > self.ttl {
                expired.push(id.clone());
            }
        }

        // Remove expired sessions
        for id in &expired {
            sessions.pop(id);
        }

        expired.len()
    }

    /// Get current session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageRole;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_create_session() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));
        let session_id = store.create().await;

        // Should be valid UUID
        assert!(Uuid::parse_str(&session_id).is_ok());

        // Should be retrievable
        let session = store.get(&session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().session_id, session_id);
    }

    #[tokio::test]
    async fn test_get_returns_none_for_missing() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));
        let session = store.get("nonexistent").await;
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn test_update_refreshes_activity() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(1));
        let session_id = store.create().await;

        sleep(Duration::from_millis(500)).await;

        // Update session
        let mut session = store.get(&session_id).await.unwrap();
        session.messages.push(ConversationMessage {
            role: MessageRole::User,
            content: "test".to_string(),
            timestamp: Utc::now(),
        });
        store.update(session).await;

        // Wait another 600ms (total 1.1s, but update was at 500ms)
        sleep(Duration::from_millis(600)).await;

        // Should still exist because update refreshed activity
        let session = store.get(&session_id).await;
        assert!(session.is_some());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let store = ConversationSessionStore::new(10, Duration::from_millis(100));
        let session_id = store.create().await;

        // Should exist immediately
        assert!(store.get(&session_id).await.is_some());

        // Wait past TTL
        sleep(Duration::from_millis(150)).await;

        // Should be expired
        assert!(store.get(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let store = ConversationSessionStore::new(3, Duration::from_secs(300));

        let id1 = store.create().await;
        let id2 = store.create().await;
        let id3 = store.create().await;

        // All should exist
        assert!(store.get(&id1).await.is_some());
        assert!(store.get(&id2).await.is_some());
        assert!(store.get(&id3).await.is_some());

        // Create 4th session - should evict oldest (id1)
        let _id4 = store.create().await;

        // id1 should be gone
        assert!(store.get(&id1).await.is_none());

        // Others should still exist
        assert!(store.get(&id2).await.is_some());
        assert!(store.get(&id3).await.is_some());
    }

    #[tokio::test]
    async fn test_add_message() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));
        let session_id = store.create().await;

        let message = ConversationMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: Utc::now(),
        };

        let result = store.add_message(&session_id, message).await;
        assert!(result.is_ok());

        // Verify message was added
        let session = store.get(&session_id).await.unwrap();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_add_message_to_nonexistent() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));

        let message = ConversationMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: Utc::now(),
        };

        let result = store.add_message("nonexistent", message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_pending_files() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));
        let session_id = store.create().await;

        let mut files = HashMap::new();
        files.insert("agent.yaml".to_string(), "content".to_string());

        let result = store.set_pending_files(&session_id, files).await;
        assert!(result.is_ok());

        // Verify files were set
        let session = store.get(&session_id).await.unwrap();
        assert_eq!(session.pending_files.len(), 1);
        assert_eq!(session.pending_files.get("agent.yaml"), Some(&"content".to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let store = ConversationSessionStore::new(10, Duration::from_millis(100));

        let _id1 = store.create().await;
        let _id2 = store.create().await;

        assert_eq!(store.session_count().await, 2);

        // Wait for expiry
        sleep(Duration::from_millis(150)).await;

        let removed = store.cleanup_expired().await;
        assert_eq!(removed, 2);
        assert_eq!(store.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_count() {
        let store = ConversationSessionStore::new(10, Duration::from_secs(300));

        assert_eq!(store.session_count().await, 0);

        let _id1 = store.create().await;
        assert_eq!(store.session_count().await, 1);

        let _id2 = store.create().await;
        assert_eq!(store.session_count().await, 2);
    }
}
