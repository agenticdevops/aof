// Chaos tests for network failure and circuit breaker scenarios

use aof_runtime::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState};
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_on_service_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_millis(100),
        name: "external-service".to_string(),
    };

    let breaker = CircuitBreaker::new(config);

    // Simulate 5 consecutive failures to external service
    for i in 0..5 {
        let result: Result<(), CircuitBreakerError<&str>> = breaker
            .call(async { Err("Service unavailable") })
            .await;
        assert!(result.is_err());
        println!("Failure {}: {:?}", i + 1, breaker.state().await);
    }

    // Circuit should now be open
    assert_eq!(breaker.state().await, CircuitState::Open);

    // Next request should be rejected immediately (no network call)
    let result: Result<(), CircuitBreakerError<&str>> = breaker
        .call(async { Ok(()) })
        .await;

    assert!(matches!(result, Err(CircuitBreakerError::Open)));

    // Wait for timeout to allow half-open state
    tokio::time::sleep(Duration::from_millis(150)).await;

    // First request after timeout should test recovery (half-open state)
    let result: Result<(), CircuitBreakerError<&str>> = breaker
        .call(async { Ok(()) })
        .await;

    assert!(result.is_ok());

    // If it succeeded but we need 3 successes, state should still be half-open or closed
    // depending on success_threshold
}

#[tokio::test]
async fn test_circuit_breaker_prevents_cascading_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(30),
        ..Default::default()
    };

    let breaker = CircuitBreaker::new(config);

    // Trip the circuit with 3 failures
    for _ in 0..3 {
        let _: Result<(), CircuitBreakerError<&str>> = breaker
            .call(async { Err("Downstream service down") })
            .await;
    }

    assert_eq!(breaker.state().await, CircuitState::Open);

    // Now simulate 100 rapid requests - all should be rejected immediately
    let start = std::time::Instant::now();

    for _ in 0..100 {
        let result: Result<(), CircuitBreakerError<&str>> = breaker
            .call(async { Err("Should not execute") })
            .await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }

    let elapsed = start.elapsed();

    // All 100 requests should complete very quickly (< 50ms)
    // because they're rejected immediately without executing
    assert!(elapsed < Duration::from_millis(50));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_state_testing() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("fail") }).await;
    }

    assert_eq!(breaker.state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // First success in half-open
    let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;

    // Second success should close the circuit (success_threshold = 2)
    let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;

    assert_eq!(breaker.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure_reopens() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("fail") }).await;
    assert_eq!(breaker.state().await, CircuitState::Open);

    // Wait for timeout to reach half-open
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Failure in half-open should re-open the circuit
    let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("still failing") }).await;
    assert_eq!(breaker.state().await, CircuitState::Open);
}
