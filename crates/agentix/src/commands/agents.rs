//! `agentix agents` — list agents registered in the running gateway.

use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, namespace: Option<String>) -> Result<()> {
    let client = GatewayClient::new(&cli.gateway_url);

    let agents = client.list_agents().await.map_err(|e| {
        if let Some(url) = is_gateway_unreachable(&e) {
            anyhow::anyhow!(
                "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                url
            )
        } else {
            e
        }
    })?;

    // Filter by namespace if provided (local filter on the "namespace" field if present)
    let agents: Vec<_> = if let Some(ref ns) = namespace {
        agents
            .into_iter()
            .filter(|a| a["namespace"].as_str() == Some(ns.as_str()))
            .collect()
    } else {
        agents
    };

    match cli.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&agents)?);
        }
        OutputFormat::Text => {
            if agents.is_empty() {
                println!("No agents registered. Use `agentix apply -f agent.yaml` to register one.");
                return Ok(());
            }

            // Column widths
            let col_name = 20;
            let col_desc = 35;
            let col_model = 32;
            let _col_status = 10;

            println!(
                "{:<col_name$}  {:<col_desc$}  {:<col_model$}  {}",
                "NAME", "DESCRIPTION", "MODEL", "STATUS",
                col_name = col_name,
                col_desc = col_desc,
                col_model = col_model,
            );

            for agent in &agents {
                let name = agent["name"].as_str().unwrap_or("-");
                let desc = agent["description"].as_str().unwrap_or("-");
                let model = agent["model"].as_str().unwrap_or("-");
                let status = agent["status"].as_str().unwrap_or("unknown");

                // Truncate if too long
                let desc = if desc.len() > col_desc { &desc[..col_desc - 1] } else { desc };
                let model = if model.len() > col_model { &model[..col_model - 1] } else { model };

                println!(
                    "{:<col_name$}  {:<col_desc$}  {:<col_model$}  {}",
                    name, desc, model, status,
                    col_name = col_name,
                    col_desc = col_desc,
                    col_model = col_model,
                );
            }
        }
    }

    Ok(())
}
