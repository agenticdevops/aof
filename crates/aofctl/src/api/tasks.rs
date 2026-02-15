//! Tasks API endpoints for Mission Control Kanban board
//!
//! Provides GET /api/tasks, POST /api/tasks, and POST /api/tasks/move
//! for the frontend KanbanBoard drag-and-drop functionality.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use aof_coordination::{CoordinationEvent, EventBroadcaster};
use aof_core::activity::{ActivityEvent, ActivityType};

// ============================================================================
// Types
// ============================================================================

/// Valid lane values for tasks
const VALID_LANES: &[&str] = &["backlog", "assigned", "in-progress", "review", "done"];

/// Task entity matching frontend TypeScript Task interface.
/// All fields use camelCase serialization to match the frontend contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub lane: String,
    pub assigned_to: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<String>,
}

/// Request payload for POST /api/tasks/move
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskRequest {
    pub task_id: String,
    pub new_lane: String,
    pub version: u64,
}

/// Response payload for POST /api/tasks/move
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskResponse {
    pub task: Task,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request payload for POST /api/tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub lane: Option<String>,
    pub assigned_to: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// Error Type
// ============================================================================

/// API error type with proper HTTP status codes
#[derive(Debug)]
pub enum TaskApiError {
    /// Task not found - 404
    NotFound(String),
    /// Version conflict - 409 (includes current server-side task)
    Conflict(Task),
    /// Bad request - 400
    BadRequest(String),
}

impl IntoResponse for TaskApiError {
    fn into_response(self) -> Response {
        match self {
            TaskApiError::NotFound(task_id) => {
                let body = serde_json::json!({
                    "error": format!("Task not found: {}", task_id)
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }
            TaskApiError::Conflict(task) => {
                let body = MoveTaskResponse {
                    task,
                    success: false,
                    error: Some("Version conflict: task was modified by another client".to_string()),
                };
                (StatusCode::CONFLICT, Json(body)).into_response()
            }
            TaskApiError::BadRequest(msg) => {
                let body = serde_json::json!({
                    "error": msg
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
        }
    }
}

// ============================================================================
// TaskStore
// ============================================================================

/// In-memory task storage with seed data
#[derive(Debug, Clone)]
pub struct TaskStore {
    tasks: Vec<Task>,
}

impl TaskStore {
    /// Create a new TaskStore seeded with 5 sample tasks
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let tasks = vec![
            Task {
                id: "task-001".to_string(),
                title: "Monitor k8s cluster health".to_string(),
                description: "Periodic health check of production cluster nodes and pods".to_string(),
                lane: "in-progress".to_string(),
                assigned_to: Some("sentinel".to_string()),
                version: 1,
                created_at: now.clone(),
                updated_at: now.clone(),
                status: "active".to_string(),
                priority: Some("high".to_string()),
                tags: Some(vec!["monitoring".to_string(), "k8s".to_string()]),
                due_date: None,
            },
            Task {
                id: "task-002".to_string(),
                title: "Review deployment pipeline".to_string(),
                description: "Audit CI/CD pipeline for security and efficiency improvements".to_string(),
                lane: "backlog".to_string(),
                assigned_to: None,
                version: 1,
                created_at: now.clone(),
                updated_at: now.clone(),
                status: "pending".to_string(),
                priority: Some("medium".to_string()),
                tags: Some(vec!["ci-cd".to_string(), "review".to_string()]),
                due_date: None,
            },
            Task {
                id: "task-003".to_string(),
                title: "Update security patches".to_string(),
                description: "Apply latest CVE patches to base images and dependencies".to_string(),
                lane: "assigned".to_string(),
                assigned_to: Some("patchbot".to_string()),
                version: 1,
                created_at: now.clone(),
                updated_at: now.clone(),
                status: "active".to_string(),
                priority: Some("critical".to_string()),
                tags: Some(vec!["security".to_string(), "patches".to_string()]),
                due_date: None,
            },
            Task {
                id: "task-004".to_string(),
                title: "Analyze error logs".to_string(),
                description: "Review application error logs for recurring patterns".to_string(),
                lane: "review".to_string(),
                assigned_to: Some("analyst".to_string()),
                version: 1,
                created_at: now.clone(),
                updated_at: now.clone(),
                status: "active".to_string(),
                priority: Some("medium".to_string()),
                tags: Some(vec!["logs".to_string(), "analysis".to_string()]),
                due_date: None,
            },
            Task {
                id: "task-005".to_string(),
                title: "Optimize database queries".to_string(),
                description: "Identify and optimize slow database queries in production".to_string(),
                lane: "done".to_string(),
                assigned_to: Some("dbadmin".to_string()),
                version: 1,
                created_at: now.clone(),
                updated_at: now,
                status: "completed".to_string(),
                priority: Some("low".to_string()),
                tags: Some(vec!["database".to_string(), "performance".to_string()]),
                due_date: None,
            },
        ];

        Self { tasks }
    }

    /// Get all tasks
    pub fn get_all(&self) -> Vec<Task> {
        self.tasks.clone()
    }

    /// Get a task by ID
    pub fn get_by_id(&self, id: &str) -> Option<Task> {
        self.tasks.iter().find(|t| t.id == id).cloned()
    }

    /// Create a new task from a request payload
    pub fn create(&mut self, req: CreateTaskRequest) -> Task {
        let now = chrono::Utc::now().to_rfc3339();
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: req.title,
            description: req.description.unwrap_or_default(),
            lane: req.lane.unwrap_or_else(|| "backlog".to_string()),
            assigned_to: req.assigned_to,
            version: 1,
            created_at: now.clone(),
            updated_at: now,
            status: "pending".to_string(),
            priority: req.priority,
            tags: req.tags,
            due_date: None,
        };
        self.tasks.push(task.clone());
        task
    }

    /// Move a task to a new lane, incrementing version and updating timestamp.
    /// Returns the updated task or an error string.
    pub fn move_task(&mut self, task_id: &str, new_lane: &str) -> Result<Task, String> {
        let task = self.tasks.iter_mut().find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        let old_lane = task.lane.clone();
        task.lane = new_lane.to_string();
        task.version += 1;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        // Status transitions based on lane change
        if new_lane == "done" {
            task.status = "completed".to_string();
        } else if old_lane == "backlog" && new_lane != "backlog" {
            task.status = "active".to_string();
        }

        Ok(task.clone())
    }
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Shared State
// ============================================================================

/// Shared state for task API handlers
#[derive(Clone)]
pub struct TasksState {
    pub store: Arc<RwLock<TaskStore>>,
    pub event_bus: Option<Arc<EventBroadcaster>>,
}

impl TasksState {
    pub fn new(event_bus: Option<Arc<EventBroadcaster>>) -> Self {
        Self {
            store: Arc::new(RwLock::new(TaskStore::new())),
            event_bus,
        }
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Validate that a lane value is one of the allowed values
fn validate_lane(lane: &str) -> Result<(), TaskApiError> {
    if VALID_LANES.contains(&lane) {
        Ok(())
    } else {
        Err(TaskApiError::BadRequest(format!(
            "Invalid lane: '{}'. Must be one of: {}",
            lane,
            VALID_LANES.join(", ")
        )))
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/tasks - Returns all tasks
pub async fn get_tasks(
    State(state): State<TasksState>,
) -> Json<Vec<Task>> {
    let store = state.store.read().await;
    Json(store.get_all())
}

/// POST /api/tasks - Create a new task
pub async fn create_task(
    State(state): State<TasksState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), TaskApiError> {
    // Validate required fields
    if payload.title.trim().is_empty() {
        return Err(TaskApiError::BadRequest("title is required".to_string()));
    }

    // Validate lane if provided
    if let Some(ref lane) = payload.lane {
        validate_lane(lane)?;
    }

    let mut store = state.store.write().await;
    let task = store.create(payload);

    // Emit TASK_CREATED event via WebSocket
    if let Some(ref event_bus) = state.event_bus {
        let event = build_task_event("TASK_CREATED", &task, None);
        event_bus.emit(event);
    }

    Ok((StatusCode::CREATED, Json(task)))
}

/// POST /api/tasks/move - Move a task to a different lane
pub async fn move_task(
    State(state): State<TasksState>,
    Json(payload): Json<MoveTaskRequest>,
) -> Result<Json<MoveTaskResponse>, TaskApiError> {
    validate_lane(&payload.new_lane)?;

    let mut store = state.store.write().await;

    // Find task
    let task = store.get_by_id(&payload.task_id)
        .ok_or_else(|| TaskApiError::NotFound(payload.task_id.clone()))?;

    // Version conflict check
    if task.version != payload.version {
        return Err(TaskApiError::Conflict(task));
    }

    let from_lane = task.lane.clone();

    // Move task
    let updated = store.move_task(&payload.task_id, &payload.new_lane)
        .map_err(|e| TaskApiError::BadRequest(e))?;

    // Emit TASK_MOVED event via event bus for WebSocket broadcast
    if let Some(ref event_bus) = state.event_bus {
        let event = build_task_event("TASK_MOVED", &updated, Some(&from_lane));
        event_bus.emit(event);
    }

    Ok(Json(MoveTaskResponse {
        task: updated,
        success: true,
        error: None,
    }))
}

// ============================================================================
// Event Builder
// ============================================================================

/// Build a CoordinationEvent for task mutations (TASK_CREATED, TASK_MOVED)
fn build_task_event(
    event_type: &str,
    task: &Task,
    from_lane: Option<&str>,
) -> CoordinationEvent {
    let message = if let Some(from) = from_lane {
        format!("{}: {} (from {} to {})", event_type, task.title, from, task.lane)
    } else {
        format!("{}: {} (lane: {})", event_type, task.title, task.lane)
    };

    CoordinationEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        agent_id: "task-api".to_string(),
        session_id: "daemon".to_string(),
        timestamp: chrono::Utc::now(),
        activity: ActivityEvent {
            activity_type: ActivityType::Info,
            message,
            timestamp: chrono::Utc::now(),
            details: None,
        },
        introduction: None,
        coordination_activity: None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let mut store = TaskStore::new();
        let req = CreateTaskRequest {
            title: "Test task".to_string(),
            description: Some("A test".to_string()),
            lane: None,
            assigned_to: None,
            priority: Some("high".to_string()),
            tags: None,
        };

        let task = store.create(req);

        // UUID format check (36 chars with hyphens)
        assert_eq!(task.id.len(), 36);
        assert_eq!(task.version, 1);
        assert_eq!(task.lane, "backlog"); // default lane
        assert_eq!(task.status, "pending");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, "A test");
        assert_eq!(task.priority, Some("high".to_string()));
    }

    #[test]
    fn test_get_all_tasks() {
        let store = TaskStore::new();
        let tasks = store.get_all();
        assert_eq!(tasks.len(), 5); // 5 seeded tasks
    }

    #[test]
    fn test_get_task_by_id() {
        let store = TaskStore::new();

        // Existing task
        let task = store.get_by_id("task-001");
        assert!(task.is_some());
        assert_eq!(task.unwrap().title, "Monitor k8s cluster health");

        // Missing task
        let missing = store.get_by_id("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_move_task() {
        let mut store = TaskStore::new();

        let result = store.move_task("task-001", "review");
        assert!(result.is_ok());

        let moved = result.unwrap();
        assert_eq!(moved.lane, "review");
        assert_eq!(moved.version, 2); // incremented from 1
    }

    #[test]
    fn test_move_task_version_conflict() {
        let mut store = TaskStore::new();

        // Move once to increment version to 2
        let _ = store.move_task("task-001", "review").unwrap();

        // Now the task is at version 2, simulate conflict check
        let task = store.get_by_id("task-001").unwrap();
        assert_eq!(task.version, 2);

        // A request with version=1 would be a conflict (handled at handler level)
        // At store level, we just verify the version changed
        assert_ne!(task.version, 1);
    }

    #[test]
    fn test_move_task_not_found() {
        let mut store = TaskStore::new();
        let result = store.move_task("nonexistent", "review");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task not found"));
    }

    #[test]
    fn test_invalid_lane() {
        let result = validate_lane("invalid-lane");
        assert!(result.is_err());

        let result = validate_lane("backlog");
        assert!(result.is_ok());

        let result = validate_lane("in-progress");
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_to_done_sets_completed() {
        let mut store = TaskStore::new();

        // task-002 is in backlog with status "pending"
        let task = store.get_by_id("task-002").unwrap();
        assert_eq!(task.status, "pending");

        let result = store.move_task("task-002", "done");
        assert!(result.is_ok());

        let moved = result.unwrap();
        assert_eq!(moved.lane, "done");
        assert_eq!(moved.status, "completed");
    }
}
