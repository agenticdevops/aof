use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{AofError, AofResult};

/// Bulkhead configuration for resource isolation
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent agents (default: 20)
    pub max_concurrent_agents: usize,
    /// Maximum event queue size (default: 1000)
    pub max_event_queue_size: usize,
    /// Maximum WebSocket clients (default: 100)
    pub max_websocket_clients: usize,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 20,
            max_event_queue_size: 1000,
            max_websocket_clients: 100,
        }
    }
}

/// Bulkhead pattern for resource isolation
pub struct Bulkhead {
    config: BulkheadConfig,
    agent_semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicUsize>,
}

impl Bulkhead {
    /// Create a new bulkhead with the given configuration
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            agent_semaphore: Arc::new(Semaphore::new(config.max_concurrent_agents)),
            active_count: Arc::new(AtomicUsize::new(0)),
            config,
        }
    }

    /// Acquire a slot for agent execution. Returns error if at capacity.
    pub async fn acquire_agent_slot(&self) -> AofResult<BulkheadPermit> {
        let permit = self
            .agent_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| AofError::runtime(format!("Failed to acquire bulkhead permit: {}", e)))?;

        self.active_count.fetch_add(1, Ordering::Relaxed);

        Ok(BulkheadPermit {
            _permit: permit,
            active_count: self.active_count.clone(),
        })
    }

    /// Try to acquire without waiting. Returns None if at capacity.
    pub fn try_acquire_agent_slot(&self) -> Option<BulkheadPermit> {
        let permit = self.agent_semaphore.clone().try_acquire_owned().ok()?;

        self.active_count.fetch_add(1, Ordering::Relaxed);

        Some(BulkheadPermit {
            _permit: permit,
            active_count: self.active_count.clone(),
        })
    }

    /// Current utilization (0.0 - 1.0)
    pub fn utilization(&self) -> f64 {
        let active = self.active_count.load(Ordering::Relaxed);
        active as f64 / self.config.max_concurrent_agents as f64
    }

    /// Number of active agents
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }
}

/// Permit for bulkhead slot
pub struct BulkheadPermit {
    _permit: OwnedSemaphorePermit,
    active_count: Arc<AtomicUsize>,
}

impl Drop for BulkheadPermit {
    fn drop(&mut self) {
        self.active_count.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bulkhead_permits_acquired_up_to_max() {
        let config = BulkheadConfig {
            max_concurrent_agents: 3,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);

        // Acquire 3 permits
        let _permit1 = bulkhead.acquire_agent_slot().await.unwrap();
        let _permit2 = bulkhead.acquire_agent_slot().await.unwrap();
        let _permit3 = bulkhead.acquire_agent_slot().await.unwrap();

        assert_eq!(bulkhead.active_count(), 3);

        // Try to acquire one more should fail
        let permit4 = bulkhead.try_acquire_agent_slot();
        assert!(permit4.is_none());
    }

    #[tokio::test]
    async fn test_bulkhead_dropped_permit_frees_slot() {
        let config = BulkheadConfig {
            max_concurrent_agents: 2,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);

        // Acquire 2 permits
        let permit1 = bulkhead.acquire_agent_slot().await.unwrap();
        let _permit2 = bulkhead.acquire_agent_slot().await.unwrap();

        assert_eq!(bulkhead.active_count(), 2);

        // Drop first permit
        drop(permit1);
        assert_eq!(bulkhead.active_count(), 1);

        // Now we should be able to acquire another
        assert!(bulkhead.try_acquire_agent_slot().is_some());
    }

    #[tokio::test]
    async fn test_bulkhead_utilization_reports_correctly() {
        let config = BulkheadConfig {
            max_concurrent_agents: 10,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);

        assert_eq!(bulkhead.utilization(), 0.0);

        let _permit1 = bulkhead.acquire_agent_slot().await.unwrap();
        assert_eq!(bulkhead.utilization(), 0.1);

        let _permit2 = bulkhead.acquire_agent_slot().await.unwrap();
        assert_eq!(bulkhead.utilization(), 0.2);

        let _permit3 = bulkhead.acquire_agent_slot().await.unwrap();
        let _permit4 = bulkhead.acquire_agent_slot().await.unwrap();
        let _permit5 = bulkhead.acquire_agent_slot().await.unwrap();
        assert_eq!(bulkhead.utilization(), 0.5);
    }

    #[tokio::test]
    async fn test_bulkhead_try_acquire_fails_at_capacity() {
        let config = BulkheadConfig {
            max_concurrent_agents: 1,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);

        let _permit1 = bulkhead.try_acquire_agent_slot();
        assert!(bulkhead.try_acquire_agent_slot().is_none());
    }
}
