//! Behavioral anomaly detection for credential access
//!
//! This module provides anomaly detection based on behavioral baselines established
//! over time. It tracks access frequency, volume, time-of-day patterns, and burst
//! behavior to identify suspicious credential access.

use aof_core::credential::{CredentialAccessAnomaly, CredentialType};
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

/// Anomaly detector for credential access patterns
pub struct AnomalyDetector {
    baselines: RwLock<HashMap<String, AgentBaseline>>,
    learning_mode: AtomicBool,
    access_history: RwLock<Vec<CredentialAccessRecord>>,
}

/// Baseline behavioral pattern for an agent
#[derive(Debug, Clone)]
pub struct AgentBaseline {
    pub agent_id: String,
    pub credential_type: CredentialType,
    pub access_frequency: FrequencyBaseline,
    pub access_volume: VolumeBaseline,
    pub active_hours: Vec<u32>,
    pub established_at: DateTime<Utc>,
}

/// Frequency baseline (time between accesses)
#[derive(Debug, Clone)]
pub struct FrequencyBaseline {
    pub mean_interval_secs: f64,
    pub stddev_interval_secs: f64,
}

impl Default for FrequencyBaseline {
    fn default() -> Self {
        Self {
            mean_interval_secs: 3600.0, // 1 hour default
            stddev_interval_secs: 1800.0, // 30 minutes default
        }
    }
}

/// Volume baseline (accesses per day)
#[derive(Debug, Clone)]
pub struct VolumeBaseline {
    pub mean_daily_accesses: f64,
    pub stddev_daily_accesses: f64,
}

impl Default for VolumeBaseline {
    fn default() -> Self {
        Self {
            mean_daily_accesses: 10.0,
            stddev_daily_accesses: 5.0,
        }
    }
}

/// Record of a credential access for baseline learning
#[derive(Debug, Clone)]
struct CredentialAccessRecord {
    agent_id: String,
    credential_type: CredentialType,
    timestamp: DateTime<Utc>,
}

impl AnomalyDetector {
    /// Create a new anomaly detector in learning mode
    pub fn new() -> Self {
        Self {
            baselines: RwLock::new(HashMap::new()),
            learning_mode: AtomicBool::new(true),
            access_history: RwLock::new(Vec::new()),
        }
    }

    /// Record a credential access for baseline learning
    ///
    /// # Arguments
    /// * `agent_id` - Agent performing the access
    /// * `credential_type` - Type of credential accessed
    pub fn record_access(&self, agent_id: &str, credential_type: &CredentialType) {
        let mut history = self.access_history.write().unwrap();
        history.push(CredentialAccessRecord {
            agent_id: agent_id.to_string(),
            credential_type: credential_type.clone(),
            timestamp: Utc::now(),
        });

        // Establish baseline if we have enough samples (>= 10)
        let agent_key = format!("{}:{}", agent_id, credential_type.name());
        let agent_records: Vec<_> = history
            .iter()
            .filter(|r| r.agent_id == agent_id && r.credential_type == *credential_type)
            .cloned()
            .collect();

        if agent_records.len() >= 10 {
            if let Some(baseline) = self.calculate_baseline(agent_id, credential_type, &agent_records) {
                let mut baselines = self.baselines.write().unwrap();
                baselines.insert(agent_key, baseline);
            }
        }
    }

    /// Calculate baseline from historical records
    fn calculate_baseline(
        &self,
        agent_id: &str,
        credential_type: &CredentialType,
        records: &[CredentialAccessRecord],
    ) -> Option<AgentBaseline> {
        if records.len() < 10 {
            return None;
        }

        // Calculate frequency baseline (intervals between accesses)
        let mut intervals = Vec::new();
        for i in 1..records.len() {
            let interval = (records[i].timestamp - records[i - 1].timestamp)
                .num_seconds() as f64;
            intervals.push(interval);
        }

        let freq_baseline = if !intervals.is_empty() {
            let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
            let variance = intervals
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>()
                / intervals.len() as f64;
            let stddev = variance.sqrt();

            FrequencyBaseline {
                mean_interval_secs: mean,
                stddev_interval_secs: stddev,
            }
        } else {
            FrequencyBaseline::default()
        };

        // Calculate volume baseline (accesses per day)
        let first = records.first()?;
        let last = records.last()?;
        let days = (last.timestamp - first.timestamp).num_days() as f64;
        let days = if days < 1.0 { 1.0 } else { days };
        let daily_accesses = records.len() as f64 / days;

        let vol_baseline = VolumeBaseline {
            mean_daily_accesses: daily_accesses,
            stddev_daily_accesses: (daily_accesses * 0.5).max(1.0), // 50% stddev
        };

        // Extract active hours (hours when accesses occur)
        let mut hour_counts: HashMap<u32, usize> = HashMap::new();
        for record in records {
            *hour_counts.entry(record.timestamp.hour()).or_insert(0) += 1;
        }

        let active_hours: Vec<u32> = hour_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2) // Must occur at least twice to be considered active
            .map(|(hour, _)| hour)
            .collect();

        Some(AgentBaseline {
            agent_id: agent_id.to_string(),
            credential_type: credential_type.clone(),
            access_frequency: freq_baseline,
            access_volume: vol_baseline,
            active_hours,
            established_at: Utc::now(),
        })
    }

    /// Score a credential access against established baseline
    ///
    /// # Arguments
    /// * `agent_id` - Agent requesting access
    /// * `credential_type` - Type of credential being accessed
    ///
    /// # Returns
    /// Anomaly with score (0.0-1.0) and reasons
    pub async fn score_access(
        &self,
        agent_id: &str,
        credential_type: &CredentialType,
    ) -> CredentialAccessAnomaly {
        // Always allow during learning period
        if self.learning_mode.load(Ordering::SeqCst) {
            return CredentialAccessAnomaly::new(
                agent_id.to_string(),
                credential_type.clone(),
                0.0,
                vec!["Learning mode active".to_string()],
            );
        }

        let agent_key = format!("{}:{}", agent_id, credential_type.name());
        let baselines = self.baselines.read().unwrap();
        let baseline = match baselines.get(&agent_key) {
            Some(b) => b,
            None => {
                // No baseline established yet, score as 0.0 (allow)
                return CredentialAccessAnomaly::new(
                    agent_id.to_string(),
                    credential_type.clone(),
                    0.0,
                    vec!["No baseline established".to_string()],
                );
            }
        };

        // Calculate anomaly score components
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // 1. Frequency anomaly (0.0-0.4)
        let history = self.access_history.read().unwrap();
        let recent_accesses: Vec<_> = history
            .iter()
            .filter(|r| r.agent_id == agent_id && r.credential_type == *credential_type)
            .rev()
            .take(2)
            .collect();

        if recent_accesses.len() >= 2 {
            let interval = (recent_accesses[0].timestamp - recent_accesses[1].timestamp)
                .num_seconds() as f64;
            let baseline_min = baseline.access_frequency.mean_interval_secs * 0.1;

            if interval < baseline_min {
                let freq_score = 0.4 * (1.0 - (interval / baseline_min));
                score += freq_score;
                reasons.push(format!(
                    "Frequency spike: {} seconds vs baseline {} seconds",
                    interval as u64, baseline.access_frequency.mean_interval_secs as u64
                ));
            }
        }

        // 2. Volume anomaly (0.0-0.3)
        let today_date = Utc::now().date_naive();
        let today_accesses = history
            .iter()
            .filter(|r| {
                r.agent_id == agent_id
                    && r.credential_type == *credential_type
                    && r.timestamp.date_naive() == today_date
            })
            .count() as f64;

        let volume_threshold = baseline.access_volume.mean_daily_accesses * 3.0;
        if today_accesses > volume_threshold {
            let vol_score = 0.3 * ((today_accesses - volume_threshold) / volume_threshold).min(1.0);
            score += vol_score;
            reasons.push(format!(
                "Volume spike: {} accesses today vs baseline {}",
                today_accesses as u64, baseline.access_volume.mean_daily_accesses as u64
            ));
        }

        // 3. Time-of-day anomaly (0.0-0.2)
        let current_hour = Utc::now().hour();
        if !baseline.active_hours.is_empty() && !baseline.active_hours.contains(&current_hour) {
            score += 0.2;
            reasons.push(format!(
                "Unusual time: hour {} not in active hours {:?}",
                current_hour, baseline.active_hours
            ));
        }

        // 4. Burst anomaly (0.0-0.3)
        let now = Utc::now();
        let burst_window = chrono::Duration::seconds(60);
        let burst_count = history
            .iter()
            .filter(|r| {
                r.agent_id == agent_id
                    && r.credential_type == *credential_type
                    && (now - r.timestamp) < burst_window
            })
            .count();

        if burst_count > 5 {
            let burst_score = 0.3 * ((burst_count as f64 - 5.0) / 5.0).min(1.0);
            score += burst_score;
            reasons.push(format!("Burst detected: {} accesses in 60 seconds", burst_count));
        }

        // Cap score at 1.0
        score = score.min(1.0);

        if reasons.is_empty() {
            reasons.push("Normal access pattern".to_string());
        }

        CredentialAccessAnomaly::new(
            agent_id.to_string(),
            credential_type.clone(),
            score,
            reasons,
        )
    }

    /// Get baseline for an agent
    pub fn get_baseline(&self, agent_id: &str, credential_type: &CredentialType) -> Option<AgentBaseline> {
        let agent_key = format!("{}:{}", agent_id, credential_type.name());
        let baselines = self.baselines.read().unwrap();
        baselines.get(&agent_key).cloned()
    }

    /// Check if detector is in learning mode
    pub fn is_learning(&self) -> bool {
        self.learning_mode.load(Ordering::SeqCst)
    }

    /// Exit learning mode (call after 7 days or sufficient data)
    pub fn exit_learning_mode(&self) {
        self.learning_mode.store(false, Ordering::SeqCst);
        tracing::info!("Anomaly detector exited learning mode");
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_learning_mode_returns_zero_score() {
        let detector = AnomalyDetector::new();
        assert!(detector.is_learning());

        let anomaly = detector.score_access("agent-1", &CredentialType::Kubernetes).await;
        assert_eq!(anomaly.anomaly_score, 0.0);
        assert!(anomaly.reasons[0].contains("Learning mode"));
    }

    #[tokio::test]
    async fn test_normal_access_low_score() {
        let detector = AnomalyDetector::new();
        detector.exit_learning_mode();

        // Record normal baseline pattern (10 accesses over 10 hours)
        for i in 0..10 {
            detector.record_access("agent-1", &CredentialType::Kubernetes);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let anomaly = detector.score_access("agent-1", &CredentialType::Kubernetes).await;
        assert!(anomaly.anomaly_score < 0.5);
    }

    #[test]
    fn test_record_access_builds_baseline() {
        let detector = AnomalyDetector::new();

        // Record 10 accesses to establish baseline
        for _ in 0..10 {
            detector.record_access("agent-1", &CredentialType::Kubernetes);
        }

        let baseline = detector.get_baseline("agent-1", &CredentialType::Kubernetes);
        assert!(baseline.is_some());
    }

    #[test]
    fn test_no_baseline_before_10_samples() {
        let detector = AnomalyDetector::new();

        // Record only 5 accesses
        for _ in 0..5 {
            detector.record_access("agent-1", &CredentialType::Kubernetes);
        }

        let baseline = detector.get_baseline("agent-1", &CredentialType::Kubernetes);
        assert!(baseline.is_none());
    }

    #[tokio::test]
    async fn test_exit_learning_mode() {
        let detector = AnomalyDetector::new();
        assert!(detector.is_learning());

        detector.exit_learning_mode();
        assert!(!detector.is_learning());
    }
}
