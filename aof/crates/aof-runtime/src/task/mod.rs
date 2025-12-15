//! Task management module
//!
//! Provides task scheduling and execution coordination for agents.

use aof_core::{AgentContext, AofResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Task execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: String,

    /// Task name/description
    pub name: String,

    /// Current status
    pub status: TaskStatus,

    /// Agent name to execute
    pub agent_name: String,

    /// Input for the agent
    pub input: String,

    /// Task priority (higher = more important)
    #[serde(default)]
    pub priority: u32,

    /// Task metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Task {
    /// Create a new task
    pub fn new(id: String, name: String, agent_name: String, input: String) -> Self {
        Self {
            id,
            name,
            status: TaskStatus::Pending,
            agent_name,
            input,
            priority: 0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Task handle for async operations
pub struct TaskHandle {
    task: Arc<RwLock<Task>>,
    result: Arc<RwLock<Option<TaskResult>>>,
}

impl TaskHandle {
    /// Create a new task handle
    pub fn new(task: Task) -> Self {
        Self {
            task: Arc::new(RwLock::new(task)),
            result: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current task status
    pub async fn status(&self) -> TaskStatus {
        self.task.read().await.status
    }

    /// Get task information
    pub async fn task(&self) -> Task {
        self.task.read().await.clone()
    }

    /// Wait for task completion
    pub async fn wait(&self) -> AofResult<TaskResult> {
        loop {
            let status = self.status().await;
            match status {
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    if let Some(result) = self.result.read().await.clone() {
                        return Ok(result);
                    }
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Update task status
    pub async fn update_status(&self, status: TaskStatus) {
        self.task.write().await.status = status;
    }

    /// Set task result
    pub async fn set_result(&self, result: TaskResult) {
        *self.result.write().await = Some(result);
    }
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,

    /// Success status
    pub success: bool,

    /// Output/result
    pub output: String,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution time (ms)
    pub execution_time_ms: u64,

    /// Token usage
    #[serde(default)]
    pub input_tokens: usize,

    #[serde(default)]
    pub output_tokens: usize,
}

impl TaskResult {
    /// Create a successful result
    pub fn success(task_id: String, output: String) -> Self {
        Self {
            task_id,
            success: true,
            output,
            error: None,
            execution_time_ms: 0,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    /// Create a failed result
    pub fn failure(task_id: String, error: String) -> Self {
        Self {
            task_id,
            success: false,
            output: String::new(),
            error: Some(error),
            execution_time_ms: 0,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, input_tokens: usize, output_tokens: usize) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        );

        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, 0);
    }

    #[test]
    fn test_task_with_priority() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        )
        .with_priority(10);

        assert_eq!(task.priority, 10);
    }

    #[tokio::test]
    async fn test_task_handle() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        );

        let handle = TaskHandle::new(task);
        assert_eq!(handle.status().await, TaskStatus::Pending);

        handle.update_status(TaskStatus::Running).await;
        assert_eq!(handle.status().await, TaskStatus::Running);
    }

    #[test]
    fn test_task_result_success() {
        let result = TaskResult::success("task-1".to_string(), "Success!".to_string());
        assert!(result.success);
        assert_eq!(result.output, "Success!");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let result = TaskResult::failure("task-1".to_string(), "Failed!".to_string());
        assert!(!result.success);
        assert_eq!(result.error, Some("Failed!".to_string()));
    }
}
