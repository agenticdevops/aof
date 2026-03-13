//! `agentix channels` — list configured channel routes.
//! `agentix notify` — send a notification to a channel.

use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

/// `agentix channels` — list all configured channel routes.
pub async fn list_channels(ctx: &CliContext) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let channels = client.list_channels().await.map_err(|e| {
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
            println!("{}", serde_json::to_string_pretty(&channels)?);
        }
        OutputFormat::Text => {
            if channels.is_empty() {
                println!("No channels configured.");
                println!("Add channels to your agentix.yaml workspace config.");
                return Ok(());
            }

            let col_platform = 12;
            let col_channel = 20;
            let col_agent = 24;
            let col_direction = 16;

            println!(
                "{:<col_platform$}  {:<col_channel$}  {:<col_agent$}  {:<col_direction$}  {}",
                "PLATFORM", "CHANNEL ID", "AGENT", "DIRECTION", "DESCRIPTION"
            );

            for ch in &channels {
                let platform = ch["platform"].as_str().unwrap_or("-");
                let channel_id = ch["channel_id"].as_str().unwrap_or("-");
                let agent = ch["agent_name"].as_str().unwrap_or("-");
                let direction = ch["direction"].as_str().unwrap_or("-");
                let description = ch["description"].as_str().unwrap_or("");

                println!(
                    "{:<col_platform$}  {:<col_channel$}  {:<col_agent$}  {:<col_direction$}  {}",
                    platform, channel_id, agent, direction, description
                );
            }

            println!("\n{} channel route(s) configured.", channels.len());
        }
    }

    Ok(())
}

/// `agentix notify` — send a notification to a configured channel.
pub async fn send_notify(
    ctx: &CliContext,
    platform: &str,
    channel_id: &str,
    title: &str,
    body: &str,
    severity: &str,
) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let result = client
        .send_notify(platform, channel_id, title, body, severity)
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

    match ctx.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            let status = result["status"].as_str().unwrap_or("unknown");
            if status == "sent" {
                println!("Notification sent to {}:{}", platform, channel_id);
            } else {
                println!("Notification status: {}", status);
            }
        }
    }

    Ok(())
}
