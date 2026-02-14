//! Token metrics and auto-degradation for coordination protocols
//!
//! Tracks coordination vs production token usage with atomic counters,
//! calculates overhead percentage, and implements automatic degradation
//! to enforce the 30% coordination overhead budget.
//!
//! # Architecture
//!
//! ```text
//! TokenMetrics (atomic counters)
//!   ├── coordination_input_tokens
//!   ├── coordination_output_tokens
//!   ├── production_input_tokens
//!   ├── production_output_tokens
//!   ├── heartbeat_tokens
//!   └── standup_tokens
//!
//! DegradationManager (state machine)
//!   ├── Evaluate overhead every 60s
//!   ├── Degrade if > 30%: Full -> Standard -> Reduced -> HeartbeatOnly -> Disabled
//!   └── Recover if < 20%: Disabled -> HeartbeatOnly -> Reduced -> Standard -> Full
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aof_coordination_protocols::metrics::{TokenMetrics, DegradationConfig, DegradationManager};
//! use std::time::Duration;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create metrics tracker
//!     let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
//!
//!     // Record tokens
//!     metrics.record_coordination(50, 30, "heartbeat");
//!     metrics.record_production(5000, 3000);
//!
//!     // Check overhead
//!     let overhead = metrics.coordination_overhead();
//!     println!("Coordination overhead: {:.1}%", overhead);
//!
//!     // Create degradation manager
//!     let config = DegradationConfig::default();
//!     let manager = Arc::new(DegradationManager::new(config, metrics));
//!
//!     // Evaluate and auto-degrade
//!     if let Some(new_mode) = manager.evaluate().await {
//!         println!("Degraded to: {:?}", new_mode);
//!     }
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::events::CoordinationMode;

/// Token metrics tracker with atomic counters
///
/// Tracks coordination vs production token usage separately.
/// All counters are atomic for lock-free concurrent access.
pub struct TokenMetrics {
    /// Total coordination input tokens (heartbeat + standup)
    coordination_input_tokens: AtomicU64,
    /// Total coordination output tokens
    coordination_output_tokens: AtomicU64,
    /// Total production input tokens (agent tasks)
    production_input_tokens: AtomicU64,
    /// Total production output tokens
    production_output_tokens: AtomicU64,
    /// Heartbeat protocol tokens (input + output)
    heartbeat_tokens: AtomicU64,
    /// Standup protocol tokens (input + output)
    standup_tokens: AtomicU64,
    /// When metrics were last reset
    last_reset: RwLock<DateTime<Utc>>,
    /// Rolling window duration
    window_duration: Duration,
}

impl TokenMetrics {
    /// Create new token metrics tracker
    ///
    /// # Arguments
    ///
    /// * `window` - Rolling window duration (e.g., 1 hour, 24 hours)
    pub fn new(window: Duration) -> Self {
        Self {
            coordination_input_tokens: AtomicU64::new(0),
            coordination_output_tokens: AtomicU64::new(0),
            production_input_tokens: AtomicU64::new(0),
            production_output_tokens: AtomicU64::new(0),
            heartbeat_tokens: AtomicU64::new(0),
            standup_tokens: AtomicU64::new(0),
            last_reset: RwLock::new(Utc::now()),
            window_duration: window,
        }
    }

    /// Record coordination protocol tokens
    ///
    /// # Arguments
    ///
    /// * `input_tokens` - Number of input tokens
    /// * `output_tokens` - Number of output tokens
    /// * `protocol` - Protocol name ("heartbeat", "standup", etc.)
    pub fn record_coordination(&self, input_tokens: u64, output_tokens: u64, protocol: &str) {
        self.coordination_input_tokens
            .fetch_add(input_tokens, Ordering::Relaxed);
        self.coordination_output_tokens
            .fetch_add(output_tokens, Ordering::Relaxed);

        // Track per-protocol breakdown
        let total = input_tokens + output_tokens;
        match protocol {
            "heartbeat" => {
                self.heartbeat_tokens.fetch_add(total, Ordering::Relaxed);
            }
            "standup" => {
                self.standup_tokens.fetch_add(total, Ordering::Relaxed);
            }
            _ => {
                // Other coordination protocols (future: roundtables, etc.)
            }
        }
    }

    /// Record production work tokens (agent tasks)
    ///
    /// # Arguments
    ///
    /// * `input_tokens` - Number of input tokens
    /// * `output_tokens` - Number of output tokens
    pub fn record_production(&self, input_tokens: u64, output_tokens: u64) {
        self.production_input_tokens
            .fetch_add(input_tokens, Ordering::Relaxed);
        self.production_output_tokens
            .fetch_add(output_tokens, Ordering::Relaxed);
    }

    /// Calculate coordination overhead percentage
    ///
    /// Returns percentage of total tokens spent on coordination (0-100).
    /// Formula: (coordination / (coordination + production)) * 100
    pub fn coordination_overhead(&self) -> f64 {
        let coord = self.total_coordination_tokens() as f64;
        let prod = self.total_production_tokens() as f64;
        let total = coord + prod;

        if total == 0.0 {
            0.0
        } else {
            (coord / total) * 100.0
        }
    }

    /// Total coordination tokens (input + output)
    pub fn total_coordination_tokens(&self) -> u64 {
        self.coordination_input_tokens.load(Ordering::Relaxed)
            + self.coordination_output_tokens.load(Ordering::Relaxed)
    }

    /// Total production tokens (input + output)
    pub fn total_production_tokens(&self) -> u64 {
        self.production_input_tokens.load(Ordering::Relaxed)
            + self.production_output_tokens.load(Ordering::Relaxed)
    }

    /// Heartbeat tokens total
    pub fn heartbeat_tokens(&self) -> u64 {
        self.heartbeat_tokens.load(Ordering::Relaxed)
    }

    /// Standup tokens total
    pub fn standup_tokens(&self) -> u64 {
        self.standup_tokens.load(Ordering::Relaxed)
    }

    /// Reset all counters to zero
    ///
    /// Called at the start of each window period.
    pub async fn reset(&self) {
        self.coordination_input_tokens.store(0, Ordering::Relaxed);
        self.coordination_output_tokens.store(0, Ordering::Relaxed);
        self.production_input_tokens.store(0, Ordering::Relaxed);
        self.production_output_tokens.store(0, Ordering::Relaxed);
        self.heartbeat_tokens.store(0, Ordering::Relaxed);
        self.standup_tokens.store(0, Ordering::Relaxed);
        *self.last_reset.write().await = Utc::now();
    }

    /// Get metrics snapshot for serialization
    ///
    /// Returns a serializable snapshot of current metrics.
    pub async fn snapshot(&self, current_mode: CoordinationMode) -> MetricsSnapshot {
        MetricsSnapshot {
            coordination_tokens: self.total_coordination_tokens(),
            production_tokens: self.total_production_tokens(),
            overhead_percent: self.coordination_overhead(),
            heartbeat_tokens: self.heartbeat_tokens(),
            standup_tokens: self.standup_tokens(),
            window_start: *self.last_reset.read().await,
            current_mode: format!("{:?}", current_mode),
        }
    }
}

/// Serializable metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Total coordination tokens (all protocols)
    pub coordination_tokens: u64,
    /// Total production tokens (agent tasks)
    pub production_tokens: u64,
    /// Coordination overhead percentage (0-100)
    pub overhead_percent: f64,
    /// Heartbeat protocol tokens
    pub heartbeat_tokens: u64,
    /// Standup protocol tokens
    pub standup_tokens: u64,
    /// When current window started
    pub window_start: DateTime<Utc>,
    /// Current coordination mode
    pub current_mode: String,
}

/// Auto-degradation configuration
#[derive(Debug, Clone)]
pub struct DegradationConfig {
    /// Maximum coordination overhead before degradation (default: 30%)
    pub max_overhead_percent: f64,
    /// Enable automatic degradation
    pub auto_degrade: bool,
    /// Check interval for degradation evaluation
    pub check_interval: Duration,
    /// Hysteresis: only recover if overhead drops below this (default: 20%)
    pub recovery_threshold: f64,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            max_overhead_percent: 30.0,
            auto_degrade: true,
            check_interval: Duration::from_secs(60),
            recovery_threshold: 20.0,
        }
    }
}

/// Auto-degradation manager
///
/// Evaluates token metrics and automatically degrades coordination mode
/// when overhead exceeds threshold. Implements hysteresis for recovery.
pub struct DegradationManager {
    config: DegradationConfig,
    metrics: Arc<TokenMetrics>,
    current_mode: Arc<RwLock<CoordinationMode>>,
}

impl DegradationManager {
    /// Create new degradation manager
    ///
    /// # Arguments
    ///
    /// * `config` - Degradation configuration
    /// * `metrics` - Token metrics tracker
    pub fn new(config: DegradationConfig, metrics: Arc<TokenMetrics>) -> Self {
        Self {
            config,
            metrics,
            current_mode: Arc::new(RwLock::new(CoordinationMode::Full)),
        }
    }

    /// Evaluate metrics and decide if degradation needed
    ///
    /// Returns Some(new_mode) if mode should change, None otherwise.
    ///
    /// # Degradation Rules
    ///
    /// - If overhead > max_overhead_percent: degrade one level
    /// - If overhead < recovery_threshold: recover one level
    /// - Between thresholds: no change (hysteresis prevents flapping)
    ///
    /// # Mode Transitions
    ///
    /// Degradation: Full -> Standard -> Reduced -> HeartbeatOnly -> Disabled
    /// Recovery: Disabled -> HeartbeatOnly -> Reduced -> Standard -> Full
    pub async fn evaluate(&self) -> Option<CoordinationMode> {
        if !self.config.auto_degrade {
            return None;
        }

        let overhead = self.metrics.coordination_overhead();
        let current = *self.current_mode.read().await;

        // Degrade if over threshold
        if overhead > self.config.max_overhead_percent {
            let new_mode = match current {
                CoordinationMode::Full => CoordinationMode::Standard,
                CoordinationMode::Standard => CoordinationMode::Reduced,
                CoordinationMode::Reduced => CoordinationMode::HeartbeatOnly,
                CoordinationMode::HeartbeatOnly => CoordinationMode::Disabled,
                CoordinationMode::Disabled => return None, // Already at minimum
            };

            tracing::warn!(
                "Coordination overhead {:.1}% > {:.1}%, degrading from {:?} to {:?}",
                overhead,
                self.config.max_overhead_percent,
                current,
                new_mode
            );

            *self.current_mode.write().await = new_mode;
            return Some(new_mode);
        }

        // Recover if under recovery threshold
        if overhead < self.config.recovery_threshold {
            let new_mode = match current {
                CoordinationMode::Disabled => CoordinationMode::HeartbeatOnly,
                CoordinationMode::HeartbeatOnly => CoordinationMode::Reduced,
                CoordinationMode::Reduced => CoordinationMode::Standard,
                CoordinationMode::Standard => CoordinationMode::Full,
                CoordinationMode::Full => return None, // Already at maximum
            };

            tracing::info!(
                "Coordination overhead {:.1}% < {:.1}%, recovering from {:?} to {:?}",
                overhead,
                self.config.recovery_threshold,
                current,
                new_mode
            );

            *self.current_mode.write().await = new_mode;
            return Some(new_mode);
        }

        // In hysteresis zone (between recovery and degradation thresholds)
        None
    }

    /// Start periodic evaluation loop
    ///
    /// Spawns a background task that evaluates metrics at check_interval.
    /// Runs indefinitely until dropped.
    pub async fn run(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.config.check_interval);

        loop {
            interval.tick().await;

            if self.config.auto_degrade {
                if let Some(new_mode) = self.evaluate().await {
                    tracing::info!("Coordination mode changed to: {:?}", new_mode);
                }
            }
        }
    }

    /// Get current coordination mode
    pub async fn current_mode(&self) -> CoordinationMode {
        *self.current_mode.read().await
    }

    /// Force coordination mode (manual override)
    ///
    /// Disables auto-degradation until next evaluation.
    pub async fn force_mode(&self, mode: CoordinationMode) {
        tracing::info!("Forcing coordination mode to: {:?}", mode);
        *self.current_mode.write().await = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_counters_zero() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        assert_eq!(metrics.total_coordination_tokens(), 0);
        assert_eq!(metrics.total_production_tokens(), 0);
        assert_eq!(metrics.heartbeat_tokens(), 0);
        assert_eq!(metrics.standup_tokens(), 0);
    }

    #[test]
    fn test_record_coordination_increments() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_coordination(100, 50, "heartbeat");

        assert_eq!(metrics.total_coordination_tokens(), 150);
        assert_eq!(metrics.heartbeat_tokens(), 150);
    }

    #[test]
    fn test_record_production_increments() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_production(5000, 3000);

        assert_eq!(metrics.total_production_tokens(), 8000);
    }

    #[test]
    fn test_overhead_calculation_zero() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        assert_eq!(metrics.coordination_overhead(), 0.0);
    }

    #[test]
    fn test_overhead_calculation_30_percent() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_coordination(30, 0, "heartbeat");
        metrics.record_production(70, 0);

        let overhead = metrics.coordination_overhead();
        assert!((overhead - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_overhead_100_percent() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_coordination(100, 0, "heartbeat");

        let overhead = metrics.coordination_overhead();
        assert_eq!(overhead, 100.0);
    }

    #[test]
    fn test_protocol_breakdown() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_coordination(50, 30, "heartbeat");
        metrics.record_coordination(100, 80, "standup");

        assert_eq!(metrics.heartbeat_tokens(), 80);
        assert_eq!(metrics.standup_tokens(), 180);
        assert_eq!(metrics.total_coordination_tokens(), 260);
    }

    #[tokio::test]
    async fn test_reset_clears_counters() {
        let metrics = TokenMetrics::new(Duration::from_secs(3600));
        metrics.record_coordination(100, 50, "heartbeat");
        metrics.record_production(5000, 3000);

        assert_eq!(metrics.total_coordination_tokens(), 150);
        assert_eq!(metrics.total_production_tokens(), 8000);

        metrics.reset().await;

        assert_eq!(metrics.total_coordination_tokens(), 0);
        assert_eq!(metrics.total_production_tokens(), 0);
        assert_eq!(metrics.heartbeat_tokens(), 0);
    }

    #[tokio::test]
    async fn test_snapshot_serialization() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        metrics.record_coordination(50, 30, "heartbeat");
        metrics.record_production(5000, 3000);

        let snapshot = metrics.snapshot(CoordinationMode::Full).await;

        assert_eq!(snapshot.coordination_tokens, 80);
        assert_eq!(snapshot.production_tokens, 8000);
        assert!((snapshot.overhead_percent - 0.99).abs() < 0.01);
        assert_eq!(snapshot.current_mode, "Full");

        // Test serialization
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: MetricsSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.coordination_tokens, 80);
    }

    #[tokio::test]
    async fn test_concurrent_recording() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));

        // Spawn 10 tasks recording tokens simultaneously
        let mut handles = vec![];
        for _ in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    metrics_clone.record_coordination(10, 5, "heartbeat");
                    metrics_clone.record_production(100, 50);
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify totals (10 tasks * 100 iterations * tokens)
        assert_eq!(metrics.total_coordination_tokens(), 10 * 100 * 15);
        assert_eq!(metrics.total_production_tokens(), 10 * 100 * 150);
    }

    // DegradationManager tests

    #[tokio::test]
    async fn test_no_degradation_under_threshold() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        metrics.record_coordination(20, 0, "heartbeat");
        metrics.record_production(80, 0);

        let config = DegradationConfig::default();
        let manager = DegradationManager::new(config, metrics);

        let result = manager.evaluate().await;
        assert!(result.is_none()); // 20% overhead < 30% threshold
    }

    #[tokio::test]
    async fn test_degrade_at_threshold() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        metrics.record_coordination(35, 0, "heartbeat");
        metrics.record_production(65, 0);

        let config = DegradationConfig::default();
        let manager = DegradationManager::new(config, metrics);

        let result = manager.evaluate().await;
        assert_eq!(result, Some(CoordinationMode::Standard)); // Degraded from Full
    }

    #[tokio::test]
    async fn test_degrade_cascade() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        let config = DegradationConfig::default();
        let manager = Arc::new(DegradationManager::new(config, metrics.clone()));

        // Set overhead to 35% (over threshold)
        metrics.record_coordination(35, 0, "heartbeat");
        metrics.record_production(65, 0);

        // Degrade Full -> Standard
        let mode1 = manager.evaluate().await;
        assert_eq!(mode1, Some(CoordinationMode::Standard));

        // Degrade Standard -> Reduced
        let mode2 = manager.evaluate().await;
        assert_eq!(mode2, Some(CoordinationMode::Reduced));

        // Degrade Reduced -> HeartbeatOnly
        let mode3 = manager.evaluate().await;
        assert_eq!(mode3, Some(CoordinationMode::HeartbeatOnly));

        // Degrade HeartbeatOnly -> Disabled
        let mode4 = manager.evaluate().await;
        assert_eq!(mode4, Some(CoordinationMode::Disabled));

        // Already at Disabled, can't degrade further
        let mode5 = manager.evaluate().await;
        assert_eq!(mode5, None);
    }

    #[tokio::test]
    async fn test_recovery_under_recovery_threshold() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        let config = DegradationConfig::default();
        let manager = Arc::new(DegradationManager::new(config, metrics.clone()));

        // Start at HeartbeatOnly
        manager.force_mode(CoordinationMode::HeartbeatOnly).await;

        // Set overhead to 15% (under recovery threshold)
        metrics.record_coordination(15, 0, "heartbeat");
        metrics.record_production(85, 0);

        let result = manager.evaluate().await;
        assert_eq!(result, Some(CoordinationMode::Reduced));
    }

    #[tokio::test]
    async fn test_hysteresis_prevents_flapping() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        let config = DegradationConfig::default();
        let manager = DegradationManager::new(config, metrics.clone());

        // Set overhead to 25% (between recovery 20% and degrade 30%)
        metrics.record_coordination(25, 0, "heartbeat");
        metrics.record_production(75, 0);

        let result = manager.evaluate().await;
        assert!(result.is_none()); // In hysteresis zone, no change
    }

    #[tokio::test]
    async fn test_force_mode_overrides() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        let config = DegradationConfig::default();
        let manager = DegradationManager::new(config, metrics);

        manager.force_mode(CoordinationMode::Disabled).await;

        let mode = manager.current_mode().await;
        assert_eq!(mode, CoordinationMode::Disabled);
    }

    #[tokio::test]
    async fn test_disabled_auto_degrade() {
        let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));
        let mut config = DegradationConfig::default();
        config.auto_degrade = false;

        let manager = DegradationManager::new(config, metrics.clone());

        // Set overhead to 50% (way over threshold)
        metrics.record_coordination(50, 0, "heartbeat");
        metrics.record_production(50, 0);

        let result = manager.evaluate().await;
        assert!(result.is_none()); // auto_degrade=false, no change
    }
}
