// Chaos tests for agent crash and recovery scenarios

use aof_runtime::{
    resilience::{
        bulkhead::{Bulkhead, BulkheadConfig},
        circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
        retry::RetryPolicy,
        supervisor::AgentSupervisor,
    },
    AofError, AofMetrics,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

#[tokio::test]
async fn test_agent_crash_recovery() {
    let retry_policy = RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        jitter: false,
        ..Default::default()
    };
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    let bulkhead = Arc::new(Bulkhead::new(BulkheadConfig::default()));
    let metrics = Arc::new(AofMetrics::new().unwrap());

    let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics.clone());

    let attempt_count = Arc::new(AtomicUsize::new(0));
    let result = supervisor
        .supervise("crash-agent", || {
            let counter = attempt_count.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                if count == 0 {
                    // First attempt crashes
                    Err(AofError::runtime("Agent crashed"))
                } else {
                    // Second attempt succeeds
                    Ok(())
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(attempt_count.load(Ordering::Relaxed), 2); // Crash + recovery
    assert!(metrics.agent_restarts_total.get() > 0.0);
}

#[tokio::test]
async fn test_mass_agent_crash() {
    let retry_policy = RetryPolicy {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        jitter: false,
        ..Default::default()
    };
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    let bulkhead = Arc::new(Bulkhead::new(BulkheadConfig {
        max_concurrent_agents: 20,
        ..Default::default()
    }));
    let metrics = Arc::new(AofMetrics::new().unwrap());

    let supervisor = Arc::new(AgentSupervisor::new(
        retry_policy,
        circuit_breaker,
        bulkhead,
        metrics.clone(),
    ));

    // Spawn 10 agents that all crash once then succeed
    let mut handles = vec![];
    for i in 0..10 {
        let sup = supervisor.clone();
        let handle = tokio::spawn(async move {
            let crashed = Arc::new(AtomicBool::new(false));
            sup.supervise(&format!("agent-{}", i), || {
                let c = crashed.clone();
                async move {
                    if !c.swap(true, Ordering::Relaxed) {
                        Err(AofError::runtime("Crash!"))
                    } else {
                        Ok(())
                    }
                }
            })
            .await
        });
        handles.push(handle);
    }

    // All agents should recover
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // System remained stable
    assert!(metrics.agent_restarts_total.get() >= 10.0);
}

#[tokio::test]
async fn test_agent_crash_loop_circuit_break() {
    let retry_policy = RetryPolicy {
        max_attempts: 10,
        base_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let circuit_config = CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    };
    let circuit_breaker = CircuitBreaker::new(circuit_config);
    let bulkhead = Arc::new(Bulkhead::new(BulkheadConfig::default()));
    let metrics = Arc::new(AofMetrics::new().unwrap());

    let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics.clone());

    // Agent that always crashes
    let result = supervisor
        .supervise("crash-loop-agent", || async {
            Err(AofError::runtime("Always crashes"))
        })
        .await;

    assert!(result.is_err());

    // Should have stopped before max_attempts due to circuit breaker
    let attempts = metrics.agent_restarts_total.get();
    assert!(attempts <= 6.0); // Circuit breaker should trip around 5 failures
}
