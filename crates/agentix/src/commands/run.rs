/// `agentix run <agent> --input "..."` — fire an agent once from the CLI.
///
/// This creates a TriggerEvent{source: Cli, payload: {input: "..."}} and sends
/// it to the gateway via `POST /api/v1/agents/:name/trigger`. The run is
/// recorded in run history with `trigger_source: cli`.
///
/// Output format:
/// - `text` (default): prints run_id and status
/// - `json`: prints the full gateway response as JSON
use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(
    cli: &CliContext,
    agent: String,
    input: String,
    _format: Option<String>,
) -> Result<()> {
    let client = GatewayClient::new(&cli.gateway_url);

    let response = client
        .trigger_agent(&agent, &input, Some("cli"))
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
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        OutputFormat::Text => {
            let run_id = response["run_id"].as_str().unwrap_or("-");
            let status = response["status"].as_str().unwrap_or("queued");
            println!("Agent:  {}", agent);
            println!("Run ID: {}", run_id);
            println!("Status: {}", status);
            println!();
            println!("Run queued. Track progress with:");
            println!("  agentix runs {}", agent);
        }
    }

    Ok(())
}
