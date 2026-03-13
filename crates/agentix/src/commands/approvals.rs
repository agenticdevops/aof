//! `agentix approve`, `agentix deny`, `agentix approvals` — approval workflow commands.
//!
//! Commands:
//! - `agentix approvals`                           # List pending approvals
//! - `agentix approvals --agent my-agent`           # Filter by agent
//! - `agentix approvals --all`                      # Show all statuses
//! - `agentix approve <request-id>`                 # Approve a pending request
//! - `agentix approve <request-id> --reason "ok"`   # Approve with reason
//! - `agentix deny <request-id>`                    # Deny a pending request
//! - `agentix deny <request-id> --reason "no"`      # Deny with reason

use anyhow::Result;
use colored::Colorize;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

/// Execute the `agentix approvals` command — list approval requests.
pub async fn list(ctx: &CliContext, agent: Option<&str>, all: bool) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let status_filter = if all { None } else { Some("pending") };
    let entries = client.list_approvals(agent, status_filter).await.map_err(|e| {
        if let Some(url) = is_gateway_unreachable(&e) {
            anyhow::anyhow!(
                "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                url
            )
        } else {
            e
        }
    })?;

    match ctx.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Text => {
            if entries.is_empty() {
                if all {
                    println!("No approval requests found.");
                } else {
                    println!("No pending approval requests.");
                }
                return Ok(());
            }

            let col_id = 12;
            let col_agent = 20;
            let col_tool = 24;
            let col_status = 10;
            let col_time = 19;

            println!(
                "{:<col_id$}  {:<col_agent$}  {:<col_tool$}  {:<col_status$}  {}",
                "REQUEST ID", "AGENT", "TOOL", "STATUS", "CREATED",
                col_id = col_id,
                col_agent = col_agent,
                col_tool = col_tool,
                col_status = col_status,
            );
            println!(
                "{}",
                "-".repeat(col_id + col_agent + col_tool + col_status + col_time + 8)
            );

            for entry in &entries {
                let id_raw = entry
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let id_short = if id_raw.len() > col_id {
                    &id_raw[..col_id]
                } else {
                    id_raw
                };

                let agent_name = entry
                    .get("agent_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");

                let tool = entry
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let tool_short = if tool.len() > col_tool {
                    &tool[..col_tool]
                } else {
                    tool
                };

                let status_raw = entry
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let status_display = match status_raw.to_lowercase().as_str() {
                    "pending" => "pending".yellow().to_string(),
                    "approved" => "approved".green().to_string(),
                    "denied" => "denied".red().to_string(),
                    "expired" => "expired".dimmed().to_string(),
                    _ => status_raw.to_string(),
                };

                let created = entry
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .map(|d| {
                        let d = d.replace('T', " ");
                        if d.len() >= 19 {
                            d[..19].to_string()
                        } else {
                            d
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<col_id$}  {:<col_agent$}  {:<col_tool$}  {:<col_status$}  {}",
                    id_short,
                    agent_name,
                    tool_short,
                    status_display,
                    created,
                    col_id = col_id,
                    col_agent = col_agent,
                    col_tool = col_tool,
                    col_status = col_status,
                );
            }

            println!("\n{} request(s) shown.", entries.len());
        }
    }

    Ok(())
}

/// Execute the `agentix approve <request-id>` command.
pub async fn approve(ctx: &CliContext, request_id: &str, reason: Option<&str>) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let result = client.approve_request(request_id, reason).await.map_err(|e| {
        if let Some(url) = is_gateway_unreachable(&e) {
            anyhow::anyhow!(
                "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                url
            )
        } else {
            e
        }
    })?;

    match ctx.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            let status = result
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("approved");
            println!(
                "{} Request {} {}",
                "Approved.".green().bold(),
                request_id,
                format!("(status: {})", status).dimmed()
            );
        }
    }

    Ok(())
}

/// Execute the `agentix deny <request-id>` command.
pub async fn deny(ctx: &CliContext, request_id: &str, reason: Option<&str>) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let result = client.deny_request(request_id, reason).await.map_err(|e| {
        if let Some(url) = is_gateway_unreachable(&e) {
            anyhow::anyhow!(
                "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                url
            )
        } else {
            e
        }
    })?;

    match ctx.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            let status = result
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("denied");
            println!(
                "{} Request {} {}",
                "Denied.".red().bold(),
                request_id,
                format!("(status: {})", status).dimmed()
            );
        }
    }

    Ok(())
}
