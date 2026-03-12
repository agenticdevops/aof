//! `agentix runs` — show agent run history.

use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, agent: Option<String>, limit: usize) -> Result<()> {
    let client = GatewayClient::new(&cli.gateway_url);

    let runs = client
        .list_runs(agent.as_deref(), limit)
        .await
        .map_err(|e| {
            if let Some(url) = is_gateway_unreachable(&e) {
                anyhow::anyhow!(
                    "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                    url
                )
            } else {
                e
            }
        })?;

    match cli.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&runs)?);
        }
        OutputFormat::Text => {
            if runs.is_empty() {
                println!("No runs found.");
                return Ok(());
            }

            let col_id = 12;
            let col_agent = 20;
            let col_trigger = 10;
            let col_status = 12;
            let col_started = 20;

            println!(
                "{:<col_id$}  {:<col_agent$}  {:<col_trigger$}  {:<col_status$}  {:<col_started$}  {}",
                "RUN ID", "AGENT", "TRIGGER", "STATUS", "STARTED", "ITERATIONS",
                col_id = col_id,
                col_agent = col_agent,
                col_trigger = col_trigger,
                col_status = col_status,
                col_started = col_started,
            );

            for run in &runs {
                let id = run["id"].as_str().unwrap_or("-");
                let id_short = if id.len() > col_id { &id[..col_id] } else { id };

                let agent_name = run["agent"]
                    .as_str()
                    .or_else(|| run["agent_name"].as_str())
                    .unwrap_or("-");

                let trigger = run["trigger_source"]
                    .as_str()
                    .unwrap_or("-");

                let status = run["status"].as_str().unwrap_or("unknown");

                let started = run["started_at"]
                    .as_str()
                    .map(|s| {
                        // Format datetime: take first 16 chars "2026-03-12T09:15"
                        let s = s.replace('T', " ");
                        if s.len() >= 16 { s[..16].to_string() } else { s }
                    })
                    .unwrap_or_else(|| "-".to_string());

                let iters = run["iterations"]
                    .as_u64()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<col_id$}  {:<col_agent$}  {:<col_trigger$}  {:<col_status$}  {:<col_started$}  {}",
                    id_short, agent_name, trigger, status, started, iters,
                    col_id = col_id,
                    col_agent = col_agent,
                    col_trigger = col_trigger,
                    col_status = col_status,
                    col_started = col_started,
                );
            }
        }
    }

    Ok(())
}
