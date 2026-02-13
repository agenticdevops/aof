//! Coordination types for multi-agent event streaming
//!
//! This module provides types for coordinating multiple agents through an event-driven
//! architecture. CoordinationEvent wraps ActivityEvent with routing metadata, enabling
//! event streaming to multiple subscribers via broadcast channels.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::activity::ActivityEvent;

/// Coordination event wrapper with routing metadata
///
/// Wraps an ActivityEvent with agent_id, session_id, and event_id for
/// multi-agent coordination. This enables event streaming, deduplication,
/// and session grouping across WebSocket connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEvent {
    /// The underlying activity event
    pub activity: ActivityEvent,
    /// Agent that emitted this event
    pub agent_id: String,
    /// Session grouping (UUID, generated once per daemon lifetime)
    pub session_id: String,
    /// Unique event ID (UUID v4, for deduplication)
    pub event_id: String,
    /// When the coordination event was created (may differ from activity timestamp)
    pub timestamp: DateTime<Utc>,
}

impl CoordinationEvent {
    /// Create a coordination event from an activity event
    ///
    /// Automatically generates a unique event_id (UUID v4) for deduplication.
    pub fn from_activity(
        activity: ActivityEvent,
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            activity,
            agent_id: agent_id.into(),
            session_id: session_id.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }

    /// Create event for agent started
    pub fn agent_started(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        let agent_id_str = agent_id.into();
        let activity = ActivityEvent::started(&agent_id_str);
        Self::from_activity(activity, agent_id_str, session_id)
    }

    /// Create event for agent completed
    pub fn agent_completed(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        let agent_id_str = agent_id.into();
        let activity = ActivityEvent::completed(duration_ms);
        Self::from_activity(activity, agent_id_str, session_id)
    }

    /// Create event for tool executing
    pub fn tool_executing(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
        tool_name: impl Into<String>,
        args: Option<String>,
    ) -> Self {
        let agent_id_str = agent_id.into();
        let activity = ActivityEvent::tool_executing(tool_name, args);
        Self::from_activity(activity, agent_id_str, session_id)
    }

    /// Create event for agent thinking
    pub fn thinking(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let agent_id_str = agent_id.into();
        let activity = ActivityEvent::thinking(message);
        Self::from_activity(activity, agent_id_str, session_id)
    }

    /// Create event for error
    pub fn error(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let agent_id_str = agent_id.into();
        let activity = ActivityEvent::error(message);
        Self::from_activity(activity, agent_id_str, session_id)
    }
}

/// Serializable session snapshot for persistence
///
/// Captures the complete state of a coordination session, including
/// agent states, pending tasks, and session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub session_id: String,
    /// Agent states keyed by agent_id
    pub agent_states: HashMap<String, AgentState>,
    /// Pending tasks
    pub task_queue: Vec<TaskInfo>,
    /// When session was created
    pub created_at: DateTime<Utc>,
    /// Last state update time
    pub last_updated: DateTime<Utc>,
}

impl SessionState {
    /// Create a new session state
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.into(),
            agent_states: HashMap::new(),
            task_queue: Vec::new(),
            created_at: now,
            last_updated: now,
        }
    }

    /// Update the last_updated timestamp
    pub fn touch(&mut self) {
        self.last_updated = Utc::now();
    }

    /// Add or update an agent state
    pub fn update_agent(&mut self, agent_id: String, state: AgentState) {
        self.agent_states.insert(agent_id, state);
        self.touch();
    }

    /// Add a task to the queue
    pub fn add_task(&mut self, task: TaskInfo) {
        self.task_queue.push(task);
        self.touch();
    }

    /// Remove a task by ID
    pub fn remove_task(&mut self, task_id: &str) -> Option<TaskInfo> {
        if let Some(pos) = self.task_queue.iter().position(|t| t.task_id == task_id) {
            self.touch();
            Some(self.task_queue.remove(pos))
        } else {
            None
        }
    }
}

/// State of an individual agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentState {
    /// Agent identifier
    pub agent_id: String,
    /// Current agent status
    pub status: AgentStatus,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Current task description (optional)
    pub current_task: Option<String>,
}

impl AgentState {
    /// Create a new agent state
    pub fn new(agent_id: impl Into<String>, status: AgentStatus) -> Self {
        Self {
            agent_id: agent_id.into(),
            status,
            last_activity: Utc::now(),
            current_task: None,
        }
    }

    /// Update status and refresh last_activity
    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_activity = Utc::now();
    }

    /// Set current task and update activity timestamp
    pub fn set_task(&mut self, task: impl Into<String>) {
        self.current_task = Some(task.into());
        self.last_activity = Utc::now();
    }

    /// Clear current task
    pub fn clear_task(&mut self) {
        self.current_task = None;
        self.last_activity = Utc::now();
    }
}

/// Agent status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is idle, waiting for work
    Idle,
    /// Agent is executing a task
    Running,
    /// Agent has completed its work
    Completed,
    /// Agent encountered an error
    Error,
    /// Agent disconnected from coordination layer
    Disconnected,
}

/// Task information for coordination queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskInfo {
    /// Unique task identifier
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Agent assigned to this task (optional)
    pub assigned_agent: Option<String>,
    /// Current task status
    pub status: TaskStatus,
    /// When task was created
    pub created_at: DateTime<Utc>,
}

impl TaskInfo {
    /// Create a new task
    pub fn new(task_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            description: description.into(),
            assigned_agent: None,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
        }
    }

    /// Assign task to an agent
    pub fn assign_to(&mut self, agent_id: impl Into<String>) {
        self.assigned_agent = Some(agent_id.into());
        self.status = TaskStatus::InProgress;
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
    }

    /// Mark task as failed
    pub fn fail(&mut self) {
        self.status = TaskStatus::Failed;
    }
}

/// Task status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending assignment
    Pending,
    /// Task is in progress
    InProgress,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Decision log entry for agent decision tracking
///
/// Records a decision made by an agent with reasoning, confidence, and contextual metadata.
/// Used for audit trails, team communication, and learning from agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLogEntry {
    /// Unique identifier for this decision
    pub event_id: String,
    /// Agent that made this decision
    pub agent_id: String,
    /// When the decision was made
    pub timestamp: DateTime<Utc>,
    /// Action taken (e.g., "classify_alert", "search_logs", "restart_pod")
    pub action: String,
    /// Reasoning behind the decision
    pub reasoning: String,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Tags for searchability (agent, action type, resource, severity)
    pub tags: Vec<String>,
    /// IDs of related decisions (for threading)
    pub related: Vec<String>,
    /// Action-specific context (alert_id, severity, matches, etc.)
    pub metadata: serde_json::Value,
}

impl DecisionLogEntry {
    /// Create a new decision log entry
    pub fn new(
        agent_id: impl Into<String>,
        action: impl Into<String>,
        reasoning: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            timestamp: Utc::now(),
            action: action.into(),
            reasoning: reasoning.into(),
            confidence: confidence.clamp(0.0, 1.0),
            tags: Vec::new(),
            related: Vec::new(),
            metadata: serde_json::json!({}),
        }
    }

    /// Add tags to the decision
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add related decision IDs
    pub fn with_related(mut self, related: Vec<String>) -> Self {
        self.related = related;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityType;

    #[test]
    fn test_coordination_event_from_activity() {
        let activity = ActivityEvent::new(ActivityType::Thinking, "Processing request");
        let event = CoordinationEvent::from_activity(activity.clone(), "agent-1", "session-123");

        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.session_id, "session-123");
        assert!(!event.event_id.is_empty());
        assert_eq!(event.activity.message, "Processing request");
    }

    #[test]
    fn test_coordination_event_unique_ids() {
        let activity1 = ActivityEvent::new(ActivityType::Thinking, "Task 1");
        let activity2 = ActivityEvent::new(ActivityType::Thinking, "Task 2");

        let event1 = CoordinationEvent::from_activity(activity1, "agent-1", "session-123");
        let event2 = CoordinationEvent::from_activity(activity2, "agent-1", "session-123");

        // Event IDs should be unique
        assert_ne!(event1.event_id, event2.event_id);
    }

    #[test]
    fn test_session_state_creation() {
        let state = SessionState::new("session-456");

        assert_eq!(state.session_id, "session-456");
        assert!(state.agent_states.is_empty());
        assert!(state.task_queue.is_empty());
    }

    #[test]
    fn test_session_state_serialization() {
        let mut state = SessionState::new("session-789");
        state.update_agent("agent-1".to_string(), AgentState::new("agent-1", AgentStatus::Running));
        state.add_task(TaskInfo::new("task-1", "Process data"));

        // Serialize to JSON
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("session-789"));
        assert!(json.contains("agent-1"));
        assert!(json.contains("Process data"));

        // Deserialize back
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "session-789");
        assert_eq!(deserialized.agent_states.len(), 1);
        assert_eq!(deserialized.task_queue.len(), 1);
    }

    #[test]
    fn test_agent_status_equality() {
        assert_eq!(AgentStatus::Idle, AgentStatus::Idle);
        assert_eq!(AgentStatus::Running, AgentStatus::Running);
        assert_ne!(AgentStatus::Idle, AgentStatus::Running);
    }

    #[test]
    fn test_agent_state_updates() {
        let mut agent = AgentState::new("agent-1", AgentStatus::Idle);

        agent.update_status(AgentStatus::Running);
        assert_eq!(agent.status, AgentStatus::Running);

        agent.set_task("Analyzing logs");
        assert_eq!(agent.current_task, Some("Analyzing logs".to_string()));

        agent.clear_task();
        assert_eq!(agent.current_task, None);
    }

    #[test]
    fn test_task_info_lifecycle() {
        let mut task = TaskInfo::new("task-1", "Deploy application");

        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.assigned_agent, None);

        task.assign_to("agent-1");
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.assigned_agent, Some("agent-1".to_string()));

        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_session_state_task_management() {
        let mut state = SessionState::new("session-1");

        let task1 = TaskInfo::new("task-1", "Task 1");
        let task2 = TaskInfo::new("task-2", "Task 2");

        state.add_task(task1);
        state.add_task(task2);
        assert_eq!(state.task_queue.len(), 2);

        let removed = state.remove_task("task-1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().task_id, "task-1");
        assert_eq!(state.task_queue.len(), 1);

        let not_found = state.remove_task("task-999");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_convenience_constructor_agent_started() {
        let event = CoordinationEvent::agent_started("agent-1", "session-123");
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.session_id, "session-123");
        assert_eq!(event.activity.activity_type, ActivityType::Started);
    }

    #[test]
    fn test_convenience_constructor_agent_completed() {
        let event = CoordinationEvent::agent_completed("agent-1", "session-123", 5000);
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.activity.activity_type, ActivityType::Completed);
        assert_eq!(
            event.activity.details.as_ref().unwrap().duration_ms,
            Some(5000)
        );
    }

    #[test]
    fn test_convenience_constructor_tool_executing() {
        let event = CoordinationEvent::tool_executing(
            "agent-1",
            "session-123",
            "kubectl",
            Some("get pods".to_string()),
        );
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.activity.activity_type, ActivityType::ToolExecuting);
        let details = event.activity.details.as_ref().unwrap();
        assert_eq!(details.tool_name, Some("kubectl".to_string()));
    }

    #[test]
    fn test_convenience_constructor_thinking() {
        let event = CoordinationEvent::thinking("agent-1", "session-123", "Analyzing data");
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.activity.activity_type, ActivityType::Thinking);
        assert_eq!(event.activity.message, "Analyzing data");
    }

    #[test]
    fn test_convenience_constructor_error() {
        let event = CoordinationEvent::error("agent-1", "session-123", "Connection failed");
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.activity.activity_type, ActivityType::Error);
        assert_eq!(event.activity.message, "Connection failed");
    }

    #[test]
    fn test_decision_log_entry_creation() {
        let entry = DecisionLogEntry::new("agent-1", "restart_pod", "Pod was unhealthy", 0.95);

        assert_eq!(entry.agent_id, "agent-1");
        assert_eq!(entry.action, "restart_pod");
        assert_eq!(entry.reasoning, "Pod was unhealthy");
        assert_eq!(entry.confidence, 0.95);
        assert!(!entry.event_id.is_empty());
        assert!(entry.tags.is_empty());
        assert!(entry.related.is_empty());
    }

    #[test]
    fn test_decision_log_entry_with_tags() {
        let entry = DecisionLogEntry::new("agent-1", "search_logs", "Searching for errors", 0.85)
            .with_tags(vec!["incident".to_string(), "logs".to_string()]);

        assert_eq!(entry.tags.len(), 2);
        assert!(entry.tags.contains(&"incident".to_string()));
        assert!(entry.tags.contains(&"logs".to_string()));
    }

    #[test]
    fn test_decision_log_entry_with_related() {
        let entry = DecisionLogEntry::new("agent-1", "escalate", "Escalating to human", 0.6)
            .with_related(vec!["decision-001".to_string(), "decision-002".to_string()]);

        assert_eq!(entry.related.len(), 2);
    }

    #[test]
    fn test_decision_log_entry_confidence_clamping() {
        let entry_high = DecisionLogEntry::new("agent-1", "action", "test", 1.5);
        assert_eq!(entry_high.confidence, 1.0);

        let entry_low = DecisionLogEntry::new("agent-1", "action", "test", -0.5);
        assert_eq!(entry_low.confidence, 0.0);
    }

    #[test]
    fn test_decision_log_entry_serialization() {
        let entry = DecisionLogEntry::new("agent-1", "classify", "Alert is SEV2", 0.88)
            .with_tags(vec!["incident".to_string()])
            .with_metadata(serde_json::json!({
                "alert_id": "ALT-001",
                "severity": "SEV2"
            }));

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DecisionLogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_id, "agent-1");
        assert_eq!(deserialized.action, "classify");
        assert_eq!(deserialized.confidence, 0.88);
        assert_eq!(deserialized.tags.len(), 1);
    }
}
