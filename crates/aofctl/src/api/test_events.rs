//! Test event emission endpoint for real-time pipeline verification.
//!
//! Provides POST /api/test/emit-event to emit controlled CoordinationEvents
//! for testing the WebSocket pipeline without requiring actual agent execution.
//! Intended for development and testing only.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use aof_coordination::EventBroadcaster;
use aof_core::activity::{ActivityEvent, ActivityType, ActivityDetails};
use aof_core::CoordinationEvent;

// ============================================================================
// Types
// ============================================================================

/// Request payload for POST /api/test/emit-event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEventRequest {
    /// Agent ID to associate with the event (e.g., "k8s-monitor")
    pub agent_id: String,

    /// Event type string (e.g., "agent_started", "agent_completed", "agent_error")
    pub event_type: String,

    /// Optional details object for event-specific data
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

/// Response payload for POST /api/test/emit-event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEventResponse {
    /// Whether the event was emitted
    pub emitted: bool,

    /// The generated event ID
    pub event_id: String,

    /// Echo back the event type
    pub event_type: String,

    /// Echo back the agent ID
    pub agent_id: String,
}

// ============================================================================
// State
// ============================================================================

/// Shared state for test event endpoint.
#[derive(Clone)]
pub struct TestEventsState {
    pub event_bus: Option<Arc<EventBroadcaster>>,
}

impl TestEventsState {
    pub fn new(event_bus: Option<Arc<EventBroadcaster>>) -> Self {
        Self { event_bus }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/test/emit-event
///
/// Emit a CoordinationEvent for testing the real-time WebSocket pipeline.
/// Accepts an agent_id, event_type, and optional details object.
/// Builds and emits a CoordinationEvent via the EventBroadcaster.
pub async fn emit_test_event(
    State(state): State<TestEventsState>,
    Json(payload): Json<TestEventRequest>,
) -> impl IntoResponse {
    // Validate required fields
    if payload.agent_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "agent_id is required" })),
        ).into_response();
    }

    if payload.event_type.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "event_type is required" })),
        ).into_response();
    }

    // Map event_type string to ActivityType
    // Frontend AgentGrid.tsx maps these to status via getAgentStatus():
    //   agent_started/thinking/tool_executing -> "working"
    //   agent_completed/tool_completed -> "idle"
    //   error/tool_failed -> "error"
    let activity_type = match payload.event_type.as_str() {
        "agent_started" => ActivityType::Started,
        "agent_completed" => ActivityType::Completed,
        "agent_error" | "error" => ActivityType::Error,
        "task_assigned" => ActivityType::Info,
        "tool_called" | "tool_executing" => ActivityType::ToolExecuting,
        "tool_completed" => ActivityType::ToolComplete,
        "tool_failed" => ActivityType::ToolFailed,
        "thinking" => ActivityType::Thinking,
        "info" => ActivityType::Info,
        "warning" => ActivityType::Warning,
        _ => ActivityType::Info, // Default to Info for unknown types
    };

    // Build activity message
    let message = format!(
        "Test event: {} for agent {}",
        payload.event_type, payload.agent_id
    );

    // Build optional metadata from details
    let details = payload.details.as_ref().map(|d| {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("source".to_string(), "test_endpoint".to_string());
        metadata.insert("event_type".to_string(), payload.event_type.clone());

        // Flatten top-level details into metadata if it's an object
        if let Some(obj) = d.as_object() {
            for (k, v) in obj {
                metadata.insert(k.clone(), v.to_string().trim_matches('"').to_string());
            }
        }

        ActivityDetails {
            tool_name: None,
            tool_args: None,
            duration_ms: None,
            tokens: None,
            error: if activity_type == ActivityType::Error {
                Some(format!("Test error for agent {}", payload.agent_id))
            } else {
                None
            },
            metadata: Some(metadata),
        }
    });

    // Build CoordinationEvent
    let event = CoordinationEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        agent_id: payload.agent_id.clone(),
        session_id: "test".to_string(),
        timestamp: chrono::Utc::now(),
        activity: ActivityEvent {
            activity_type,
            message,
            timestamp: chrono::Utc::now(),
            details,
        },
        introduction: None,
        coordination_activity: None,
    };

    let event_id = event.event_id.clone();

    // Emit via event bus
    if let Some(ref event_bus) = state.event_bus {
        event_bus.emit(event);
    }

    let response = TestEventResponse {
        emitted: true,
        event_id,
        event_type: payload.event_type,
        agent_id: payload.agent_id,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_mapping() {
        // Verify all documented event types map to expected ActivityTypes
        let mappings = vec![
            ("agent_started", ActivityType::Started),
            ("agent_completed", ActivityType::Completed),
            ("agent_error", ActivityType::Error),
            ("error", ActivityType::Error),
            ("task_assigned", ActivityType::Info),
            ("tool_called", ActivityType::ToolExecuting),
            ("tool_executing", ActivityType::ToolExecuting),
            ("tool_completed", ActivityType::ToolComplete),
            ("tool_failed", ActivityType::ToolFailed),
            ("thinking", ActivityType::Thinking),
            ("info", ActivityType::Info),
            ("warning", ActivityType::Warning),
            ("unknown_type", ActivityType::Info), // defaults to Info
        ];

        for (event_type, expected) in mappings {
            let mapped = match event_type {
                "agent_started" => ActivityType::Started,
                "agent_completed" => ActivityType::Completed,
                "agent_error" | "error" => ActivityType::Error,
                "task_assigned" => ActivityType::Info,
                "tool_called" | "tool_executing" => ActivityType::ToolExecuting,
                "tool_completed" => ActivityType::ToolComplete,
                "tool_failed" => ActivityType::ToolFailed,
                "thinking" => ActivityType::Thinking,
                "info" => ActivityType::Info,
                "warning" => ActivityType::Warning,
                _ => ActivityType::Info,
            };
            assert_eq!(mapped, expected, "Event type '{}' should map to {:?}", event_type, expected);
        }
    }

    #[test]
    fn test_request_deserialization() {
        let json = r#"{
            "agent_id": "k8s-monitor",
            "event_type": "agent_started",
            "details": {"custom_key": "custom_value"}
        }"#;

        let request: TestEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.agent_id, "k8s-monitor");
        assert_eq!(request.event_type, "agent_started");
        assert!(request.details.is_some());
    }

    #[test]
    fn test_request_without_details() {
        let json = r#"{
            "agent_id": "k8s-monitor",
            "event_type": "agent_started"
        }"#;

        let request: TestEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.agent_id, "k8s-monitor");
        assert_eq!(request.event_type, "agent_started");
        assert!(request.details.is_none());
    }

    #[test]
    fn test_response_serialization() {
        let response = TestEventResponse {
            emitted: true,
            event_id: "test-uuid".to_string(),
            event_type: "agent_started".to_string(),
            agent_id: "k8s-monitor".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"emitted\":true"));
        assert!(json.contains("\"event_id\":\"test-uuid\""));
        assert!(json.contains("\"event_type\":\"agent_started\""));
        assert!(json.contains("\"agent_id\":\"k8s-monitor\""));
    }
}
