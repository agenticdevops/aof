use std::fmt;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation, requests pass through
    Closed,
    /// Failures exceeded threshold, requests rejected
    Open,
    /// Testing recovery, limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Consecutive failures before opening (default: 5)
    pub failure_threshold: usize,
    /// Consecutive successes in half-open to close (default: 3)
    pub success_threshold: usize,
    /// How long to stay open before half-open (default: 30s)
    pub timeout: Duration,
    /// Identifier for logging/metrics
    pub name: String,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            name: "circuit-breaker".to_string(),
        }
    }
}

/// Circuit breaker for protecting external service calls
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    consecutive_failures: Arc<AtomicUsize>,
    consecutive_successes: Arc<AtomicUsize>,
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            consecutive_failures: Arc::new(AtomicUsize::new(0)),
            consecutive_successes: Arc::new(AtomicUsize::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
        }
    }

    /// Execute a fallible operation through the circuit breaker
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check if circuit should transition from Open to HalfOpen
        self.check_timeout().await;

        // Get current state
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open => {
                // Circuit is open, reject the call
                Err(CircuitBreakerError::Open)
            }
            CircuitState::Closed | CircuitState::HalfOpen => {
                // Execute the operation
                match operation.await {
                    Ok(result) => {
                        self.on_success().await;
                        Ok(result)
                    }
                    Err(err) => {
                        self.on_failure().await;
                        Err(CircuitBreakerError::Inner(err))
                    }
                }
            }
        }
    }

    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Manually reset circuit to Closed
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        let mut last_failure = self.last_failure.write().await;
        *last_failure = None;
    }

    /// Check if timeout has elapsed and transition from Open to HalfOpen
    async fn check_timeout(&self) {
        let state = self.state.read().await.clone();
        if state != CircuitState::Open {
            return;
        }

        let last_failure = self.last_failure.read().await;
        if let Some(last_failure_time) = *last_failure {
            if last_failure_time.elapsed() >= self.config.timeout {
                drop(last_failure);
                let mut state = self.state.write().await;
                *state = CircuitState::HalfOpen;
                self.consecutive_successes.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Closed => {
                // Reset failure counter
                self.consecutive_failures.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Increment success counter
                let successes = self.consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;

                // Check if we should close the circuit
                if successes >= self.config.success_threshold {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Closed;
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // This shouldn't happen, but just in case
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Closed => {
                // Increment failure counter
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

                // Check if we should open the circuit
                if failures >= self.config.failure_threshold {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Open;
                    let mut last_failure = self.last_failure.write().await;
                    *last_failure = Some(Instant::now());
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                drop(state);
                let mut state = self.state.write().await;
                *state = CircuitState::Open;
                let mut last_failure = self.last_failure.write().await;
                *last_failure = Some(Instant::now());
                self.consecutive_failures.store(0, Ordering::Relaxed);
                self.consecutive_successes.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {
                // Already open, update last failure time
                let mut last_failure = self.last_failure.write().await;
                *last_failure = Some(Instant::now());
            }
        }
    }
}

/// Circuit breaker error
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open, call rejected
    Open,
    /// Call failed, inner error
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::Inner(err) => write!(f, "Operation failed: {}", err),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[tokio::test]
    async fn test_circuit_starts_closed() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_five_failures_transition_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Simulate 5 consecutive failures
        for _ in 0..5 {
            let result: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("error") }).await;
            assert!(result.is_err());
        }

        assert_eq!(breaker.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_open_circuit_rejects_calls() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Trigger open state
        let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Next call should be rejected
        let result: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }

    #[tokio::test]
    async fn test_transition_to_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,  // Close immediately on first success in half-open
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Trigger open state
        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next call should trigger transition to HalfOpen, then to Closed after success
        let result: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.state().await, CircuitState::Closed); // Should close after success
    }

    #[tokio::test]
    async fn test_three_successes_in_half_open_transition_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 3,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Trigger open state
        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Wait for timeout to transition to HalfOpen
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 3 successful calls should close the circuit
        for _ in 0..3 {
            let result: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;
            assert!(result.is_ok());
        }

        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_single_failure_in_half_open_transitions_back_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Trigger open state
        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Wait for timeout to transition to HalfOpen
        tokio::time::sleep(Duration::from_millis(150)).await;

        // A failure in HalfOpen should open the circuit again
        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_reset_forces_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Trigger open state
        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Reset should force Closed state
        breaker.reset().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);

        // Should accept calls now
        let result: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_successful_calls_reset_failure_count_in_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // 4 failures (not enough to open)
        for _ in 0..4 {
            let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("error") }).await;
        }

        // A success should reset the counter
        let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Ok(()) }).await;
        assert_eq!(breaker.state().await, CircuitState::Closed);

        // Now we need 5 more failures to open
        for _ in 0..4 {
            let _: Result<(), CircuitBreakerError<&str>> = breaker.call(async { Err("error") }).await;
        }
        assert_eq!(breaker.state().await, CircuitState::Closed); // Still closed

        let _: Result<(), _> = breaker.call(async { Err("error") }).await;
        assert_eq!(breaker.state().await, CircuitState::Open); // Now open
    }
}
