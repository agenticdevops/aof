// Chaos tests for resource exhaustion scenarios

use aof_runtime::{
    resilience::{
        bulkhead::{Bulkhead, BulkheadConfig},
        degradation::{DegradationEngine, DegradationThresholds, SystemHealth},
    },
    AofMetrics,
};
use std::sync::Arc;

#[tokio::test]
async fn test_bulkhead_at_capacity() {
    let config = BulkheadConfig {
        max_concurrent_agents: 5,
        ..Default::default()
    };
    let bulkhead = Bulkhead::new(config);

    // Acquire all permits
    let permits: Vec<_> = (0..5)
        .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
        .collect();

    assert_eq!(bulkhead.active_count(), 5);
    assert_eq!(bulkhead.utilization(), 1.0);

    // Attempt to spawn one more - should be rejected
    assert!(bulkhead.try_acquire_agent_slot().is_none());

    // Existing agents should be unaffected
    assert_eq!(permits.len(), 5);
}

#[tokio::test]
async fn test_memory_pressure_degradation() {
    // This test simulates memory pressure detection
    // In real impl, would use actual memory metrics
    let thresholds = DegradationThresholds {
        memory_warning: 0.70,
        agent_warning: 0.80,
        ..Default::default()
    };

    let engine = DegradationEngine::new(thresholds);
    let bulkhead = Bulkhead::new(BulkheadConfig {
        max_concurrent_agents: 10,
        ..Default::default()
    });

    // Fill bulkhead to 90% capacity
    let _permits: Vec<_> = (0..9)
        .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
        .collect();

    let metrics = AofMetrics::new().unwrap();

    // Assess health - should detect high agent capacity
    engine.assess_health(&bulkhead, &metrics).await;
    let health = engine.health().await;

    match health {
        SystemHealth::Degraded(components) => {
            assert!(!components.is_empty());
        }
        _ => panic!("Expected degraded state"),
    }
}

#[tokio::test]
async fn test_degradation_recovery() {
    let thresholds = DegradationThresholds {
        agent_warning: 0.80,
        ..Default::default()
    };

    let engine = DegradationEngine::new(thresholds);
    let bulkhead = Bulkhead::new(BulkheadConfig {
        max_concurrent_agents: 10,
        ..Default::default()
    });

    let metrics = AofMetrics::new().unwrap();

    // Initial state should be healthy
    engine.assess_health(&bulkhead, &metrics).await;
    assert_eq!(engine.health().await, SystemHealth::Healthy);

    // Fill bulkhead to trigger degradation
    let permits: Vec<_> = (0..9)
        .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
        .collect();

    engine.assess_health(&bulkhead, &metrics).await;
    assert!(matches!(engine.health().await, SystemHealth::Degraded(_)));

    // Drop permits to recover
    drop(permits);

    engine.assess_health(&bulkhead, &metrics).await;
    assert_eq!(engine.health().await, SystemHealth::Healthy);
}

#[tokio::test]
async fn test_backpressure_handling() {
    let config = BulkheadConfig {
        max_concurrent_agents: 3,
        ..Default::default()
    };
    let bulkhead = Arc::new(Bulkhead::new(config));

    // Simulate backpressure scenario
    let _permits: Vec<_> = (0..3)
        .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
        .collect();

    // Additional requests should be rejected with clear error
    let result = bulkhead.try_acquire_agent_slot();
    assert!(result.is_none(), "Should reject at capacity");

    // System should remain stable (no panic, no deadlock)
    assert_eq!(bulkhead.active_count(), 3);
    assert_eq!(bulkhead.utilization(), 1.0);
}
