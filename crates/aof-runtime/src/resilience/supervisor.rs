use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use super::bulkhead::Bulkhead;
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use super::retry::RetryPolicy;
use crate::metrics::AofMetrics;
use crate::{AofError, AofResult};

/// Agent supervision status
#[derive(Debug, Clone)]
pub enum SupervisionStatus {
    /// Agent is running
    Running,
    /// Agent is restarting
    Restarting {
        attempt: usize,
        next_retry: Instant,
    },
    /// Agent failed after max retries
    Failed { reason: String, attempts: usize },
    /// Agent manually stopped
    Stopped,
}

/// Supervised agent metadata
pub struct SupervisedAgent {
    pub agent_id: String,
    pub restart_count: AtomicUsize,
    pub last_crash: RwLock<Option<Instant>>,
    pub status: RwLock<SupervisionStatus>,
}

/// Agent supervisor for crash detection and recovery
pub struct AgentSupervisor {
    retry_policy: RetryPolicy,
    circuit_breaker: CircuitBreaker,
    bulkhead: Arc<Bulkhead>,
    metrics: Arc<AofMetrics>,
    agents: RwLock<HashMap<String, Arc<SupervisedAgent>>>,
}

impl AgentSupervisor {
    /// Create a new agent supervisor
    pub fn new(
        retry_policy: RetryPolicy,
        circuit_breaker: CircuitBreaker,
        bulkhead: Arc<Bulkhead>,
        metrics: Arc<AofMetrics>,
    ) -> Self {
        Self {
            retry_policy,
            circuit_breaker,
            bulkhead,
            metrics,
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Supervise an agent task. Restarts on crash with backoff.
    pub async fn supervise<F, Fut>(&self, agent_id: &str, task_fn: F) -> AofResult<()>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = AofResult<()>> + Send,
    {
        let agent = self.get_or_create_agent(agent_id).await;
        let mut attempt = 0;

        loop {
            // Acquire bulkhead slot
            let _permit = match self.bulkhead.acquire_agent_slot().await {
                Ok(p) => p,
                Err(e) => {
                    let mut status = agent.status.write().await;
                    *status = SupervisionStatus::Failed {
                        reason: format!("Failed to acquire bulkhead slot: {}", e),
                        attempts: attempt,
                    };
                    return Err(e);
                }
            };

            // Update status to Running
            {
                let mut status = agent.status.write().await;
                *status = SupervisionStatus::Running;
            }

            // Run agent task through circuit breaker
            let result = self.circuit_breaker.call(task_fn()).await;

            match result {
                Ok(()) => {
                    // Success - agent completed normally
                    let mut status = agent.status.write().await;
                    *status = SupervisionStatus::Stopped;
                    return Ok(());
                }
                Err(super::circuit_breaker::CircuitBreakerError::Inner(err)) => {
                    // Task failed
                    attempt += 1;
                    agent.restart_count.fetch_add(1, Ordering::Relaxed);
                    let mut last_crash = agent.last_crash.write().await;
                    *last_crash = Some(Instant::now());

                    // Record metrics
                    self.metrics.agent_restarts_total.inc();

                    // Check if we should retry
                    if attempt >= self.retry_policy.max_attempts {
                        self.metrics.agent_failures_total.inc();
                        let mut status = agent.status.write().await;
                        *status = SupervisionStatus::Failed {
                            reason: format!("Max retries exceeded: {}", err),
                            attempts: attempt,
                        };
                        return Err(err);
                    }

                    // Calculate backoff delay
                    let delay = self.retry_policy.delay_for_attempt(attempt);
                    let next_retry = Instant::now() + delay;

                    {
                        let mut status = agent.status.write().await;
                        *status = SupervisionStatus::Restarting {
                            attempt,
                            next_retry,
                        };
                    }

                    // Wait before retrying
                    tokio::time::sleep(delay).await;
                }
                Err(super::circuit_breaker::CircuitBreakerError::Open) => {
                    // Circuit breaker is open - do not retry
                    self.metrics.agent_failures_total.inc();
                    let mut status = agent.status.write().await;
                    *status = SupervisionStatus::Failed {
                        reason: "Circuit breaker open".to_string(),
                        attempts: attempt,
                    };
                    return Err(AofError::runtime("Circuit breaker open"));
                }
            }
        }
    }

    /// Get status of all supervised agents
    pub async fn agent_statuses(&self) -> Vec<(String, SupervisionStatus)> {
        let agents = self.agents.read().await;
        let mut statuses = Vec::new();

        for (id, agent) in agents.iter() {
            let status = agent.status.read().await.clone();
            statuses.push((id.clone(), status));
        }

        statuses
    }

    /// Manually stop an agent (no restart)
    pub async fn stop_agent(&self, agent_id: &str) -> AofResult<()> {
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(agent_id) {
            let mut status = agent.status.write().await;
            *status = SupervisionStatus::Stopped;
            Ok(())
        } else {
            Err(AofError::runtime(format!("Agent not found: {}", agent_id)))
        }
    }

    /// Get or create supervised agent metadata
    async fn get_or_create_agent(&self, agent_id: &str) -> Arc<SupervisedAgent> {
        let mut agents = self.agents.write().await;
        agents
            .entry(agent_id.to_string())
            .or_insert_with(|| {
                Arc::new(SupervisedAgent {
                    agent_id: agent_id.to_string(),
                    restart_count: AtomicUsize::new(0),
                    last_crash: RwLock::new(None),
                    status: RwLock::new(SupervisionStatus::Running),
                })
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    #[tokio::test]
    async fn test_successful_task_completes_without_restart() {
        let retry_policy = RetryPolicy::default();
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics);

        let result = supervisor
            .supervise("test-agent", || async { Ok(()) })
            .await;

        assert!(result.is_ok());

        let statuses = supervisor.agent_statuses().await;
        assert_eq!(statuses.len(), 1);
        assert!(matches!(statuses[0].1, SupervisionStatus::Stopped));
    }

    #[tokio::test]
    async fn test_single_crash_triggers_one_restart() {
        let retry_policy = RetryPolicy {
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics);

        let counter = Arc::new(AtomicUsize::new(0));
        let result = supervisor
            .supervise("test-agent", || {
                let c = counter.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::Relaxed);
                    if count == 0 {
                        Err(AofError::runtime("First attempt fails"))
                    } else {
                        Ok(())
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::Relaxed), 2); // First attempt + 1 restart
    }

    #[tokio::test]
    async fn test_five_crashes_exhausts_retry_budget() {
        let retry_policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(10),
            jitter: false,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics);

        let counter = Arc::new(AtomicUsize::new(0));
        let result = supervisor
            .supervise("test-agent", || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    Err(AofError::runtime("Always fails"))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::Relaxed), 5); // Max attempts

        let statuses = supervisor.agent_statuses().await;
        assert!(matches!(
            statuses[0].1,
            SupervisionStatus::Failed { attempts: 5, .. }
        ));
    }

    #[tokio::test]
    async fn test_exponential_backoff_delays() {
        let retry_policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            jitter: false,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics);

        let start = Instant::now();
        let counter = Arc::new(AtomicUsize::new(0));
        let _result = supervisor
            .supervise("test-agent", || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    Err(AofError::runtime("Always fails"))
                }
            })
            .await;

        let elapsed = start.elapsed();

        // Expected delays: 0ms (first attempt) + 200ms (retry 1) + 400ms (retry 2) = 600ms
        assert!(elapsed >= Duration::from_millis(500)); // Account for some variance
        assert!(elapsed < Duration::from_millis(800)); // But not too much
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let retry_policy = RetryPolicy {
            max_attempts: 10,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(circuit_config);
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor = AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics);

        let counter = Arc::new(AtomicUsize::new(0));
        let result = supervisor
            .supervise("test-agent", || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    Err(AofError::runtime("Always fails"))
                }
            })
            .await;

        assert!(result.is_err());

        // Should stop before max_attempts due to circuit breaker
        let attempts = counter.load(Ordering::Relaxed);
        assert!(attempts <= 5); // Circuit breaker should trip around failure_threshold
    }

    #[tokio::test]
    async fn test_metrics_recorded_for_restarts_and_failures() {
        let retry_policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            jitter: false,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Arc::new(Bulkhead::new(Default::default()));
        let metrics = Arc::new(AofMetrics::new().unwrap());

        let supervisor =
            AgentSupervisor::new(retry_policy, circuit_breaker, bulkhead, metrics.clone());

        let _result = supervisor
            .supervise("test-agent", || async {
                Err(AofError::runtime("Always fails"))
            })
            .await;

        // Should have recorded restarts and final failure
        assert!(metrics.agent_restarts_total.get() > 0.0);
        assert_eq!(metrics.agent_failures_total.get(), 1.0);
    }
}
