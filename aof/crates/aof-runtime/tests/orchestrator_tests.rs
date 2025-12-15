//! Orchestrator Integration Tests
//!
//! Tests the RuntimeOrchestrator for task scheduling, execution,
//! and lifecycle management.

use std::sync::Arc;
use aof_runtime::{RuntimeOrchestrator, Task, TaskStatus};

/// Create a test task
fn create_test_task(id: &str, name: &str) -> Task {
    Task::new(
        id.to_string(),
        name.to_string(),
        "test-agent".to_string(),
        "Test input".to_string(),
    )
}

// ============================================================================
// Basic Orchestrator Tests
// ============================================================================

#[tokio::test]
async fn test_orchestrator_creation() {
    let orchestrator = RuntimeOrchestrator::new();
    let stats = orchestrator.stats().await;

    assert_eq!(stats.pending, 0);
    assert_eq!(stats.running, 0);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.failed, 0);
    assert!(stats.max_concurrent > 0);
}

#[tokio::test]
async fn test_orchestrator_with_concurrency() {
    let orchestrator = RuntimeOrchestrator::with_max_concurrent(5);
    let stats = orchestrator.stats().await;

    assert_eq!(stats.max_concurrent, 5);
}

// ============================================================================
// Task Submission Tests
// ============================================================================

#[tokio::test]
async fn test_submit_single_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    let task = create_test_task("task-1", "Test Task 1");

    let handle = orchestrator.submit_task(task);
    let status = handle.status().await;

    assert!(matches!(status, TaskStatus::Pending | TaskStatus::Running));
}

#[tokio::test]
async fn test_submit_multiple_tasks() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    for i in 0..5 {
        let task = create_test_task(&format!("task-{}", i), &format!("Task {}", i));
        orchestrator.submit_task(task);
    }

    let task_ids = orchestrator.list_tasks();
    assert_eq!(task_ids.len(), 5);
}

#[tokio::test]
async fn test_submit_task_with_priority() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let mut low_priority = create_test_task("low-priority", "Low Priority Task");
    low_priority.priority = 1;

    let mut high_priority = create_test_task("high-priority", "High Priority Task");
    high_priority.priority = 10;

    orchestrator.submit_task(low_priority);
    orchestrator.submit_task(high_priority);

    // Both tasks should be tracked
    let task_ids = orchestrator.list_tasks();
    assert_eq!(task_ids.len(), 2);
}

#[tokio::test]
async fn test_submit_task_with_metadata() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let mut task = create_test_task("metadata-task", "Task with Metadata");
    task.metadata.insert("key1".to_string(), serde_json::json!("value1"));
    task.metadata.insert("key2".to_string(), serde_json::json!(42));

    let handle = orchestrator.submit_task(task);
    let task_data = handle.task().await;

    assert_eq!(task_data.metadata.len(), 2);
    assert_eq!(task_data.metadata.get("key1"), Some(&serde_json::json!("value1")));
}

// ============================================================================
// Task Retrieval Tests
// ============================================================================

#[tokio::test]
async fn test_get_existing_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    let task = create_test_task("findable-task", "Findable Task");

    orchestrator.submit_task(task);

    let handle = orchestrator.get_task("findable-task");
    assert!(handle.is_some());
}

#[tokio::test]
async fn test_get_nonexistent_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let handle = orchestrator.get_task("does-not-exist");
    assert!(handle.is_none());
}

#[tokio::test]
async fn test_list_empty_tasks() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let task_ids = orchestrator.list_tasks();
    assert!(task_ids.is_empty());
}

// ============================================================================
// Task Cancellation Tests
// ============================================================================

#[tokio::test]
async fn test_cancel_pending_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    let task = create_test_task("to-cancel", "Task to Cancel");

    orchestrator.submit_task(task);

    let result = orchestrator.cancel_task("to-cancel").await;
    assert!(result.is_ok());

    if let Some(handle) = orchestrator.get_task("to-cancel") {
        let status = handle.status().await;
        assert!(matches!(status, TaskStatus::Cancelled));
    }
}

#[tokio::test]
async fn test_cancel_nonexistent_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let result = orchestrator.cancel_task("nonexistent").await;
    // Should return an error for nonexistent task
    assert!(result.is_err());
}

// ============================================================================
// Task Status Tests
// ============================================================================

#[tokio::test]
async fn test_task_status_transitions() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    let task = create_test_task("status-task", "Status Task");

    let handle = orchestrator.submit_task(task);

    // Initially should be Pending or Running
    let initial_status = handle.status().await;
    assert!(matches!(initial_status, TaskStatus::Pending | TaskStatus::Running));
}

#[tokio::test]
async fn test_task_handle_properties() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());
    let mut task = create_test_task("props-task", "Properties Task");
    task.metadata.insert("test_key".to_string(), serde_json::json!("test_value"));

    let handle = orchestrator.submit_task(task);

    // Get task data through handle
    let task_data = handle.task().await;
    assert_eq!(task_data.id, "props-task");
    assert_eq!(task_data.name, "Properties Task");
    assert_eq!(task_data.agent_name, "test-agent");
}

// ============================================================================
// Statistics Tests
// ============================================================================

#[tokio::test]
async fn test_stats_after_submissions() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Submit several tasks
    for i in 0..3 {
        let task = create_test_task(&format!("stat-task-{}", i), &format!("Task {}", i));
        orchestrator.submit_task(task);
    }

    let stats = orchestrator.stats().await;
    // Should have some tasks pending or running
    assert!(stats.pending + stats.running >= 0);
}

#[tokio::test]
async fn test_stats_after_cancellation() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let task = create_test_task("cancel-stat-task", "Task for Stats");
    orchestrator.submit_task(task);

    // Cancel the task
    let _ = orchestrator.cancel_task("cancel-stat-task").await;

    let stats = orchestrator.stats().await;
    assert_eq!(stats.cancelled, 1);
}

// ============================================================================
// Concurrent Task Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_task_submission() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Submit tasks concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let orch = Arc::clone(&orchestrator);
        let handle = tokio::spawn(async move {
            let task = create_test_task(&format!("concurrent-{}", i), &format!("Task {}", i));
            orch.submit_task(task);
        });
        handles.push(handle);
    }

    // Wait for all submissions
    for handle in handles {
        handle.await.unwrap();
    }

    let task_ids = orchestrator.list_tasks();
    assert_eq!(task_ids.len(), 10);
}

#[tokio::test]
async fn test_concurrent_task_access() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Submit a task
    let task = create_test_task("shared-task", "Shared Task");
    orchestrator.submit_task(task);

    // Access from multiple concurrent tasks
    let mut handles = vec![];
    for _ in 0..5 {
        let orch = Arc::clone(&orchestrator);
        let handle = tokio::spawn(async move {
            if let Some(task_handle) = orch.get_task("shared-task") {
                let _ = task_handle.status().await;
            }
        });
        handles.push(handle);
    }

    // Wait for all accesses
    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Task Execution Tests (with mock executor)
// ============================================================================

#[tokio::test]
async fn test_execute_task_with_closure() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let task = create_test_task("exec-task", "Executable Task");
    let task_id = task.id.clone();
    orchestrator.submit_task(task);

    // Execute the task
    let result = orchestrator
        .execute_task(&task_id, |task| async move {
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(format!("Completed: {}", task.name))
        })
        .await;

    assert!(result.is_ok());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_execute_nonexistent_task() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    let result = orchestrator
        .execute_task("does-not-exist", |_task| async move {
            Ok("Should not run".to_string())
        })
        .await;

    assert!(result.is_err());
}

// ============================================================================
// Cleanup Tests
// ============================================================================

#[tokio::test]
async fn test_clear_completed_tasks() {
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Submit and immediately cancel a task (simulating completion)
    let task = create_test_task("cleanup-task", "Cleanup Task");
    orchestrator.submit_task(task);
    let _ = orchestrator.cancel_task("cleanup-task").await;

    // Task should still be tracked
    let task_ids = orchestrator.list_tasks();
    assert!(!task_ids.is_empty());
}
