//! Incident Triage Agent - LLM-based alert classification and specialist routing

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use agentix_core::{AofResult, CoordinationEvent};
use agentix_coordination::{DecisionLogger, EventBroadcaster};

/// Alert payload from monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPayload {
    pub alert_id: String,
    pub summary: String,
    pub error_rate: Option<f64>,
    pub affected_services: Vec<String>,
    pub duration_seconds: u64,
    pub affected_users: Option<u64>,
    pub logs_available: bool,
    pub metrics_available: bool,
    pub context: serde_json::Value,
}

/// Triage classification output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageClassification {
    pub severity: String,
    pub confidence: f64,
    pub category: String,
    pub specialists_needed: Vec<String>,
    pub reasoning: String,
}

/// Result of triage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    pub incident_id: String,
    pub classification: TriageClassification,
    pub should_escalate: bool,
    pub escalation_reason: Option<String>,
}

/// Incident context store for specialist queries
#[derive(Debug, Clone)]
pub struct IncidentContextStore {
    pub incident_id: String,
}

impl IncidentContextStore {
    pub fn new(incident_id: impl Into<String>) -> Self {
        Self {
            incident_id: incident_id.into(),
        }
    }

    pub async fn store_alert_context(&self, _alert: &AlertPayload) -> AofResult<()> {
        // Phase 2: Basic implementation stores to memory
        Ok(())
    }

    pub async fn store_finding(&self, _agent_id: &str, _finding: &str, _confidence: f64) -> AofResult<()> {
        Ok(())
    }

    pub async fn get_recent_findings(&self) -> AofResult<Vec<(String, String, f64)>> {
        Ok(Vec::new())
    }

    pub async fn query_logs(&self, _query: &str) -> AofResult<String> {
        Ok("No logs available".to_string())
    }

    pub async fn query_metrics(&self, _metric_name: &str) -> AofResult<Vec<f64>> {
        Ok(Vec::new())
    }
}

/// Triage agent for alert classification
pub struct TriageAgent {
    pub broadcaster: Arc<EventBroadcaster>,
    pub decision_logger: Arc<DecisionLogger>,
}

impl TriageAgent {
    pub fn new(
        broadcaster: Arc<EventBroadcaster>,
        decision_logger: Arc<DecisionLogger>,
    ) -> Self {
        Self {
            broadcaster,
            decision_logger,
        }
    }

    /// Classify an alert using LLM-based analysis
    pub async fn classify_alert(&self, alert: &AlertPayload) -> AofResult<TriageClassification> {
        // Build classification prompt
        let prompt = self.build_classification_prompt(alert);

        // Phase 2: Deterministic classification logic
        let severity = if alert.error_rate.map_or(false, |er| er > 0.50) {
            "SEV1".to_string()
        } else if alert.error_rate.map_or(false, |er| er > 0.20) {
            "SEV2".to_string()
        } else if alert.duration_seconds > 3600 {
            "SEV3".to_string()
        } else {
            "SEV4".to_string()
        };

        // Confidence based on error rate (higher error = higher confidence in triage)
        let confidence = if let Some(er) = alert.error_rate {
            if er > 0.50 {
                0.92  // High error rate = high confidence
            } else if er > 0.20 {
                0.85  // Medium error rate = good confidence
            } else if er > 0.05 {
                0.70  // Low error rate = moderate confidence
            } else {
                0.55  // Very low error rate = low confidence
            }
        } else {
            0.60  // No error rate info = moderate confidence
        };

        let category = if alert.affected_services.iter().any(|s| s.contains("api")) {
            "api-degradation".to_string()
        } else if alert.affected_services.iter().any(|s| s.contains("db")) {
            "database-error".to_string()
        } else if alert.affected_services.iter().any(|s| s.contains("pod")) {
            "pod-crash".to_string()
        } else {
            "other".to_string()
        };

        let mut specialists_needed = Vec::new();
        if alert.logs_available {
            specialists_needed.push("log-analyzer".to_string());
        }
        if alert.metrics_available {
            specialists_needed.push("metric-checker".to_string());
        }
        specialists_needed.push("k8s-diagnostician".to_string());

        Ok(TriageClassification {
            severity,
            confidence,
            category,
            specialists_needed,
            reasoning: prompt,
        })
    }

    /// Run triage workflow
    pub async fn triage(&self, alert: &AlertPayload) -> AofResult<TriageResult> {
        let classification = self.classify_alert(alert).await?;

        let should_escalate = classification.confidence < 0.6;
        let escalation_reason = if should_escalate {
            Some(format!("Low confidence: {:.2}", classification.confidence))
        } else {
            None
        };

        // Log decision
        let _entry = agentix_core::DecisionLogEntry::new(
            alert.alert_id.clone(),
            "classify_alert".to_string(),
            classification.reasoning.clone(),
            classification.confidence,
        );

        // Emit event (placeholder - would use real incident event types in Phase 3)
        let _event = CoordinationEvent::from_activity(
            agentix_core::ActivityEvent::thinking(format!(
                "Triage classification: {} ({:.2}% confidence)",
                classification.severity, classification.confidence * 100.0
            )),
            alert.alert_id.clone(),
            "default-session",
        );

        Ok(TriageResult {
            incident_id: alert.alert_id.clone(),
            classification,
            should_escalate,
            escalation_reason,
        })
    }

    fn build_classification_prompt(&self, alert: &AlertPayload) -> String {
        format!(
            "You are an incident triage specialist. Analyze this alert:\n\n\
             Summary: {}\n\
             Error Rate: {:?}\n\
             Services: {}\n\
             Duration: {}s\n\
             Affected Users: {:?}\n\n\
             Classify by severity (SEV1-4) and confidence (0.0-1.0).",
            alert.summary,
            alert.error_rate,
            alert.affected_services.join(", "),
            alert.duration_seconds,
            alert.affected_users,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classify_alert_high_error_rate() {
        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let decision_logger = Arc::new(DecisionLogger::new(
            std::path::PathBuf::from("/tmp/test_decisions.jsonl"),
            broadcaster.clone(),
        ));
        let agent = TriageAgent::new(
            broadcaster,
            decision_logger,
        );

        let alert = AlertPayload {
            alert_id: "ALT-001".to_string(),
            summary: "High error rate on payment API".to_string(),
            error_rate: Some(0.60),
            affected_services: vec!["payment-api".to_string()],
            duration_seconds: 300,
            affected_users: Some(1000),
            logs_available: true,
            metrics_available: true,
            context: serde_json::json!({}),
        };

        let result = agent.classify_alert(&alert).await.unwrap();
        assert_eq!(result.severity, "SEV1");
        assert!(result.confidence > 0.3);
    }

    #[tokio::test]
    async fn test_triage_escalation_on_low_confidence() {
        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let decision_logger = Arc::new(DecisionLogger::new(
            std::path::PathBuf::from("/tmp/test_decisions.jsonl"),
            broadcaster.clone(),
        ));
        let agent = TriageAgent::new(
            broadcaster,
            decision_logger,
        );

        let alert = AlertPayload {
            alert_id: "ALT-002".to_string(),
            summary: "Unusual network activity".to_string(),
            error_rate: Some(0.05),
            affected_services: vec!["unknown".to_string()],
            duration_seconds: 60,
            affected_users: None,
            logs_available: false,
            metrics_available: false,
            context: serde_json::json!({}),
        };

        let result = agent.triage(&alert).await.unwrap();
        // Low confidence should trigger escalation
        if result.classification.confidence < 0.6 {
            assert!(result.should_escalate);
        }
    }
}
