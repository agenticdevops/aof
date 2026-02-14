use aof_coordination::{EventBroadcaster, SessionPersistence};
use aof_core::{ActivityEvent, ActivityType, CoordinationEvent};
use chrono::Utc;
use std::sync::Arc;

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

// Helper to get current memory usage (RSS) in bytes
// Note: This is a rough approximation. For production, use a proper memory profiler.
fn get_memory_usage() -> usize {
    // On Unix systems, we can read /proc/self/statm
    #[cfg(target_os = "linux")]
    {
        let statm = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if let Some(rss_pages) = parts.get(1) {
            if let Ok(pages) = rss_pages.parse::<usize>() {
                return pages * 4096; // Assuming 4KB page size
            }
        }
    }

    // Fallback: return 0 (test will log but not fail)
    0
}

#[tokio::test]
#[ignore] // Long-running test
async fn test_memory_stability_event_emission() {
    let broadcaster = Arc::new(EventBroadcaster::new(2000));

    // Create a subscriber to ensure events are being delivered
    let mut rx = broadcaster.subscribe();
    let subscriber_task = tokio::spawn(async move {
        while rx.recv().await.is_ok() {
            // Just drain events
        }
    });

    let memory_before = get_memory_usage();
    println!("Memory before: {} bytes", memory_before);

    // Emit 10,000 events
    for i in 0..10_000 {
        broadcaster.emit(create_test_event(i));

        // Small delay to allow processing
        if i % 1000 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    // Allow time for cleanup
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let memory_after = get_memory_usage();
    println!("Memory after: {} bytes", memory_after);

    subscriber_task.abort();

    if memory_before > 0 && memory_after > 0 {
        let growth = memory_after.saturating_sub(memory_before);
        let growth_mb = growth as f64 / 1_048_576.0;

        println!("Memory growth: {:.2} MB", growth_mb);

        // Assert growth < 10MB
        assert!(
            growth < 10_485_760, // 10MB in bytes
            "Memory growth {:.2} MB exceeds 10MB threshold",
            growth_mb
        );
    } else {
        println!("Warning: Could not measure memory usage on this platform");
    }
}

#[tokio::test]
#[ignore] // Long-running test
async fn test_memory_stability_session_churn() {
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let persistence = SessionPersistence::new(temp_dir.path().to_path_buf());

    let memory_before = get_memory_usage();
    println!("Memory before: {} bytes", memory_before);

    // Create and destroy 100 sessions
    for session_num in 0..100 {
        let session_id = format!("session-{}", session_num);
        let events: Vec<CoordinationEvent> = (0..10).map(create_test_event).collect();

        // Save session
        persistence
            .save_session(&session_id, &events)
            .await
            .expect("Failed to save session");

        // Restore session
        let _restored = persistence
            .restore_session(&session_id)
            .await
            .expect("Failed to restore session");

        // Periodic cleanup hint
        if session_num % 10 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    // Allow time for cleanup
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let memory_after = get_memory_usage();
    println!("Memory after: {} bytes", memory_after);

    if memory_before > 0 && memory_after > 0 {
        let growth = memory_after.saturating_sub(memory_before);
        let baseline_tolerance = memory_before as f64 * 0.10; // 10% tolerance

        println!(
            "Memory growth: {} bytes (baseline tolerance: {} bytes)",
            growth, baseline_tolerance as usize
        );

        // Assert memory returns to within 10% of baseline
        assert!(
            growth as f64 <= baseline_tolerance,
            "Memory did not return to baseline (growth: {} bytes, tolerance: {} bytes)",
            growth,
            baseline_tolerance as usize
        );
    } else {
        println!("Warning: Could not measure memory usage on this platform");
    }
}
