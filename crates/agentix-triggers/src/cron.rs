// CronTrigger — schedule-based agent trigger
//
// CronTrigger is an *active* trigger: `start()` spawns a background tokio task that
// sleeps until the next scheduled time (per the cron expression), sends a TriggerEvent
// via the provided mpsc sender, then repeats. `stop()` sends a shutdown signal to
// terminate the background task cleanly.

use agentix_core::{AgentixError, TriggerEvent, TriggerSource, TriggerTrait};
use async_trait::async_trait;
use cron::Schedule;
use std::str::FromStr;
use tokio::sync::{Mutex, mpsc};

/// Fires a `TriggerEvent` on a cron schedule.
///
/// Construction validates the cron expression immediately — no surprises at runtime.
///
/// # Example
///
/// ```rust,ignore
/// let trigger = CronTrigger::new("0 9 * * 1", "weekly-reporter", "cron-weekly")?;
/// trigger.start(event_sender).await?;
/// ```
pub struct CronTrigger {
    id: String,
    expression: String,
    agent_name: String,
    /// Holds the stop sender until `stop()` is called.
    stop_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl CronTrigger {
    /// Construct and validate a new `CronTrigger`.
    ///
    /// Accepts both standard 5-field cron expressions (`min hour dom month dow`)
    /// and 7-field expressions with seconds and year (`sec min hour dom month dow year`).
    /// 5-field expressions are normalized by prepending `0` (seconds) and appending `*` (year).
    ///
    /// Returns `Err` immediately if `expression` is not a valid cron expression.
    pub fn new(
        expression: impl Into<String>,
        agent_name: impl Into<String>,
        trigger_id: impl Into<String>,
    ) -> Result<Self, AgentixError> {
        let raw_expr = expression.into();
        // Normalize 5-field to 7-field format required by the cron crate
        let normalized = normalize_cron_expression(&raw_expr);
        Schedule::from_str(&normalized)
            .map_err(|e| AgentixError::Config(format!("Invalid cron expression '{}': {}", raw_expr, e)))?;
        Ok(Self {
            id: trigger_id.into(),
            // Store the original expression (user-facing) but use normalized internally
            expression: raw_expr,
            agent_name: agent_name.into(),
            stop_tx: Mutex::new(None),
        })
    }

    /// Get the normalized (7-field) cron expression used internally.
    fn normalized_expression(&self) -> String {
        normalize_cron_expression(&self.expression)
    }

    /// Create a test TriggerEvent representing one firing of this cron trigger.
    ///
    /// Useful for unit tests that need to inspect the event payload without
    /// actually running the background loop.
    pub fn create_test_event(&self) -> TriggerEvent {
        TriggerEvent::new(
            TriggerSource::Cron,
            serde_json::json!({
                "expression": self.expression,
                "scheduled_at": chrono::Utc::now().to_rfc3339(),
            }),
            &self.id,
        )
    }
}

#[async_trait]
impl TriggerTrait for CronTrigger {
    fn trigger_id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> TriggerSource {
        TriggerSource::Cron
    }

    /// Spawn the cron loop. Sends `(agent_name, TriggerEvent)` on each scheduled tick.
    ///
    /// Returns immediately after spawning the background task.
    /// Call `stop()` to terminate the loop.
    async fn start(
        &self,
        sender: mpsc::Sender<(String, TriggerEvent)>,
    ) -> Result<(), AgentixError> {
        let normalized = self.normalized_expression();
        let schedule = Schedule::from_str(&normalized)
            .map_err(|e| AgentixError::Config(format!("Cron parse error: {}", e)))?;

        let agent_name = self.agent_name.clone();
        let trigger_id = self.id.clone();
        let expression = self.expression.clone();

        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        *self.stop_tx.lock().await = Some(stop_tx);

        tokio::spawn(async move {
            for next in schedule.upcoming(chrono::Utc) {
                let now = chrono::Utc::now();
                // Calculate sleep duration; clamp to zero if we're already past (shouldn't happen)
                let duration = (next - now).to_std().unwrap_or_default();

                tokio::select! {
                    _ = tokio::time::sleep(duration) => {
                        let event = TriggerEvent::new(
                            TriggerSource::Cron,
                            serde_json::json!({
                                "expression": expression,
                                "scheduled_at": next.to_rfc3339(),
                            }),
                            &trigger_id,
                        );
                        if sender.send((agent_name.clone(), event)).await.is_err() {
                            // Receiver dropped — no point continuing
                            break;
                        }
                    }
                    _ = &mut stop_rx => {
                        tracing::debug!("CronTrigger '{}' stopped", trigger_id);
                        break;
                    }
                }
            }
            tracing::debug!("CronTrigger loop exited");
        });

        Ok(())
    }

    /// Send the stop signal to terminate the background cron loop.
    async fn stop(&self) -> Result<(), AgentixError> {
        if let Some(tx) = self.stop_tx.lock().await.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalize a cron expression to the 7-field format required by the `cron` crate.
///
/// The `cron` crate requires `sec min hour dom month dow year`.
/// Standard 5-field cron `min hour dom month dow` is normalized by:
/// - Prepending `0` (fire at second 0)
/// - Appending `*` (any year)
///
/// 6-field and 7-field expressions are returned unchanged.
pub fn normalize_cron_expression(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => format!("0 {} *", expr), // 5-field → prepend sec, append year
        6 => format!("{} *", expr),   // 6-field → append year
        _ => expr.to_string(),        // 7-field or malformed — pass through
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_5_field() {
        let normalized = normalize_cron_expression("0 9 * * 1");
        assert_eq!(normalized, "0 0 9 * * 1 *");
    }

    #[test]
    fn test_normalize_7_field_unchanged() {
        let expr = "0 0 9 * * 1 *";
        assert_eq!(normalize_cron_expression(expr), expr);
    }
}
