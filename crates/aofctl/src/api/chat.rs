//! Chat API endpoints for Mission Control SquadChat
//!
//! Provides in-memory chat message storage and HTTP endpoints for the
//! frontend SquadChat component. Messages persist within a daemon session
//! and are broadcast via WebSocket for multi-client sync.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use aof_coordination::EventBroadcaster;

// ============================================================================
// Types
// ============================================================================

/// Chat message matching the frontend TypeScript ChatMessage interface.
///
/// Field names serialize as camelCase to match the frontend contract exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// Server-assigned UUID v4
    pub id: String,
    /// Agent or user identifier
    pub sender_id: String,
    /// Display name
    pub sender_name: String,
    /// Optional avatar URL or emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_avatar: Option<String>,
    /// Message body (supports markdown)
    pub content: String,
    /// ISO 8601 timestamp (server-assigned)
    pub timestamp: String,
    /// Optional thread ID for reply threading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Optional version for conflict resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
}

/// Request payload for creating a new chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// Message body
    pub content: String,
    /// Sender identifier
    pub sender_id: String,
    /// Sender display name
    pub sender_name: String,
    /// Optional avatar URL or emoji
    #[serde(default)]
    pub sender_avatar: Option<String>,
}

// ============================================================================
// Chat Store
// ============================================================================

/// Maximum number of messages retained in memory.
const MAX_MESSAGES: usize = 1000;

/// In-memory chat message store with capacity limit.
#[derive(Debug)]
pub struct ChatStore {
    messages: Vec<ChatMessage>,
    max_capacity: usize,
}

impl ChatStore {
    /// Create a new empty chat store.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_capacity: MAX_MESSAGES,
        }
    }

    /// Create a chat store with a custom capacity (for testing).
    #[cfg(test)]
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_capacity,
        }
    }

    /// Add a message from a request. Assigns server UUID and timestamp.
    /// Returns the created ChatMessage.
    pub fn add_message(&mut self, request: SendMessageRequest) -> ChatMessage {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id: request.sender_id,
            sender_name: request.sender_name,
            sender_avatar: request.sender_avatar,
            content: request.content,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            thread_id: None,
            version: None,
        };
        self.messages.push(message.clone());
        self.trim();
        message
    }

    /// Return all messages in chronological order.
    pub fn get_all(&self) -> Vec<ChatMessage> {
        self.messages.clone()
    }

    /// Return messages after the given message ID (for reconnection recovery).
    /// If the ID is not found, returns all messages (graceful degradation).
    pub fn get_since(&self, message_id: &str) -> Vec<ChatMessage> {
        if let Some(pos) = self.messages.iter().position(|m| m.id == message_id) {
            self.messages[(pos + 1)..].to_vec()
        } else {
            // Unknown ID -> return all (graceful degradation)
            self.get_all()
        }
    }

    /// Drop oldest messages if over capacity.
    fn trim(&mut self) {
        if self.messages.len() > self.max_capacity {
            let excess = self.messages.len() - self.max_capacity;
            self.messages.drain(..excess);
        }
    }
}

impl Default for ChatStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Shared State
// ============================================================================

/// Shared state for chat API handlers.
#[derive(Clone)]
pub struct ChatState {
    pub store: Arc<RwLock<ChatStore>>,
    pub event_bus: Option<Arc<EventBroadcaster>>,
}

impl ChatState {
    /// Create a new ChatState with an empty store.
    pub fn new(event_bus: Option<Arc<EventBroadcaster>>) -> Self {
        Self {
            store: Arc::new(RwLock::new(ChatStore::new())),
            event_bus,
        }
    }
}

// ============================================================================
// Error Type
// ============================================================================

/// Chat API error type.
#[derive(Debug)]
pub enum ChatApiError {
    BadRequest(String),
}

impl IntoResponse for ChatApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ChatApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for GET /api/chat/messages.
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    /// Optional message ID to resume from (reconnection recovery).
    pub since: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/chat/messages
///
/// Returns all messages, or messages since a given ID if `?since=<id>` is provided.
pub async fn get_messages(
    State(state): State<ChatState>,
    Query(params): Query<MessageQuery>,
) -> Json<Vec<ChatMessage>> {
    let store = state.store.read().await;
    match params.since {
        Some(since_id) => Json(store.get_since(&since_id)),
        None => Json(store.get_all()),
    }
}

/// POST /api/chat/messages
///
/// Creates a new chat message. Validates required fields, assigns server UUID
/// and timestamp, stores the message, and broadcasts via WebSocket.
pub async fn send_message(
    State(state): State<ChatState>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessage>), ChatApiError> {
    // Validate required fields
    if payload.content.trim().is_empty() {
        return Err(ChatApiError::BadRequest("content is required".to_string()));
    }
    if payload.sender_id.trim().is_empty() {
        return Err(ChatApiError::BadRequest("senderId is required".to_string()));
    }
    if payload.sender_name.trim().is_empty() {
        return Err(ChatApiError::BadRequest("senderName is required".to_string()));
    }

    let mut store = state.store.write().await;
    let message = store.add_message(payload);

    // Emit CHAT_MESSAGE event via event bus for WebSocket broadcast
    if let Some(ref event_bus) = state.event_bus {
        let event = build_chat_message_event(&message);
        event_bus.emit(event);
    }

    Ok((StatusCode::CREATED, Json(message)))
}

// ============================================================================
// WebSocket Event Builder
// ============================================================================

/// Build a CoordinationEvent for a new chat message.
///
/// Creates an Info-type activity event with chat message metadata so that
/// WebSocket subscribers can process it as a CHAT_MESSAGE event. The frontend
/// SquadChat component distinguishes chat events using the metadata field.
pub fn build_chat_message_event(message: &ChatMessage) -> aof_core::CoordinationEvent {
    use std::collections::HashMap;

    let mut metadata = HashMap::new();
    metadata.insert("type".to_string(), "chat_message".to_string());
    metadata.insert("messageId".to_string(), message.id.clone());
    metadata.insert("senderId".to_string(), message.sender_id.clone());
    metadata.insert("senderName".to_string(), message.sender_name.clone());
    metadata.insert("content".to_string(), message.content.clone());
    metadata.insert("timestamp".to_string(), message.timestamp.clone());
    if let Some(ref avatar) = message.sender_avatar {
        metadata.insert("senderAvatar".to_string(), avatar.clone());
    }

    let activity = aof_core::ActivityEvent::info(
        format!("Chat message from {}", message.sender_name),
    ).with_details(aof_core::ActivityDetails {
        tool_name: None,
        tool_args: None,
        duration_ms: None,
        tokens: None,
        error: None,
        metadata: Some(metadata),
    });

    aof_core::CoordinationEvent::from_activity(
        activity,
        &message.sender_id,
        "chat",
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        let mut store = ChatStore::new();
        let request = SendMessageRequest {
            content: "Hello squad!".to_string(),
            sender_id: "user_1".to_string(),
            sender_name: "Operator".to_string(),
            sender_avatar: None,
        };

        let message = store.add_message(request);

        // Verify UUID assigned (36 chars with hyphens)
        assert_eq!(message.id.len(), 36);
        assert!(message.id.contains('-'));

        // Verify timestamp set (ISO 8601 format)
        assert!(message.timestamp.contains('T'));
        assert!(message.timestamp.ends_with('Z'));

        // Verify content preserved
        assert_eq!(message.content, "Hello squad!");
        assert_eq!(message.sender_id, "user_1");
        assert_eq!(message.sender_name, "Operator");
        assert!(message.sender_avatar.is_none());
        assert!(message.thread_id.is_none());
        assert!(message.version.is_none());
    }

    #[test]
    fn test_get_all_empty() {
        let store = ChatStore::new();
        let messages = store.get_all();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_get_all_ordered() {
        let mut store = ChatStore::new();

        for i in 1..=5 {
            store.add_message(SendMessageRequest {
                content: format!("Message {}", i),
                sender_id: "user_1".to_string(),
                sender_name: "Test".to_string(),
                sender_avatar: None,
            });
        }

        let messages = store.get_all();
        assert_eq!(messages.len(), 5);

        // Verify chronological order (insertion order)
        for i in 0..4 {
            assert!(messages[i].timestamp <= messages[i + 1].timestamp);
        }

        // Verify content order
        assert_eq!(messages[0].content, "Message 1");
        assert_eq!(messages[4].content, "Message 5");
    }

    #[test]
    fn test_get_since_valid() {
        let mut store = ChatStore::new();

        let mut ids = Vec::new();
        for i in 1..=5 {
            let msg = store.add_message(SendMessageRequest {
                content: format!("Message {}", i),
                sender_id: "user_1".to_string(),
                sender_name: "Test".to_string(),
                sender_avatar: None,
            });
            ids.push(msg.id);
        }

        // Get messages since message 2 (should return messages 3, 4, 5)
        let since_messages = store.get_since(&ids[1]);
        assert_eq!(since_messages.len(), 3);
        assert_eq!(since_messages[0].content, "Message 3");
        assert_eq!(since_messages[1].content, "Message 4");
        assert_eq!(since_messages[2].content, "Message 5");

        // Get messages since last message (should return empty)
        let since_last = store.get_since(&ids[4]);
        assert!(since_last.is_empty());
    }

    #[test]
    fn test_get_since_unknown_id() {
        let mut store = ChatStore::new();

        store.add_message(SendMessageRequest {
            content: "Hello".to_string(),
            sender_id: "user_1".to_string(),
            sender_name: "Test".to_string(),
            sender_avatar: None,
        });

        // Unknown ID should return all messages (graceful degradation)
        let messages = store.get_since("nonexistent-id");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_capacity_limit() {
        let mut store = ChatStore::with_capacity(5);

        // Add 8 messages (exceeds capacity of 5)
        for i in 1..=8 {
            store.add_message(SendMessageRequest {
                content: format!("Message {}", i),
                sender_id: "user_1".to_string(),
                sender_name: "Test".to_string(),
                sender_avatar: None,
            });
        }

        let messages = store.get_all();
        assert_eq!(messages.len(), 5);

        // Oldest messages should be dropped (1, 2, 3 gone)
        assert_eq!(messages[0].content, "Message 4");
        assert_eq!(messages[4].content, "Message 8");
    }

    #[test]
    fn test_validation_empty_content() {
        // Test the validation logic directly (not via handler)
        let payload = SendMessageRequest {
            content: "   ".to_string(), // whitespace only
            sender_id: "user_1".to_string(),
            sender_name: "Test".to_string(),
            sender_avatar: None,
        };
        assert!(payload.content.trim().is_empty());
    }

    #[test]
    fn test_validation_empty_sender() {
        let payload = SendMessageRequest {
            content: "Hello".to_string(),
            sender_id: "  ".to_string(), // whitespace only
            sender_name: "Test".to_string(),
            sender_avatar: None,
        };
        assert!(payload.sender_id.trim().is_empty());
    }

    #[test]
    fn test_camelcase_serialization() {
        let message = ChatMessage {
            id: "test-id".to_string(),
            sender_id: "user_1".to_string(),
            sender_name: "Test User".to_string(),
            sender_avatar: Some("avatar.png".to_string()),
            content: "Hello".to_string(),
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            thread_id: None,
            version: None,
        };

        let json = serde_json::to_string(&message).unwrap();

        // Verify camelCase field names
        assert!(json.contains("\"senderId\""));
        assert!(json.contains("\"senderName\""));
        assert!(json.contains("\"senderAvatar\""));

        // Verify None fields are skipped
        assert!(!json.contains("threadId"));
        assert!(!json.contains("version"));
    }

    #[test]
    fn test_chat_message_event_construction() {
        let message = ChatMessage {
            id: "msg-001".to_string(),
            sender_id: "user_1".to_string(),
            sender_name: "Operator".to_string(),
            sender_avatar: None,
            content: "Hello squad!".to_string(),
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            thread_id: None,
            version: None,
        };

        let event = build_chat_message_event(&message);

        assert_eq!(event.agent_id, "user_1");
        assert_eq!(event.session_id, "chat");
        assert!(event.activity.message.contains("Chat message from Operator"));

        // Verify metadata
        let details = event.activity.details.as_ref().unwrap();
        let metadata = details.metadata.as_ref().unwrap();
        assert_eq!(metadata.get("type").unwrap(), "chat_message");
        assert_eq!(metadata.get("messageId").unwrap(), "msg-001");
        assert_eq!(metadata.get("content").unwrap(), "Hello squad!");
    }
}
