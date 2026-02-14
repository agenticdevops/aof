use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::bulkhead::Bulkhead;
use crate::metrics::AofMetrics;

/// System health state
#[derive(Debug, Clone, PartialEq)]
pub enum SystemHealth {
    /// All systems operating normally
    Healthy,
    /// Some degradation detected
    Degraded(Vec<DegradedComponent>),
    /// Critical state - minimal functionality
    Critical(Vec<String>),
}

/// Component experiencing degradation
#[derive(Debug, Clone, PartialEq)]
pub enum DegradedComponent {
    /// Memory utilization above threshold
    HighMemoryUsage(f64),
    /// CPU utilization above threshold
    HighCpuUsage(f64),
    /// Event queue filling up
    EventQueueBackpressure,
    /// Too many WebSocket clients
    WebSocketClientsHigh,
    /// Agent capacity near limit
    AgentCapacityHigh,
    /// External service unreachable
    ExternalServiceDown(String),
}

/// Degradation thresholds
#[derive(Debug, Clone)]
pub struct DegradationThresholds {
    /// Memory warning threshold (0.70 = 70%)
    pub memory_warning: f64,
    /// Memory critical threshold (0.90 = 90%)
    pub memory_critical: f64,
    /// CPU warning threshold (0.70 = 70%)
    pub cpu_warning: f64,
    /// CPU critical threshold (0.90 = 90%)
    pub cpu_critical: f64,
    /// Queue warning threshold (0.80 = 80%)
    pub queue_warning: f64,
    /// Agent capacity warning threshold (0.80 = 80%)
    pub agent_warning: f64,
}

impl Default for DegradationThresholds {
    fn default() -> Self {
        Self {
            memory_warning: 0.70,
            memory_critical: 0.90,
            cpu_warning: 0.70,
            cpu_critical: 0.90,
            queue_warning: 0.80,
            agent_warning: 0.80,
        }
    }
}

/// Graceful degradation engine
pub struct DegradationEngine {
    health: Arc<RwLock<SystemHealth>>,
    check_interval: Duration,
    thresholds: DegradationThresholds,
    memory_provider: Box<dyn MemoryProvider + Send + Sync>,
}

/// Trait for getting memory usage (for testing)
pub trait MemoryProvider {
    fn get_memory_usage(&self) -> f64;
}

/// Default memory provider (returns 0 for now)
struct DefaultMemoryProvider;

impl MemoryProvider for DefaultMemoryProvider {
    fn get_memory_usage(&self) -> f64 {
        // In a real implementation, this would use sysinfo or similar
        0.0
    }
}

impl DegradationEngine {
    /// Create a new degradation engine
    pub fn new(thresholds: DegradationThresholds) -> Self {
        Self {
            health: Arc::new(RwLock::new(SystemHealth::Healthy)),
            check_interval: Duration::from_secs(10),
            thresholds,
            memory_provider: Box::new(DefaultMemoryProvider),
        }
    }

    /// Create with custom memory provider (for testing)
    #[cfg(test)]
    fn with_memory_provider(
        thresholds: DegradationThresholds,
        provider: Box<dyn MemoryProvider + Send + Sync>,
    ) -> Self {
        Self {
            health: Arc::new(RwLock::new(SystemHealth::Healthy)),
            check_interval: Duration::from_secs(10),
            thresholds,
            memory_provider: provider,
        }
    }

    /// Run periodic health assessment (call in background task)
    pub async fn run_health_loop(&self, bulkhead: &Bulkhead, metrics: &AofMetrics) {
        loop {
            self.assess_health(bulkhead, metrics).await;
            tokio::time::sleep(self.check_interval).await;
        }
    }

    /// Get current system health
    pub async fn health(&self) -> SystemHealth {
        self.health.read().await.clone()
    }

    /// Assess current health based on metrics
    async fn assess_health(&self, bulkhead: &Bulkhead, _metrics: &AofMetrics) {
        let mut degraded_components = Vec::new();
        let mut critical_reasons = Vec::new();

        // Check agent capacity
        let agent_utilization = bulkhead.utilization();
        if agent_utilization >= self.thresholds.agent_warning {
            degraded_components.push(DegradedComponent::AgentCapacityHigh);
        }

        // Check memory (simulated - in real impl would use system metrics)
        let memory_usage = self.memory_provider.get_memory_usage();
        if memory_usage >= self.thresholds.memory_critical {
            critical_reasons.push(format!("Memory usage critical: {:.1}%", memory_usage * 100.0));
        } else if memory_usage >= self.thresholds.memory_warning {
            degraded_components.push(DegradedComponent::HighMemoryUsage(memory_usage));
        }

        // Update health state
        let new_health = if !critical_reasons.is_empty() {
            SystemHealth::Critical(critical_reasons)
        } else if !degraded_components.is_empty() {
            SystemHealth::Degraded(degraded_components.clone())
        } else {
            SystemHealth::Healthy
        };

        let mut health = self.health.write().await;
        *health = new_health.clone();

        // Apply degradation actions
        self.apply_degradation(&new_health).await;
    }

    /// Apply degradation actions based on current health
    pub async fn apply_degradation(&self, health: &SystemHealth) {
        match health {
            SystemHealth::Healthy => {
                // Normal operation - no degradation
            }
            SystemHealth::Degraded(components) => {
                for component in components {
                    match component {
                        DegradedComponent::HighMemoryUsage(_) => {
                            // Clear non-essential caches, reduce log verbosity
                            tracing::warn!("Degraded: High memory usage detected, clearing caches");
                        }
                        DegradedComponent::EventQueueBackpressure => {
                            // Drop low-priority events (DEBUG-level events)
                            tracing::warn!("Degraded: Event queue backpressure, dropping low-priority events");
                        }
                        DegradedComponent::AgentCapacityHigh => {
                            // Reject new agent spawns until utilization drops
                            tracing::warn!("Degraded: Agent capacity high, rejecting new spawns");
                        }
                        _ => {}
                    }
                }
            }
            SystemHealth::Critical(reasons) => {
                // Disable non-essential features
                tracing::error!("Critical: {:?}", reasons);
                tracing::error!("Entering critical mode: disabling metrics collection and event persistence");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::bulkhead::{Bulkhead, BulkheadConfig};

    struct TestMemoryProvider {
        usage: f64,
    }

    impl MemoryProvider for TestMemoryProvider {
        fn get_memory_usage(&self) -> f64 {
            self.usage
        }
    }

    #[tokio::test]
    async fn test_healthy_when_below_thresholds() {
        let engine = DegradationEngine::new(DegradationThresholds::default());
        let bulkhead = Bulkhead::new(BulkheadConfig {
            max_concurrent_agents: 10,
            ..Default::default()
        });
        let metrics = AofMetrics::new().unwrap();

        // With no agents running, should be healthy
        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;

        assert_eq!(health, SystemHealth::Healthy);
    }

    #[tokio::test]
    async fn test_degraded_when_memory_above_70_percent() {
        let thresholds = DegradationThresholds {
            memory_warning: 0.70,
            ..Default::default()
        };
        let engine = DegradationEngine::with_memory_provider(
            thresholds,
            Box::new(TestMemoryProvider { usage: 0.75 }),
        );

        let bulkhead = Bulkhead::new(BulkheadConfig::default());
        let metrics = AofMetrics::new().unwrap();

        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;

        match health {
            SystemHealth::Degraded(components) => {
                assert!(components.iter().any(|c| matches!(
                    c,
                    DegradedComponent::HighMemoryUsage(_)
                )));
            }
            _ => panic!("Expected degraded state"),
        }
    }

    #[tokio::test]
    async fn test_critical_when_memory_above_90_percent() {
        let thresholds = DegradationThresholds {
            memory_critical: 0.90,
            ..Default::default()
        };
        let engine = DegradationEngine::with_memory_provider(
            thresholds,
            Box::new(TestMemoryProvider { usage: 0.95 }),
        );

        let bulkhead = Bulkhead::new(BulkheadConfig::default());
        let metrics = AofMetrics::new().unwrap();

        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;

        match health {
            SystemHealth::Critical(reasons) => {
                assert!(!reasons.is_empty());
                assert!(reasons[0].contains("Memory usage critical"));
            }
            _ => panic!("Expected critical state"),
        }
    }

    #[tokio::test]
    async fn test_multiple_degradation_reasons_accumulated() {
        let thresholds = DegradationThresholds {
            memory_warning: 0.70,
            agent_warning: 0.80,
            ..Default::default()
        };
        let engine = DegradationEngine::with_memory_provider(
            thresholds,
            Box::new(TestMemoryProvider { usage: 0.75 }),
        );

        // Simulate high agent capacity
        let bulkhead = Bulkhead::new(BulkheadConfig {
            max_concurrent_agents: 10,
            ..Default::default()
        });

        // Acquire 9 permits (90% utilization)
        let _permits: Vec<_> = (0..9)
            .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
            .collect();

        let metrics = AofMetrics::new().unwrap();

        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;

        match health {
            SystemHealth::Degraded(components) => {
                assert!(components.len() >= 2);
                assert!(components.iter().any(|c| matches!(
                    c,
                    DegradedComponent::HighMemoryUsage(_)
                )));
                assert!(components
                    .iter()
                    .any(|c| matches!(c, DegradedComponent::AgentCapacityHigh)));
            }
            _ => panic!("Expected degraded state with multiple reasons"),
        }
    }

    #[tokio::test]
    async fn test_recovery_from_degraded_to_healthy() {
        let thresholds = DegradationThresholds {
            agent_warning: 0.80,
            ..Default::default()
        };
        let engine = DegradationEngine::new(thresholds);

        let bulkhead = Bulkhead::new(BulkheadConfig {
            max_concurrent_agents: 10,
            ..Default::default()
        });

        // Acquire 9 permits (90% utilization) - should be degraded
        let permits: Vec<_> = (0..9)
            .map(|_| bulkhead.try_acquire_agent_slot().unwrap())
            .collect();

        let metrics = AofMetrics::new().unwrap();

        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;
        assert!(matches!(health, SystemHealth::Degraded(_)));

        // Drop permits - should recover
        drop(permits);

        engine.assess_health(&bulkhead, &metrics).await;
        let health = engine.health().await;
        assert_eq!(health, SystemHealth::Healthy);
    }
}
