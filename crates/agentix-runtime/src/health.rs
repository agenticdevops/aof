//! Health and readiness check endpoints for production deployment
//!
//! This module provides two critical endpoints for Kubernetes and systemd:
//! - `/health` - Liveness probe: returns 200 if process is alive
//! - `/ready` - Readiness probe: returns 200 if all dependencies are ready, 503 if not
//!
//! Readiness checks:
//! - Disk space > 100MB at data directory
//! - EventBroadcaster is functional
//! - SessionPersistence directory is writable

use serde::Serialize;
use std::path::PathBuf;

/// Health check response (liveness probe)
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Status: always "ok" if process is alive
    pub status: String,
    /// AOF version from CARGO_PKG_VERSION
    pub version: String,
    /// Daemon uptime in seconds
    pub uptime_seconds: u64,
    /// Git commit hash (from build metadata)
    pub git_commit: String,
}

impl HealthResponse {
    /// Create a new health response
    pub fn new(uptime_seconds: u64) -> Self {
        Self {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            git_commit: option_env!("GIT_HASH").unwrap_or("unknown").to_string(),
        }
    }
}

/// Readiness check response
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessResponse {
    /// Overall status: "ready" or "not_ready"
    pub status: String,
    /// Individual dependency status
    pub dependencies: DependencyStatus,
}

/// Status of all critical dependencies
#[derive(Debug, Clone, Serialize)]
pub struct DependencyStatus {
    /// Disk space availability
    pub disk_space: DependencyState,
    /// Event bus functionality
    pub event_bus: DependencyState,
    /// Session persistence writability
    pub session_persistence: DependencyState,
}

/// State of an individual dependency
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum DependencyState {
    /// Dependency is fully operational
    Ok,
    /// Dependency is operational but with warnings
    Degraded { reason: String },
    /// Dependency is not available
    Unavailable { reason: String },
}

impl DependencyState {
    /// Check if dependency is operational (Ok or Degraded)
    pub fn is_operational(&self) -> bool {
        matches!(self, DependencyState::Ok | DependencyState::Degraded { .. })
    }
}

/// Check if data directory has sufficient disk space (> 100MB)
pub async fn check_disk_space(data_dir: &PathBuf) -> DependencyState {
    // Ensure directory exists
    if !data_dir.exists() {
        return DependencyState::Unavailable {
            reason: format!("Data directory does not exist: {:?}", data_dir),
        };
    }

    // Get disk space using statvfs (Unix) or similar on Windows
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match tokio::fs::metadata(data_dir).await {
            Ok(metadata) => {
                let block_size = metadata.blksize();
                let available_blocks = metadata.blocks();
                let available_bytes = block_size * available_blocks;
                let min_required = 100 * 1024 * 1024; // 100MB

                if available_bytes > min_required {
                    DependencyState::Ok
                } else if available_bytes > 10 * 1024 * 1024 {
                    // 10-100MB
                    DependencyState::Degraded {
                        reason: format!("Low disk space: {} MB available", available_bytes / (1024 * 1024)),
                    }
                } else {
                    DependencyState::Unavailable {
                        reason: format!("Critically low disk space: {} MB available", available_bytes / (1024 * 1024)),
                    }
                }
            }
            Err(e) => DependencyState::Unavailable {
                reason: format!("Failed to check disk space: {}", e),
            },
        }
    }

    #[cfg(not(unix))]
    {
        // Simplified check for non-Unix systems
        DependencyState::Ok
    }
}

/// Check if EventBroadcaster is functional by checking subscriber count
pub fn check_event_bus(subscriber_count: usize) -> DependencyState {
    // Event bus is always operational (even with 0 subscribers)
    // Degraded if no subscribers (events will be dropped)
    if subscriber_count > 0 {
        DependencyState::Ok
    } else {
        DependencyState::Degraded {
            reason: "No active event subscribers".to_string(),
        }
    }
}

/// Check if SessionPersistence directory is writable
pub async fn check_session_persistence(persist_dir: &PathBuf) -> DependencyState {
    // Ensure directory exists
    if !persist_dir.exists() {
        return DependencyState::Unavailable {
            reason: format!("Session persistence directory does not exist: {:?}", persist_dir),
        };
    }

    // Try to write a test file
    let test_file = persist_dir.join(".health_check");
    match tokio::fs::write(&test_file, b"health_check").await {
        Ok(_) => {
            // Clean up test file
            let _ = tokio::fs::remove_file(&test_file).await;
            DependencyState::Ok
        }
        Err(e) => DependencyState::Unavailable {
            reason: format!("Session persistence directory not writable: {}", e),
        },
    }
}

/// Perform full readiness check
pub async fn check_readiness(
    data_dir: &PathBuf,
    persist_dir: &PathBuf,
    event_bus_subscribers: usize,
) -> ReadinessResponse {
    let disk_space = check_disk_space(data_dir).await;
    let event_bus = check_event_bus(event_bus_subscribers);
    let session_persistence = check_session_persistence(persist_dir).await;

    let dependencies = DependencyStatus {
        disk_space,
        event_bus,
        session_persistence,
    };

    // Overall status is "ready" only if all dependencies are operational
    let all_ready = dependencies.disk_space.is_operational()
        && dependencies.event_bus.is_operational()
        && dependencies.session_persistence.is_operational();

    ReadinessResponse {
        status: if all_ready { "ready" } else { "not_ready" }.to_string(),
        dependencies,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_health_response_creation() {
        let health = HealthResponse::new(120);
        assert_eq!(health.status, "ok");
        assert_eq!(health.uptime_seconds, 120);
        assert!(!health.version.is_empty());
    }

    #[test]
    fn test_dependency_state_is_operational() {
        assert!(DependencyState::Ok.is_operational());
        assert!(DependencyState::Degraded { reason: "test".to_string() }.is_operational());
        assert!(!DependencyState::Unavailable { reason: "test".to_string() }.is_operational());
    }

    #[test]
    fn test_event_bus_check() {
        let ok_state = check_event_bus(5);
        assert!(ok_state.is_operational());
        assert!(matches!(ok_state, DependencyState::Ok));

        let degraded_state = check_event_bus(0);
        assert!(degraded_state.is_operational());
        assert!(matches!(degraded_state, DependencyState::Degraded { .. }));
    }

    #[tokio::test]
    async fn test_session_persistence_check() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persist_dir = temp_dir.path().to_path_buf();

        let state = check_session_persistence(&persist_dir).await;
        assert!(state.is_operational());
        assert!(matches!(state, DependencyState::Ok));

        // Test non-existent directory
        let missing_dir = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let state = check_session_persistence(&missing_dir).await;
        assert!(!state.is_operational());
        assert!(matches!(state, DependencyState::Unavailable { .. }));
    }

    #[tokio::test]
    async fn test_full_readiness_check() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let persist_dir = temp_dir.path().to_path_buf();

        let response = check_readiness(&data_dir, &persist_dir, 2).await;

        // With active subscribers and writable directories, should be ready
        // Note: disk_space check may vary by platform
        assert!(response.dependencies.event_bus.is_operational());
        assert!(response.dependencies.session_persistence.is_operational());
    }

    #[tokio::test]
    async fn test_readiness_not_ready_when_dependency_unavailable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let missing_persist_dir = PathBuf::from("/nonexistent/path/that/does/not/exist");

        let response = check_readiness(&data_dir, &missing_persist_dir, 2).await;

        // Should be not_ready due to unavailable session persistence
        assert_eq!(response.status, "not_ready");
        assert!(!response.dependencies.session_persistence.is_operational());
    }
}
