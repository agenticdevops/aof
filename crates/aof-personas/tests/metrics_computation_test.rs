//! Integration tests for reliability metrics computation
//!
//! Tests the full metrics pipeline: event creation -> computation -> cache -> retrieval.
//! Validates correctness, edge cases, concurrent access, and performance.

use std::sync::Arc;

use aof_core::activity::{ActivityEvent, ActivityType};
use aof_core::coordination::CoordinationEvent;
use aof_personas::metrics::{compute_agent_metrics, ReliabilityCache};

/// Helper: create a CoordinationEvent with a specific activity type and agent
fn make_event(agent_id: &str, activity_type: ActivityType, message: &str) -> CoordinationEvent {
    let activity = ActivityEvent::new(activity_type, message);
    CoordinationEvent::from_activity(activity, agent_id, "integration-session")
}

/// Test 1: Uptime computation with all success events
#[test]
fn test_uptime_computation_all_success() {
    let events: Vec<CoordinationEvent> = (0..10)
        .map(|i| make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i)))
        .collect();

    let metrics = compute_agent_metrics("k8s-monitor", &events);

    assert_eq!(metrics.event_count, 10);
    assert!(metrics.uptime_percent.is_some());
    assert!((metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
    assert!((metrics.success_rate.unwrap() - 100.0).abs() < 0.1);
    assert!(metrics.last_error.is_none());
}

/// Test 2: Uptime computation with error events
#[test]
fn test_uptime_computation_with_errors() {
    let mut events = Vec::new();
    // 8 success + 2 errors = 80% uptime
    for i in 0..8 {
        events.push(make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i)));
    }
    events.push(make_event("k8s-monitor", ActivityType::Error, "error-1"));
    events.push(make_event("k8s-monitor", ActivityType::Error, "error-2"));

    let metrics = compute_agent_metrics("k8s-monitor", &events);

    assert_eq!(metrics.event_count, 10);
    assert!((metrics.uptime_percent.unwrap() - 80.0).abs() < 0.1);
    assert!(metrics.last_error.is_some());
}

/// Test 3: Success rate computation (completed vs total)
#[test]
fn test_success_rate_computation() {
    let mut events = Vec::new();
    // 7 completed + 3 thinking (non-completed, non-error) = 70% success rate
    for i in 0..7 {
        events.push(make_event("k8s-monitor", ActivityType::Completed, &format!("completed-{}", i)));
    }
    for i in 0..3 {
        events.push(make_event("k8s-monitor", ActivityType::Thinking, &format!("thinking-{}", i)));
    }

    let metrics = compute_agent_metrics("k8s-monitor", &events);

    assert_eq!(metrics.event_count, 10);
    // Uptime should be 100% (no errors)
    assert!((metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
    // Success rate = 7/10 = 70%
    assert!((metrics.success_rate.unwrap() - 70.0).abs() < 0.1);
}

/// Test 4: Insufficient data handling (<10 events)
#[test]
fn test_insufficient_data_handling() {
    let events: Vec<CoordinationEvent> = (0..9)
        .map(|i| make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i)))
        .collect();

    let metrics = compute_agent_metrics("k8s-monitor", &events);

    assert_eq!(metrics.event_count, 9);
    assert!(metrics.uptime_percent.is_none(), "Should be None for <10 events");
    assert!(metrics.success_rate.is_none(), "Should be None for <10 events");
}

/// Test 5: Cache updates with new events
#[tokio::test]
async fn test_cache_updates_with_events() {
    let cache = ReliabilityCache::new(1000);

    // Initially no metrics
    assert!(cache.get_metrics("k8s-monitor").await.is_none());

    // Add events one by one
    for i in 0..10 {
        let event = make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    // Now metrics should be available
    let metrics = cache.get_metrics("k8s-monitor").await.unwrap();
    assert_eq!(metrics.event_count, 10);
    assert!((metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);

    // Add an error event
    let error_event = make_event("k8s-monitor", ActivityType::Error, "critical failure");
    cache.update_with_event(&error_event).await.unwrap();

    // Metrics should update
    let updated = cache.get_metrics("k8s-monitor").await.unwrap();
    assert_eq!(updated.event_count, 11);
    // Uptime = (11-1)/11 * 100 = ~90.9%
    assert!((updated.uptime_percent.unwrap() - 90.909).abs() < 0.1);
    assert!(updated.last_error.is_some());
}

/// Test 6: API endpoint returns metrics JSON shape
/// (Tests the serialization format, not HTTP layer)
#[test]
fn test_metrics_json_shape() {
    let mut events = Vec::new();
    for i in 0..15 {
        events.push(make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i)));
    }
    events.push(make_event("k8s-monitor", ActivityType::Error, "error-1"));

    let metrics = compute_agent_metrics("k8s-monitor", &events);
    let json = serde_json::to_value(&metrics).unwrap();

    // Verify JSON shape matches API contract
    assert_eq!(json["agent_id"], "k8s-monitor");
    assert!(json["uptime_percent"].is_number());
    assert!(json["success_rate"].is_number());
    assert_eq!(json["event_count"], 16);
    assert!(json["last_update"].is_string());
    assert!(json["last_error"].is_string()); // Has an error
}

/// Test 7: 404 for missing agent (cache returns None)
#[tokio::test]
async fn test_missing_agent_returns_none() {
    let cache = ReliabilityCache::new(1000);

    // Add events for one agent only
    for i in 0..10 {
        let event = make_event("existing-agent", ActivityType::Completed, &format!("task-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    // Nonexistent agent should return None
    let result = cache.get_metrics("nonexistent-agent").await;
    assert!(result.is_none());

    // Existing agent should return Some
    let result = cache.get_metrics("existing-agent").await;
    assert!(result.is_some());
}

/// Test 8: Metrics version header increments on updates
#[tokio::test]
async fn test_metrics_version_increments() {
    let cache = ReliabilityCache::new(1000);

    let v0 = cache.version();
    assert_eq!(v0, 0);

    let event1 = make_event("agent-1", ActivityType::Completed, "task-1");
    cache.update_with_event(&event1).await.unwrap();
    let v1 = cache.version();
    assert_eq!(v1, 1);

    let event2 = make_event("agent-1", ActivityType::Error, "err-1");
    cache.update_with_event(&event2).await.unwrap();
    let v2 = cache.version();
    assert_eq!(v2, 2);

    // Version should strictly increase
    assert!(v2 > v1);
    assert!(v1 > v0);
}

/// Test 9: Concurrent metric reads don't block each other
#[tokio::test]
async fn test_concurrent_metric_reads() {
    let cache = Arc::new(ReliabilityCache::new(1000));

    // Populate cache
    for i in 0..20 {
        let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    // Spawn 10 concurrent readers
    let mut handles = Vec::new();
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            let m = cache_clone.get_metrics("agent-1").await;
            assert!(m.is_some());
            assert_eq!(m.unwrap().event_count, 20);
        }));
    }

    // All should complete without deadlock or error
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test 10: Last error timestamp is accurate
#[test]
fn test_last_error_timestamp_accurate() {
    let mut events = Vec::new();
    for i in 0..5 {
        events.push(make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)));
    }

    // Add an error — capture its timestamp
    let error_event = make_event("agent-1", ActivityType::Error, "the-error");
    let expected_error_ts = error_event.timestamp;
    events.push(error_event);

    // Add more success events after the error
    for i in 5..10 {
        events.push(make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)));
    }

    let metrics = compute_agent_metrics("agent-1", &events);

    assert!(metrics.last_error.is_some());
    assert_eq!(metrics.last_error.unwrap(), expected_error_ts);
    assert_eq!(metrics.event_count, 11);
}

/// Test 11: Multiple agents tracked independently
#[tokio::test]
async fn test_multiple_agents_independent() {
    let cache = ReliabilityCache::new(1000);

    // Agent 1: all success
    for i in 0..10 {
        let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    // Agent 2: all errors
    for i in 0..10 {
        let event = make_event("agent-2", ActivityType::Error, &format!("err-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    let m1 = cache.get_metrics("agent-1").await.unwrap();
    let m2 = cache.get_metrics("agent-2").await.unwrap();

    // Agent 1: 100% uptime, 100% success
    assert!((m1.uptime_percent.unwrap() - 100.0).abs() < 0.1);
    assert!((m1.success_rate.unwrap() - 100.0).abs() < 0.1);
    assert!(m1.last_error.is_none());

    // Agent 2: 0% uptime, 0% success
    assert!((m2.uptime_percent.unwrap() - 0.0).abs() < 0.1);
    assert!((m2.success_rate.unwrap() - 0.0).abs() < 0.1);
    assert!(m2.last_error.is_some());
}
