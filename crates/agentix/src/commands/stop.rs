//! `agentix stop` — stop a running agent.

use anyhow::Result;

use crate::cli::CliContext;
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, agent: String, run_id: Option<String>) -> Result<()> {
    let client = GatewayClient::new(&cli.gateway_url);

    // Resolve run_id: use provided, or find the active run
    let run_id = match run_id {
        Some(id) => id,
        None => {
            let runs = client
                .list_runs(Some(&agent), 20)
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

            // Find the first running run
            runs.iter()
                .find(|r| r["status"].as_str() == Some("running"))
                .and_then(|r| r["id"].as_str().map(|s| s.to_string()))
                .ok_or_else(|| {
                    anyhow::anyhow!("No active run found for agent '{}'. Use --run <id> to stop a specific run.", agent)
                })?
        }
    };

    client
        .stop_run(&agent, &run_id)
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

    println!("Stopped run {} for agent {}", run_id, agent);
    Ok(())
}
