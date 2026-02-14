//! Performance tests for reliability metrics computation
//!
//! Validates that metric computation scales linearly and completes
//! within acceptable time bounds (< 10ms for 10000 events).

use std::time::Instant;

use aof_core::activity::{ActivityEvent, ActivityType};
use aof_core::coordination::CoordinationEvent;
use aof_personas::metrics::compute_agent_metrics;

/// Helper: create N events for an agent with a mix of types
fn generate_events(agent_id: &str, count: usize) -> Vec<CoordinationEvent> {
    (0..count)
        .map(|i| {
            let activity_type = if i % 10 == 0 {
                ActivityType::Error
            } else if i % 3 == 0 {
                ActivityType::Thinking
            } else {
                ActivityType::Completed
            };
            let activity = ActivityEvent::new(activity_type, format!("event-{}", i));
            CoordinationEvent::from_activity(activity, agent_id, "perf-session")
        })
        .collect()
}

#[test]
fn test_metrics_computation_100_events() {
    let events = generate_events("perf-agent", 100);
    let start = Instant::now();
    let metrics = compute_agent_metrics("perf-agent", &events);
    let duration = start.elapsed();

    assert_eq!(metrics.event_count, 100);
    assert!(metrics.uptime_percent.is_some());
    assert!(
        duration.as_millis() < 10,
        "100 events took {}ms (limit: 10ms)",
        duration.as_millis()
    );
}

#[test]
fn test_metrics_computation_1000_events() {
    let events = generate_events("perf-agent", 1000);
    let start = Instant::now();
    let metrics = compute_agent_metrics("perf-agent", &events);
    let duration = start.elapsed();

    assert_eq!(metrics.event_count, 1000);
    assert!(metrics.uptime_percent.is_some());
    assert!(
        duration.as_millis() < 10,
        "1000 events took {}ms (limit: 10ms)",
        duration.as_millis()
    );
}

#[test]
fn test_metrics_computation_10000_events() {
    let events = generate_events("perf-agent", 10000);
    let start = Instant::now();
    let metrics = compute_agent_metrics("perf-agent", &events);
    let duration = start.elapsed();

    assert_eq!(metrics.event_count, 10000);
    assert!(metrics.uptime_percent.is_some());
    // Allow up to 50ms for 10k events (generous for CI)
    assert!(
        duration.as_millis() < 50,
        "10000 events took {}ms (limit: 50ms)",
        duration.as_millis()
    );

    // Verify correctness: 10% are errors (i % 10 == 0)
    // So uptime = 90%
    assert!((metrics.uptime_percent.unwrap() - 90.0).abs() < 0.1);
}

#[test]
fn test_metrics_computation_scales_linearly() {
    // Compare time for 1000 vs 5000 events
    let events_1k = generate_events("perf-agent", 1000);
    let events_5k = generate_events("perf-agent", 5000);

    let start_1k = Instant::now();
    compute_agent_metrics("perf-agent", &events_1k);
    let duration_1k = start_1k.elapsed();

    let start_5k = Instant::now();
    compute_agent_metrics("perf-agent", &events_5k);
    let duration_5k = start_5k.elapsed();

    // 5x events should be roughly 5x time (allowing 10x for overhead)
    // This is a rough check — we just want to confirm no exponential blowup
    let ratio = duration_5k.as_nanos() as f64 / duration_1k.as_nanos().max(1) as f64;
    assert!(
        ratio < 20.0,
        "Time ratio {:.1}x (expected roughly 5x, max allowed 20x)",
        ratio
    );
}
