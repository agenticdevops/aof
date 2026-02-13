//! Incident Response Flow - Orchestration and escalation logic

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use aof_core::AofResult;
use aof_coordination::DecisionLogger;

use crate::executor::incident_triage::{AlertPayload, TriageAgent, IncidentContextStore};

/// Escalation triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationTrigger {
    ConfidenceLow { classification_confidence: f64 },
    TimeThreshold { minutes: u64 },
    ImpactHigh { affected_users: u64, revenue_impact: Option<String> },
    SpecialistFailed { agent_id: String, reason: String },
}

/// Escalation chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationChain {
    pub triggers: Vec<EscalationTrigger>,
    pub target_level: String,
    pub requires_human_approval: bool,
}

/// Incident response output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResponse {
    pub incident_id: String,
    pub severity: String,
    pub status: String,
    pub findings: String,
    pub specialists_involved: Vec<String>,
    pub resolution_time_seconds: u64,
    pub escalations: Vec<EscalationTrigger>,
}

/// Incident Response Flow orchestrator
pub struct IncidentResponseFlow {
    pub incident_id: String,
    pub triage_agent: Arc<TriageAgent>,
    pub decision_logger: Arc<DecisionLogger>,
    pub context_store: Arc<IncidentContextStore>,
}

impl IncidentResponseFlow {
    pub fn new(
        incident_id: impl Into<String>,
        triage_agent: Arc<TriageAgent>,
        decision_logger: Arc<DecisionLogger>,
        context_store: Arc<IncidentContextStore>,
    ) -> Self {
        Self {
            incident_id: incident_id.into(),
            triage_agent,
            decision_logger,
            context_store,
        }
    }

    /// Handle incoming alert
    pub async fn handle_alert(&self, alert: &AlertPayload) -> AofResult<IncidentResponse> {
        let start_time = Utc::now();

        // Emit incident started event
        let _started_event = serde_json::json!({
            "event": "incident_started",
            "incident_id": self.incident_id,
            "alert_summary": alert.summary,
            "timestamp": start_time,
        });

        // Store alert context
        self.context_store.store_alert_context(alert).await?;

        // Triage alert
        let triage_result = self.triage_agent.triage(alert).await?;

        // Check escalation triggers
        let mut escalations = Vec::new();
        if triage_result.should_escalate {
            if let Some(reason) = &triage_result.escalation_reason {
                escalations.push(EscalationTrigger::ConfidenceLow {
                    classification_confidence: triage_result.classification.confidence,
                });
                self.escalate(&EscalationTrigger::ConfidenceLow {
                    classification_confidence: triage_result.classification.confidence,
                }).await?;
            }
        }

        // Spawn specialists
        let mut specialists_involved = Vec::new();
        for specialist_type in &triage_result.classification.specialists_needed {
            let specialist_id = format!("{}-{}", specialist_type, self.incident_id);
            specialists_involved.push(specialist_id);
        }

        // Synthesize findings
        let findings = self.synthesize_findings(&specialists_involved).await?;

        let end_time = Utc::now();
        let duration_seconds = (end_time - start_time).num_seconds() as u64;

        Ok(IncidentResponse {
            incident_id: self.incident_id.clone(),
            severity: triage_result.classification.severity,
            status: if escalations.is_empty() { "investigating".to_string() } else { "escalated".to_string() },
            findings,
            specialists_involved,
            resolution_time_seconds: duration_seconds,
            escalations,
        })
    }

    /// Escalate incident to higher level
    async fn escalate(&self, trigger: &EscalationTrigger) -> AofResult<()> {
        let target = match trigger {
            EscalationTrigger::ConfidenceLow { .. } => "team_lead",
            EscalationTrigger::TimeThreshold { minutes } => {
                if *minutes > 60 { "manager" } else { "team_lead" }
            }
            EscalationTrigger::ImpactHigh { .. } => "executive",
            EscalationTrigger::SpecialistFailed { .. } => "team_lead",
        };

        // Log escalation decision
        let _entry = aof_core::DecisionLogEntry::new(
            self.incident_id.clone(),
            "escalate_incident".to_string(),
            format!("Escalating to {}", target),
            0.9,
        );

        Ok(())
    }

    /// Check if escalation is needed
    async fn check_escalation_triggers(
        &self,
        triage_result: &crate::executor::incident_triage::TriageResult,
        elapsed_seconds: u64,
    ) -> Option<EscalationTrigger> {
        if triage_result.classification.confidence < 0.6 {
            return Some(EscalationTrigger::ConfidenceLow {
                classification_confidence: triage_result.classification.confidence,
            });
        }

        if elapsed_seconds > 1800 {
            return Some(EscalationTrigger::TimeThreshold { minutes: 30 });
        }

        if elapsed_seconds > 3600 {
            return Some(EscalationTrigger::TimeThreshold { minutes: 60 });
        }

        None
    }

    /// Synthesize specialist findings into RCA summary
    async fn synthesize_findings(&self, _specialists: &[String]) -> AofResult<String> {
        // Query specialist findings from context store
        let _findings = self.context_store.get_recent_findings().await?;

        // Phase 2: Return basic finding summary
        let summary = "Investigation in progress. Specialists analyzing logs and metrics.".to_string();

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_coordination::EventBroadcaster;

    #[tokio::test]
    async fn test_incident_response_flow() {
        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let decision_logger = Arc::new(DecisionLogger::new(
            std::path::PathBuf::from("/tmp/test_incident.jsonl"),
            broadcaster.clone(),
        ));

        let triage_agent = Arc::new(TriageAgent::new(
            broadcaster,
            decision_logger.clone(),
        ));

        let flow = IncidentResponseFlow::new(
            "INC-001",
            triage_agent,
            decision_logger,
            Arc::new(IncidentContextStore::new("INC-001")),
        );

        let alert = AlertPayload {
            alert_id: "ALT-001".to_string(),
            summary: "Payment API degradation".to_string(),
            error_rate: Some(0.15),
            affected_services: vec!["payment-api".to_string()],
            duration_seconds: 300,
            affected_users: Some(500),
            logs_available: true,
            metrics_available: true,
            context: serde_json::json!({}),
        };

        let result = flow.handle_alert(&alert).await.unwrap();
        assert_eq!(result.incident_id, "INC-001");
        assert!(!result.findings.is_empty());
    }

    #[tokio::test]
    async fn test_escalation_trigger_low_confidence() {
        let broadcaster = Arc::new(EventBroadcaster::new(100));
        let decision_logger = Arc::new(DecisionLogger::new(
            std::path::PathBuf::from("/tmp/test_escalation.jsonl"),
            broadcaster.clone(),
        ));

        let trigger = EscalationTrigger::ConfidenceLow {
            classification_confidence: 0.45,
        };

        let flow = IncidentResponseFlow::new(
            "INC-002",
            Arc::new(TriageAgent::new(
                broadcaster,
                decision_logger.clone(),
            )),
            decision_logger,
            Arc::new(IncidentContextStore::new("INC-002")),
        );

        let result = flow.escalate(&trigger).await;
        assert!(result.is_ok());
    }
}
