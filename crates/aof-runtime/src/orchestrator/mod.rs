//! RuntimeOrchestrator - Task orchestration and scheduling
//!
//! Coordinates multiple tasks and agents, providing advanced scheduling
//! and execution management capabilities.

use crate::task::{Task, TaskHandle, TaskResult, TaskStatus};
use aof_core::{AofError, AofResult};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Runtime orchestrator for task management
///
/// Provides:
/// - Task queuing and scheduling
/// - Concurrent execution with limits
/// - Task monitoring and cancellation
/// - Priority-based execution
pub struct RuntimeOrchestrator {
    /// Active tasks
    tasks: Arc<DashMap<String, Arc<TaskHandle>>>,

    /// Concurrency limiter
    semaphore: Arc<Semaphore>,

    /// Max concurrent tasks
    max_concurrent: usize,
}

impl RuntimeOrchestrator {
    /// Create a new orchestrator
    pub fn new() -> Self {
        Self::with_max_concurrent(10)
    }

    /// Create orchestrator with custom concurrency limit
    pub fn with_max_concurrent(max_concurrent: usize) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    /// Submit a task for execution
    ///
    /// Returns a task handle that can be used to monitor progress
    pub fn submit_task(&self, task: Task) -> Arc<TaskHandle> {
        let task_id = task.id.clone();
        let handle = Arc::new(TaskHandle::new(task));

        self.tasks.insert(task_id.clone(), Arc::clone(&handle));
        info!("Task submitted: {}", task_id);

        handle
    }

    /// Execute a task asynchronously
    ///
    /// This starts the task execution in the background
    pub async fn execute_task<F, Fut>(
        &self,
        task_id: &str,
        executor: F,
    ) -> AofResult<Arc<TaskHandle>>
    where
        F: FnOnce(Task) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = AofResult<String>> + Send + 'static,
    {
        let handle = self
            .tasks
            .get(task_id)
            .map(|h| Arc::clone(h.value()))
            .ok_or_else(|| AofError::agent(format!("Task not found: {}", task_id)))?;

        let semaphore = Arc::clone(&self.semaphore);
        let handle_clone: Arc<TaskHandle> = Arc::clone(&handle);

        // Spawn task execution
        tokio::spawn(async move {
            // Acquire semaphore permit
            let _permit = semaphore.acquire().await.unwrap();

            let task = handle_clone.task().await;
            let task_id = task.id.clone();

            handle_clone.update_status(TaskStatus::Running).await;
            debug!("Task started: {}", task_id);

            let start = std::time::Instant::now();

            // Execute task
            match executor(task).await {
                Ok(output) => {
                    let result = TaskResult::success(task_id.clone(), output)
                        .with_execution_time(start.elapsed().as_millis() as u64);

                    handle_clone.set_result(result).await;
                    handle_clone.update_status(TaskStatus::Completed).await;
                    info!("Task completed: {}", task_id);
                }
                Err(e) => {
                    let result = TaskResult::failure(task_id.clone(), e.to_string())
                        .with_execution_time(start.elapsed().as_millis() as u64);

                    handle_clone.set_result(result).await;
                    handle_clone.update_status(TaskStatus::Failed).await;
                    warn!("Task failed: {} - {}", task_id, e);
                }
            }
        });

        Ok(handle)
    }

    /// Get task handle by ID
    pub fn get_task(&self, task_id: &str) -> Option<Arc<TaskHandle>> {
        self.tasks.get(task_id).map(|h| Arc::clone(h.value()))
    }

    /// List all task IDs
    pub fn list_tasks(&self) -> Vec<String> {
        self.tasks.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get tasks by status
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Arc<TaskHandle>> {
        let mut result = Vec::new();

        for entry in self.tasks.iter() {
            let handle = entry.value();
            if handle.status().await == status {
                result.push(Arc::clone(handle));
            }
        }

        result
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str) -> AofResult<()> {
        if let Some(handle) = self.get_task(task_id) {
            let status = handle.status().await;

            if status == TaskStatus::Pending || status == TaskStatus::Running {
                handle.update_status(TaskStatus::Cancelled).await;
                let result = TaskResult::failure(
                    task_id.to_string(),
                    "Task cancelled by user".to_string(),
                );
                handle.set_result(result).await;
                info!("Task cancelled: {}", task_id);
                Ok(())
            } else {
                Err(AofError::agent(format!(
                    "Cannot cancel task in status: {:?}",
                    status
                )))
            }
        } else {
            Err(AofError::agent(format!("Task not found: {}", task_id)))
        }
    }

    /// Remove completed/failed tasks from tracking
    pub async fn cleanup_finished_tasks(&self) {
        let mut to_remove = Vec::new();

        for entry in self.tasks.iter() {
            let handle = entry.value();
            let status = handle.status().await;

            if matches!(
                status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ) {
                to_remove.push(entry.key().clone());
            }
        }

        for task_id in to_remove {
            self.tasks.remove(&task_id);
            debug!("Cleaned up task: {}", task_id);
        }
    }

    /// Get orchestrator statistics
    pub async fn stats(&self) -> OrchestratorStats {
        let mut stats = OrchestratorStats::default();

        for entry in self.tasks.iter() {
            let status = entry.value().status().await;
            match status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Running => stats.running += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
        }

        stats.max_concurrent = self.max_concurrent;
        stats.available_permits = self.semaphore.available_permits();

        stats
    }
}

impl Default for RuntimeOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrator statistics
#[derive(Debug, Clone, Default)]
pub struct OrchestratorStats {
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub max_concurrent: usize,
    pub available_permits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = RuntimeOrchestrator::new();
        let stats = orchestrator.stats().await;

        assert_eq!(stats.pending, 0);
        assert_eq!(stats.running, 0);
        assert_eq!(stats.max_concurrent, 10);
    }

    #[tokio::test]
    async fn test_submit_task() {
        let orchestrator = RuntimeOrchestrator::new();

        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        );

        let handle = orchestrator.submit_task(task);
        assert_eq!(handle.status().await, TaskStatus::Pending);

        let stats = orchestrator.stats().await;
        assert_eq!(stats.pending, 1);
    }

    #[tokio::test]
    async fn test_execute_task() {
        let orchestrator = RuntimeOrchestrator::new();

        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        );

        let handle = orchestrator.submit_task(task);

        orchestrator
            .execute_task("task-1", |_task| async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                Ok("Success!".to_string())
            })
            .await
            .unwrap();

        // Wait a bit for task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let status = handle.status().await;
        assert_eq!(status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let orchestrator = RuntimeOrchestrator::new();

        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "test-agent".to_string(),
            "test input".to_string(),
        );

        orchestrator.submit_task(task);
        orchestrator.cancel_task("task-1").await.unwrap();

        let handle = orchestrator.get_task("task-1").unwrap();
        assert_eq!(handle.status().await, TaskStatus::Cancelled);
    }
}
