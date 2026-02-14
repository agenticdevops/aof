use aof_coordination::EventBroadcaster;
use aof_core::{ActivityEvent, ActivityType, CoordinationEvent};
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

fn create_agent_event(agent_id: &str, iteration: usize) -> CoordinationEvent {
    CoordinationEvent {
        activity: ActivityEvent {
            activity_type: ActivityType::Thinking,
            message: format!("Agent {} iteration {}", agent_id, iteration),
            timestamp: Utc::now(),
            details: None,
        },
        agent_id: agent_id.to_string(),
        session_id: "session-123".to_string(),
        event_id: format!("evt-{}-{}", agent_id, iteration),
        timestamp: Utc::now(),
        introduction: None,
        coordination_activity: None,
    }
}

#[tokio::test]
async fn test_20_concurrent_agents() {
    let broadcaster = Arc::new(EventBroadcaster::new(5000));

    // Create 5 subscribers (simulating WebSocket clients)
    let mut subscribers = Vec::new();
    for _ in 0..5 {
        subscribers.push(broadcaster.subscribe());
    }

    // Track latencies
    let latencies = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn subscriber tasks
    let mut subscriber_tasks = JoinSet::new();
    for (idx, mut rx) in subscribers.into_iter().enumerate() {
        let latencies_clone = Arc::clone(&latencies);
        subscriber_tasks.spawn(async move {
            let mut received = 0;
            while let Ok(event) = tokio::time::timeout(Duration::from_secs(15), rx.recv()).await {
                if event.is_err() {
                    continue;
                }

                // Measure latency from event timestamp to receive time
                let now = Utc::now();
                let event_time = event.unwrap().timestamp;
                let latency = (now - event_time).num_milliseconds().max(0) as u64;

                latencies_clone.lock().await.push(Duration::from_millis(latency));
                received += 1;

                // Each subscriber should receive 200 events (20 agents * 10 events each)
                if received >= 200 {
                    break;
                }
            }
            println!("Subscriber {} received {} events", idx, received);
            received
        });
    }

    // Spawn 20 agent tasks, each emitting 10 events
    let start = Instant::now();
    let mut agent_tasks = JoinSet::new();

    for agent_num in 0..20 {
        let broadcaster_clone = Arc::clone(&broadcaster);
        agent_tasks.spawn(async move {
            let agent_id = format!("agent-{}", agent_num);

            for iteration in 0..10 {
                broadcaster_clone.emit(create_agent_event(&agent_id, iteration));
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }

    // Wait for all agents to complete
    while agent_tasks.join_next().await.is_some() {}

    let total_time = start.elapsed();
    println!("Total agent execution time: {:?}", total_time);

    // Wait for all subscribers to receive events
    let mut subscriber_counts = Vec::new();
    while let Some(result) = subscriber_tasks.join_next().await {
        if let Ok(count) = result {
            subscriber_counts.push(count);
        }
    }

    // Verify all subscribers received all events
    for (idx, count) in subscriber_counts.iter().enumerate() {
        assert_eq!(
            *count, 200,
            "Subscriber {} received {} events instead of 200",
            idx, count
        );
    }

    // Calculate p95 latency
    let mut latencies_vec = latencies.lock().await;
    latencies_vec.sort();
    let p95_index = (latencies_vec.len() as f64 * 0.95) as usize;
    let p95_latency = latencies_vec.get(p95_index).copied().unwrap_or(Duration::ZERO);

    println!("P95 fanout latency: {:?}", p95_latency);

    // Assert total time < 10 seconds
    assert!(
        total_time < Duration::from_secs(10),
        "Total execution time {:?} exceeds 10s threshold",
        total_time
    );

    // Assert p95 latency < 100ms
    assert!(
        p95_latency < Duration::from_millis(100),
        "P95 latency {:?} exceeds 100ms threshold",
        p95_latency
    );
}

#[tokio::test]
async fn test_agent_throughput_scaling() {
    // Test throughput at different agent counts: 1, 5, 10, 20
    let agent_counts = vec![1, 5, 10, 20];
    let mut throughputs = Vec::new();

    for agent_count in agent_counts {
        let broadcaster = Arc::new(EventBroadcaster::new(2000));
        let events_per_agent = 50;

        let start = Instant::now();
        let mut tasks = JoinSet::new();

        for agent_num in 0..agent_count {
            let broadcaster_clone = Arc::clone(&broadcaster);
            tasks.spawn(async move {
                let agent_id = format!("agent-{}", agent_num);
                for iteration in 0..events_per_agent {
                    broadcaster_clone.emit(create_agent_event(&agent_id, iteration));
                }
            });
        }

        // Wait for completion
        while tasks.join_next().await.is_some() {}

        let elapsed = start.elapsed();
        let total_events = agent_count * events_per_agent;
        let throughput = total_events as f64 / elapsed.as_secs_f64();

        throughputs.push(throughput);
        println!(
            "{} agents: {:.0} events/sec",
            agent_count, throughput
        );
    }

    // Verify throughput scales (doesn't degrade linearly)
    // With 20x agents, throughput should be > 5x the single-agent throughput
    // (sub-linear degradation is acceptable, linear degradation is not)
    let single_agent_throughput = throughputs[0];
    let twenty_agent_throughput = throughputs[3];

    assert!(
        twenty_agent_throughput > single_agent_throughput * 5.0,
        "Throughput degraded too much: single={:.0}, 20-agent={:.0}",
        single_agent_throughput,
        twenty_agent_throughput
    );
}
