use aof_coordination::EventBroadcaster;
use aof_core::{ActivityEvent, ActivityType, CoordinationEvent};
use chrono::Utc;
use std::time::{Duration, Instant};

fn create_test_event(id: usize) -> CoordinationEvent {
    CoordinationEvent {
        activity: ActivityEvent {
            activity_type: ActivityType::Thinking,
            message: format!("Test event {}", id),
            timestamp: Utc::now(),
            details: None,
        },
        agent_id: "test-agent".to_string(),
        session_id: "session-123".to_string(),
        event_id: format!("evt-{}", id),
        timestamp: Utc::now(),
        introduction: None,
        coordination_activity: None,
    }
}

#[tokio::test]
async fn test_baseline_event_emission_latency() {
    let broadcaster = EventBroadcaster::new(1000);
    let mut rx = broadcaster.subscribe();

    let mut latencies = Vec::new();

    for i in 0..100 {
        let start = Instant::now();

        // Emit event
        broadcaster.emit(create_test_event(i));

        // Receive event
        let _event = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Failed to receive event");

        let latency = start.elapsed();
        latencies.push(latency);
    }

    // Calculate p95 latency
    latencies.sort();
    let p95_index = (latencies.len() as f64 * 0.95) as usize;
    let p95_latency = latencies[p95_index.min(latencies.len() - 1)];

    println!("P95 latency: {:?}", p95_latency);

    // Assert p95 < 50ms
    assert!(
        p95_latency < Duration::from_millis(50),
        "P95 latency {:?} exceeds 50ms threshold",
        p95_latency
    );
}

#[tokio::test]
async fn test_baseline_session_persistence_roundtrip() {
    use aof_coordination::SessionPersistence;
    use std::collections::HashMap;
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let persistence = SessionPersistence::new(temp_dir.path().to_path_buf());

    // Create session state with 10 events
    let session_id = "session-123".to_string();
    let events: Vec<CoordinationEvent> = (0..10).map(create_test_event).collect();

    let mut state = HashMap::new();
    state.insert(session_id.clone(), events);

    // Measure round-trip time
    let start = Instant::now();

    // Save
    persistence
        .save_session(&session_id, &state[&session_id])
        .await
        .expect("Failed to save session");

    // Restore
    let restored = persistence
        .restore_session(&session_id)
        .await
        .expect("Failed to restore session");

    let roundtrip_time = start.elapsed();

    println!("Round-trip time: {:?}", roundtrip_time);

    // Verify correctness
    assert_eq!(restored.len(), 10, "Should restore all 10 events");

    // Assert < 50ms
    assert!(
        roundtrip_time < Duration::from_millis(50),
        "Round-trip time {:?} exceeds 50ms threshold",
        roundtrip_time
    );
}
