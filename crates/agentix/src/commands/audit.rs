//! `agentix audit <agent-name>` — view agent audit trail.
//!
//! Commands:
//! - `agentix audit <agent-name>`                # Show last 50 audit entries
//! - `agentix audit <agent-name> --limit 100`    # Show last 100 entries
//! - `agentix audit <agent-name> --security`     # Security events only
//! - `agentix audit <agent-name> --run <run-id>` # Filter to specific run

use anyhow::Result;
use colored::Colorize;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

/// Execute the `agentix audit` command.
pub async fn run(ctx: &CliContext, agent: &str, limit: usize, security: bool, run_id: Option<&str>) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let entries: Vec<serde_json::Value> = if security {
        // GET /api/v1/audit/security?limit=N
        client.get_security_audit(limit).await.map_err(|e| {
            if let Some(url) = is_gateway_unreachable(&e) {
                anyhow::anyhow!(
                    "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                    url
                )
            } else {
                e
            }
        })?
    } else {
        // GET /api/v1/agents/:name/audit?limit=N
        client.get_agent_audit(agent, limit).await.map_err(|e| {
            if let Some(url) = is_gateway_unreachable(&e) {
                anyhow::anyhow!(
                    "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                    url
                )
            } else {
                e
            }
        })?
    };

    // Filter by run_id if provided
    let entries: Vec<&serde_json::Value> = if let Some(rid) = run_id {
        entries.iter().filter(|e| {
            e.get("run_id")
                .and_then(|v| v.as_str())
                .map(|r| r == rid)
                .unwrap_or(false)
        }).collect()
    } else {
        entries.iter().collect()
    };

    match ctx.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Text => {
            if entries.is_empty() {
                if security {
                    println!("No security events recorded.");
                } else {
                    println!("No audit entries for agent '{}'.", agent);
                }
                println!("Audit entries are recorded automatically when agents run via the gateway.");
                return Ok(());
            }

            // Table header
            let col_time = 19;
            let col_type = 20;
            let col_action = 44;
            let col_outcome = 12;

            println!(
                "{:<col_time$}  {:<col_type$}  {:<col_action$}  {}",
                "TIMESTAMP", "EVENT TYPE", "ACTION", "OUTCOME",
                col_time = col_time, col_type = col_type, col_action = col_action,
            );
            println!("{}", "-".repeat(col_time + col_type + col_action + col_outcome + 6));

            for entry in &entries {
                let timestamp = entry.get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(|d| {
                        let d = d.replace('T', " ");
                        if d.len() >= 19 { d[..19].to_string() } else { d }
                    })
                    .unwrap_or_else(|| "-".to_string());

                let event_type = entry.get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");

                let action = entry.get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let action_short = if action.len() > col_action {
                    &action[..col_action]
                } else {
                    action
                };

                let outcome_raw = entry.get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");

                let outcome_display = match outcome_raw.to_lowercase().as_str() {
                    "success" => "success".green().to_string(),
                    s if s.starts_with("failure") => outcome_raw.red().to_string(),
                    s if s.starts_with("blocked") => outcome_raw.red().to_string(),
                    s if s.starts_with("denied") => outcome_raw.yellow().to_string(),
                    _ => outcome_raw.to_string(),
                };

                println!(
                    "{:<col_time$}  {:<col_type$}  {:<col_action$}  {}",
                    timestamp, event_type, action_short, outcome_display,
                    col_time = col_time, col_type = col_type, col_action = col_action,
                );
            }

            println!("\n{} entries shown.", entries.len());
        }
    }

    Ok(())
}
