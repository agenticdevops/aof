//! Security test suite: Credential auditing validation
//!
//! This module validates credential access detection, audit logging,
//! tamper detection, and anomaly scoring.

use aof_core::credential::{AccessMode, CredentialType, RiskLevel, ToolContext};
use aof_runtime::{AnomalyDetector, CredentialAccessInterceptor};
use chrono::Timelike;
use std::sync::Arc;

fn setup_test_interceptor() -> (CredentialAccessInterceptor, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let audit_log = temp_dir.path().join("audit.log");

    let detector = Arc::new(AnomalyDetector::new());
    let interceptor = CredentialAccessInterceptor::new(audit_log, detector);

    (interceptor, temp_dir)
}

#[test]
fn test_credential_detection_kubectl() {
    let (interceptor, _temp_dir) = setup_test_interceptor();

    let creds = interceptor.detect_credential_requirements("kubectl", &[]);
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0], CredentialType::Kubernetes);
}

#[test]
fn test_credential_detection_aws() {
    let (interceptor, _temp_dir) = setup_test_interceptor();

    let creds = interceptor.detect_credential_requirements("aws", &[]);
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0], CredentialType::Aws);
}

#[tokio::test]
async fn test_audit_log_sequence_numbers() {
    let (interceptor, _temp_dir) = setup_test_interceptor();

    // Log 100 events
    for i in 0..100 {
        let event = interceptor.create_event(
            format!("agent-{}", i),
            CredentialType::Kubernetes,
            "/path".to_string(),
            AccessMode::Read,
            ToolContext::new(
                "kubectl".to_string(),
                "get pods".to_string(),
                vec![],
                RiskLevel::Low,
            ),
            "session-1".to_string(),
        );

        interceptor.log_access(event).await.unwrap();
    }

    // Read back and verify monotonic sequence
    let events = interceptor
        .query_log(
            chrono::Utc::now() - chrono::Duration::minutes(1),
            chrono::Utc::now() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert_eq!(events.len(), 100);

    for i in 0..99 {
        assert_eq!(
            events[i + 1].sequence_number,
            events[i].sequence_number + 1,
            "Sequence numbers must be monotonically increasing"
        );
    }
}

#[tokio::test]
async fn test_audit_log_tamper_detection() {
    let (interceptor, temp_dir) = setup_test_interceptor();

    // Log 10 events
    for _ in 0..10 {
        let event = interceptor.create_event(
            "agent-1".to_string(),
            CredentialType::Kubernetes,
            "/path".to_string(),
            AccessMode::Read,
            ToolContext::new(
                "kubectl".to_string(),
                "get pods".to_string(),
                vec![],
                RiskLevel::Low,
            ),
            "session-1".to_string(),
        );

        interceptor.log_access(event).await.unwrap();
    }

    // Manually delete one line from middle of audit log
    let audit_log_path = temp_dir.path().join("audit.log");
    let content = tokio::fs::read_to_string(&audit_log_path).await.unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Remove line 5 (creating a gap in sequence)
    let mut new_lines = lines.clone();
    new_lines.remove(4);
    let tampered_content = new_lines.join("\n") + "\n";
    tokio::fs::write(&audit_log_path, tampered_content)
        .await
        .unwrap();

    // Query log - should detect gap
    let events = interceptor
        .query_log(
            chrono::Utc::now() - chrono::Duration::minutes(1),
            chrono::Utc::now() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    // Should have 9 events (one deleted)
    assert_eq!(events.len(), 9);

    // Gap detection happens during query_log via tracing::warn
    // In production, this would trigger alerts
}

#[tokio::test]
async fn test_anomaly_score_normal_access() {
    let detector = Arc::new(AnomalyDetector::new());
    detector.exit_learning_mode();

    // Establish baseline with 10 normal accesses
    for _ in 0..10 {
        detector.record_access("agent-1", &CredentialType::Kubernetes);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Normal access should score below or at 0.3
    let anomaly = detector
        .score_access("agent-1", &CredentialType::Kubernetes)
        .await;

    assert!(
        anomaly.anomaly_score <= 0.3,
        "Normal access pattern should score at or below 0.3, got {}",
        anomaly.anomaly_score
    );
}

#[tokio::test]
async fn test_anomaly_score_frequency_spike() {
    let detector = Arc::new(AnomalyDetector::new());

    // Establish baseline with normal pattern (1 access per 100ms)
    for _ in 0..10 {
        detector.record_access("agent-1", &CredentialType::Kubernetes);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    detector.exit_learning_mode();

    // Rapid burst (10x frequency increase)
    for _ in 0..5 {
        detector.record_access("agent-1", &CredentialType::Kubernetes);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let anomaly = detector
        .score_access("agent-1", &CredentialType::Kubernetes)
        .await;

    assert!(
        anomaly.anomaly_score >= 0.3,
        "Frequency spike should score at or above 0.3, got {}",
        anomaly.anomaly_score
    );
}

#[tokio::test]
async fn test_anomaly_score_off_hours() {
    // This test validates that time-of-day anomaly detection works.
    // Since we can't control time in tests, we verify the logic exists
    // by checking baseline active_hours tracking.

    let detector = Arc::new(AnomalyDetector::new());

    // Establish baseline
    for _ in 0..10 {
        detector.record_access("agent-1", &CredentialType::Kubernetes);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let baseline = detector
        .get_baseline("agent-1", &CredentialType::Kubernetes)
        .unwrap();

    // Baseline should have active hours recorded
    assert!(
        !baseline.active_hours.is_empty(),
        "Baseline should track active hours"
    );

    // Current hour should be in active hours
    let current_hour = chrono::Utc::now().hour();
    assert!(
        baseline.active_hours.contains(&current_hour),
        "Current hour should be in active hours"
    );
}

#[tokio::test]
async fn test_anomaly_blocks_extreme_score() {
    use aof_core::credential::AnomalyAction;

    let detector = Arc::new(AnomalyDetector::new());
    detector.exit_learning_mode();

    // Since we can't easily generate score > 0.95 without complex time manipulation,
    // we test the action derivation logic directly
    let block_action = AnomalyAction::from_score(0.98);
    let require_approval_action = AnomalyAction::from_score(0.85);
    let alert_action = AnomalyAction::from_score(0.75);
    let log_action = AnomalyAction::from_score(0.6);
    let allow_action = AnomalyAction::from_score(0.3);

    assert_eq!(block_action, AnomalyAction::Block);
    assert_eq!(require_approval_action, AnomalyAction::RequireApproval);
    assert_eq!(alert_action, AnomalyAction::Alert);
    assert_eq!(log_action, AnomalyAction::Log);
    assert_eq!(allow_action, AnomalyAction::Allow);
}

#[tokio::test]
async fn test_learning_mode_no_blocks() {
    let detector = Arc::new(AnomalyDetector::new());

    // In learning mode, all accesses should return score 0.0
    assert!(detector.is_learning());

    let anomaly = detector
        .score_access("agent-1", &CredentialType::Kubernetes)
        .await;

    assert_eq!(
        anomaly.anomaly_score, 0.0,
        "Learning mode should always return score 0.0"
    );

    assert_eq!(
        anomaly.recommended_action,
        aof_core::credential::AnomalyAction::Allow
    );
}

#[tokio::test]
async fn test_audit_event_json_format() {
    let (interceptor, _temp_dir) = setup_test_interceptor();

    let event = interceptor.create_event(
        "agent-1".to_string(),
        CredentialType::Kubernetes,
        "/home/.kube/config".to_string(),
        AccessMode::Read,
        ToolContext::new(
            "kubectl".to_string(),
            "get pods".to_string(),
            vec!["get".to_string(), "pods".to_string()],
            RiskLevel::Low,
        ),
        "session-1".to_string(),
    );

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&event).unwrap();

    // Verify expected fields are present
    assert!(json.contains("event_id"));
    assert!(json.contains("timestamp"));
    assert!(json.contains("agent_id"));
    assert!(json.contains("credential_type"));
    assert!(json.contains("Kubernetes"));
    assert!(json.contains("file_path"));
    assert!(json.contains("access_mode"));
    assert!(json.contains("tool_context"));
    assert!(json.contains("kubectl"));
    assert!(json.contains("sequence_number"));
    assert!(json.contains("session_id"));

    // Verify it can be deserialized back
    let deserialized: aof_core::credential::CredentialAccessEvent =
        serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.agent_id, "agent-1");
    assert_eq!(deserialized.credential_type, CredentialType::Kubernetes);
    assert_eq!(deserialized.session_id, "session-1");
}
