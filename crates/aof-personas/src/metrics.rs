//! Reliability metrics computation for agent personas
//!
//! Computes uptime percentage and success rate from CoordinationEvent history.
//! Metrics are derived from event streams (Phase 1 broadcast events) and
//! cached in a concurrent-safe ReliabilityCache for efficient access.
//!
//! # Computation Rules
//!
//! - **Uptime %** = (total events - error events) / total events * 100
//! - **Success rate %** = completed events / total events * 100
//! - If fewer than 10 events, percentages are returned as `None` (insufficient data)
//! - Empty event list → uptime = None, success = None
//!
//! # Example
//!
//! ```rust
//! use aof_personas::metrics::{compute_agent_metrics, ReliabilityMetrics};
//! use aof_core::coordination::CoordinationEvent;
//!
//! let events: Vec<CoordinationEvent> = vec![];
//! let metrics = compute_agent_metrics("k8s-monitor", &events);
//! assert!(metrics.uptime_percent.is_none());
//! assert_eq!(metrics.event_count, 0);
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use aof_core::activity::ActivityType;
use aof_core::coordination::CoordinationEvent;

/// Minimum number of events required before computing percentages.
/// Below this threshold, metrics return `None` to avoid misleading values.
const MIN_EVENTS_FOR_METRICS: usize = 10;

/// Reliability metrics for a single agent
///
/// Computed from CoordinationEvent history. All percentage fields are
/// `Option<f32>` — `None` when insufficient data (< 10 events).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    /// Agent identifier
    pub agent_id: String,

    /// Uptime percentage (0.0 - 100.0), None if < 10 events
    pub uptime_percent: Option<f32>,

    /// Success rate percentage (0.0 - 100.0), None if < 10 events
    pub success_rate: Option<f32>,

    /// Total number of events processed for this agent
    pub event_count: usize,

    /// When these metrics were last computed (ISO 8601)
    pub last_update: DateTime<Utc>,

    /// Timestamp of the most recent error event, if any
    pub last_error: Option<DateTime<Utc>>,
}

impl ReliabilityMetrics {
    /// Create empty metrics for an agent with no events
    pub fn empty(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            uptime_percent: None,
            success_rate: None,
            event_count: 0,
            last_update: Utc::now(),
            last_error: None,
        }
    }
}

/// Compute reliability metrics for a specific agent from event history
///
/// Filters events by `agent_id`, then computes:
/// - uptime_percent: (total - errors) / total * 100
/// - success_rate: completed / total * 100
///
/// Returns `None` for percentage fields if fewer than `MIN_EVENTS_FOR_METRICS` events.
///
/// # Edge cases
/// - No events → uptime=None, success=None, event_count=0
/// - All errors → uptime=0%, success=0%
/// - All completed → uptime=100%, success=100%
/// - Mixed → proportional percentages
pub fn compute_agent_metrics(agent_id: &str, events: &[CoordinationEvent]) -> ReliabilityMetrics {
    let agent_events: Vec<&CoordinationEvent> = events
        .iter()
        .filter(|e| e.agent_id == agent_id)
        .collect();

    let total = agent_events.len();

    if total == 0 {
        return ReliabilityMetrics::empty(agent_id);
    }

    let error_count = agent_events
        .iter()
        .filter(|e| e.activity.activity_type == ActivityType::Error)
        .count();

    let completed_count = agent_events
        .iter()
        .filter(|e| e.activity.activity_type == ActivityType::Completed)
        .count();

    let last_error = agent_events
        .iter()
        .filter(|e| e.activity.activity_type == ActivityType::Error)
        .map(|e| e.timestamp)
        .max();

    let (uptime_percent, success_rate) = if total >= MIN_EVENTS_FOR_METRICS {
        let uptime = ((total - error_count) as f32 / total as f32) * 100.0;
        let success = (completed_count as f32 / total as f32) * 100.0;
        (Some(uptime), Some(success))
    } else {
        (None, None)
    };

    ReliabilityMetrics {
        agent_id: agent_id.to_string(),
        uptime_percent,
        success_rate,
        event_count: total,
        last_update: Utc::now(),
        last_error,
    }
}

/// Compute metrics with a sliding time window
///
/// Only considers events within the last `hours` hours. Returns `None`
/// if no events fall within the window.
pub fn compute_metrics_with_window(
    agent_id: &str,
    events: &[CoordinationEvent],
    hours: u32,
) -> Option<ReliabilityMetrics> {
    let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);

    let windowed_events: Vec<CoordinationEvent> = events
        .iter()
        .filter(|e| e.timestamp >= cutoff)
        .cloned()
        .collect();

    if windowed_events.is_empty() {
        return None;
    }

    Some(compute_agent_metrics(agent_id, &windowed_events))
}

/// Thread-safe cache for reliability metrics
///
/// Stores events and computed metrics per agent. Supports concurrent reads
/// via `RwLock` and limits memory usage via FIFO event eviction.
///
/// # Concurrency
///
/// - Multiple readers can access metrics simultaneously (RwLock)
/// - Writers (event updates) have exclusive access briefly
/// - Version counter increments on every write (for UI cache invalidation)
pub struct ReliabilityCache {
    /// Cached metrics per agent_id
    metrics: Arc<RwLock<HashMap<String, ReliabilityMetrics>>>,

    /// All events stored for recomputation
    events: Arc<RwLock<Vec<CoordinationEvent>>>,

    /// Maximum number of events to retain (FIFO eviction)
    max_events: usize,

    /// Monotonically increasing version counter (increments on write)
    version: AtomicU64,
}

impl ReliabilityCache {
    /// Create a new ReliabilityCache with the specified max event capacity
    pub fn new(max_events: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
            version: AtomicU64::new(0),
        }
    }

    /// Create a cache with default capacity (10000 events)
    pub fn default_capacity() -> Self {
        Self::new(10_000)
    }

    /// Get the current cache version (monotonically increasing)
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }

    /// Update the cache with a new event
    ///
    /// Appends the event, enforces max_events via FIFO eviction,
    /// then recomputes metrics for the affected agent.
    pub async fn update_with_event(&self, event: &CoordinationEvent) -> anyhow::Result<()> {
        let agent_id = event.agent_id.clone();

        // Append event and enforce capacity
        {
            let mut events = self.events.write().await;
            events.push(event.clone());

            // FIFO eviction if over capacity
            if events.len() > self.max_events {
                let excess = events.len() - self.max_events;
                events.drain(0..excess);
            }
        }

        // Recompute metrics for the affected agent
        let events_snapshot = self.events.read().await;
        let new_metrics = compute_agent_metrics(&agent_id, &events_snapshot);
        drop(events_snapshot);

        // Store updated metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.insert(agent_id, new_metrics);
        }

        // Increment version
        self.version.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    /// Get metrics for a specific agent
    ///
    /// Returns cached metrics if available, otherwise computes from
    /// stored events on cache miss.
    pub async fn get_metrics(&self, agent_id: &str) -> Option<ReliabilityMetrics> {
        // Check cache first
        {
            let metrics = self.metrics.read().await;
            if let Some(cached) = metrics.get(agent_id) {
                return Some(cached.clone());
            }
        }

        // Cache miss: compute from events
        let events = self.events.read().await;
        let agent_events_exist = events.iter().any(|e| e.agent_id == agent_id);

        if !agent_events_exist {
            return None;
        }

        let computed = compute_agent_metrics(agent_id, &events);
        drop(events);

        // Store in cache
        {
            let mut metrics = self.metrics.write().await;
            metrics.insert(agent_id.to_string(), computed.clone());
        }

        Some(computed)
    }

    /// Recompute metrics for all known agents
    ///
    /// Iterates over all unique agent_ids in the event store and
    /// recomputes their metrics. Used for full cache refresh.
    pub async fn recompute_all(&self) {
        let events = self.events.read().await;

        // Collect unique agent IDs
        let agent_ids: Vec<String> = events
            .iter()
            .map(|e| e.agent_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Compute metrics for each
        let mut new_metrics = HashMap::new();
        for agent_id in &agent_ids {
            let m = compute_agent_metrics(agent_id, &events);
            new_metrics.insert(agent_id.clone(), m);
        }

        drop(events);

        // Replace all cached metrics
        {
            let mut metrics = self.metrics.write().await;
            *metrics = new_metrics;
        }

        self.version.fetch_add(1, Ordering::SeqCst);
    }

    /// Get the total number of events in the cache
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }

    /// Get all agent IDs that have metrics
    pub async fn agent_ids(&self) -> Vec<String> {
        self.metrics.read().await.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::activity::ActivityEvent;

    /// Helper: create a CoordinationEvent with a specific activity type
    fn make_event(agent_id: &str, activity_type: ActivityType, message: &str) -> CoordinationEvent {
        let activity = ActivityEvent::new(activity_type, message);
        CoordinationEvent::from_activity(activity, agent_id, "test-session")
    }

    #[test]
    fn test_compute_metrics_empty_events() {
        let events: Vec<CoordinationEvent> = vec![];
        let metrics = compute_agent_metrics("k8s-monitor", &events);

        assert_eq!(metrics.agent_id, "k8s-monitor");
        assert!(metrics.uptime_percent.is_none());
        assert!(metrics.success_rate.is_none());
        assert_eq!(metrics.event_count, 0);
        assert!(metrics.last_error.is_none());
    }

    #[test]
    fn test_compute_metrics_all_success() {
        let events: Vec<CoordinationEvent> = (0..10)
            .map(|i| make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)))
            .collect();

        let metrics = compute_agent_metrics("agent-1", &events);

        assert_eq!(metrics.event_count, 10);
        assert!((metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
        assert!((metrics.success_rate.unwrap() - 100.0).abs() < 0.1);
        assert!(metrics.last_error.is_none());
    }

    #[test]
    fn test_compute_metrics_all_errors() {
        let events: Vec<CoordinationEvent> = (0..10)
            .map(|i| make_event("agent-1", ActivityType::Error, &format!("error-{}", i)))
            .collect();

        let metrics = compute_agent_metrics("agent-1", &events);

        assert_eq!(metrics.event_count, 10);
        assert!((metrics.uptime_percent.unwrap() - 0.0).abs() < 0.1);
        assert!((metrics.success_rate.unwrap() - 0.0).abs() < 0.1);
        assert!(metrics.last_error.is_some());
    }

    #[test]
    fn test_compute_metrics_mixed() {
        let mut events = Vec::new();
        // 9 success, 1 error = 90% uptime, 90% success
        for i in 0..9 {
            events.push(make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)));
        }
        events.push(make_event("agent-1", ActivityType::Error, "error-1"));

        let metrics = compute_agent_metrics("agent-1", &events);

        assert_eq!(metrics.event_count, 10);
        assert!((metrics.uptime_percent.unwrap() - 90.0).abs() < 0.1);
        assert!((metrics.success_rate.unwrap() - 90.0).abs() < 0.1);
        assert!(metrics.last_error.is_some());
    }

    #[test]
    fn test_compute_metrics_insufficient_data() {
        // Only 9 events — below MIN_EVENTS_FOR_METRICS (10)
        let events: Vec<CoordinationEvent> = (0..9)
            .map(|i| make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)))
            .collect();

        let metrics = compute_agent_metrics("agent-1", &events);

        assert_eq!(metrics.event_count, 9);
        assert!(metrics.uptime_percent.is_none());
        assert!(metrics.success_rate.is_none());
    }

    #[test]
    fn test_compute_metrics_filters_by_agent_id() {
        let mut events = Vec::new();
        for i in 0..10 {
            events.push(make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)));
        }
        for i in 0..5 {
            events.push(make_event("agent-2", ActivityType::Error, &format!("err-{}", i)));
        }

        let m1 = compute_agent_metrics("agent-1", &events);
        let m2 = compute_agent_metrics("agent-2", &events);

        assert_eq!(m1.event_count, 10);
        assert!((m1.uptime_percent.unwrap() - 100.0).abs() < 0.1);

        assert_eq!(m2.event_count, 5);
        assert!(m2.uptime_percent.is_none()); // <10 events
    }

    #[test]
    fn test_compute_metrics_last_error_timestamp() {
        let mut events = Vec::new();
        for i in 0..8 {
            events.push(make_event("agent-1", ActivityType::Completed, &format!("task-{}", i)));
        }
        let error_event = make_event("agent-1", ActivityType::Error, "error-1");
        let error_ts = error_event.timestamp;
        events.push(error_event);
        events.push(make_event("agent-1", ActivityType::Completed, "task-final"));

        let metrics = compute_agent_metrics("agent-1", &events);

        assert_eq!(metrics.event_count, 10);
        assert!(metrics.last_error.is_some());
        assert_eq!(metrics.last_error.unwrap(), error_ts);
    }

    #[test]
    fn test_reliability_metrics_serialization() {
        let metrics = ReliabilityMetrics {
            agent_id: "test-agent".to_string(),
            uptime_percent: Some(95.5),
            success_rate: Some(92.0),
            event_count: 20,
            last_update: Utc::now(),
            last_error: None,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"uptime_percent\":95.5"));
        assert!(json.contains("\"success_rate\":92.0"));
        assert!(json.contains("\"event_count\":20"));

        let deserialized: ReliabilityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_id, "test-agent");
        assert!((deserialized.uptime_percent.unwrap() - 95.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_cache_update_and_get() {
        let cache = ReliabilityCache::new(100);

        // Add 10 events for agent-1
        for i in 0..10 {
            let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
            cache.update_with_event(&event).await.unwrap();
        }

        let metrics = cache.get_metrics("agent-1").await.unwrap();
        assert_eq!(metrics.event_count, 10);
        assert!((metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_cache_version_increments() {
        let cache = ReliabilityCache::new(100);
        assert_eq!(cache.version(), 0);

        let event = make_event("agent-1", ActivityType::Completed, "task-1");
        cache.update_with_event(&event).await.unwrap();
        assert_eq!(cache.version(), 1);

        let event2 = make_event("agent-1", ActivityType::Error, "err-1");
        cache.update_with_event(&event2).await.unwrap();
        assert_eq!(cache.version(), 2);
    }

    #[tokio::test]
    async fn test_cache_fifo_eviction() {
        let cache = ReliabilityCache::new(5); // Only keep 5 events

        for i in 0..10 {
            let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
            cache.update_with_event(&event).await.unwrap();
        }

        assert_eq!(cache.event_count().await, 5);
    }

    #[tokio::test]
    async fn test_cache_missing_agent() {
        let cache = ReliabilityCache::new(100);

        let result = cache.get_metrics("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_recompute_all() {
        let cache = ReliabilityCache::new(100);

        // Add events for two agents
        for i in 0..10 {
            let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
            cache.update_with_event(&event).await.unwrap();
        }
        for i in 0..10 {
            let event = make_event("agent-2", ActivityType::Error, &format!("err-{}", i));
            cache.update_with_event(&event).await.unwrap();
        }

        cache.recompute_all().await;

        let m1 = cache.get_metrics("agent-1").await.unwrap();
        let m2 = cache.get_metrics("agent-2").await.unwrap();

        assert!((m1.uptime_percent.unwrap() - 100.0).abs() < 0.1);
        assert!((m2.uptime_percent.unwrap() - 0.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_cache_concurrent_reads() {
        let cache = Arc::new(ReliabilityCache::new(100));

        // Populate cache
        for i in 0..10 {
            let event = make_event("agent-1", ActivityType::Completed, &format!("task-{}", i));
            cache.update_with_event(&event).await.unwrap();
        }

        // Spawn 10 concurrent readers
        let mut handles = Vec::new();
        for _ in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                cache_clone.get_metrics("agent-1").await
            });
            handles.push(handle);
        }

        // All should succeed without blocking
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap().event_count, 10);
        }
    }
}
