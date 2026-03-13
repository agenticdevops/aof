//! `agentix costs` — view LLM cost tracking data.
//!
//! Commands:
//! - `agentix costs`              — show cost summary table for all agents
//! - `agentix costs agent <name>` — show cost summary + per-run breakdown for one agent

use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

/// `agentix costs` — show cost summary for all agents.
pub async fn costs_all(ctx: &CliContext) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let summaries = client.list_all_costs().await.map_err(|e| {
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
            println!("{}", serde_json::to_string_pretty(&summaries)?);
        }
        OutputFormat::Text => {
            if summaries.is_empty() {
                println!("No cost data recorded yet.");
                println!("Costs are tracked automatically when agents are run via the gateway.");
                return Ok(());
            }

            let col_agent = 24;
            let col_runs = 8;
            let col_input = 14;
            let col_output = 15;
            let col_cost = 12;

            println!(
                "{:<col_agent$}  {:>col_runs$}  {:>col_input$}  {:>col_output$}  {:>col_cost$}  {}",
                "AGENT",
                "RUNS",
                "INPUT TOKENS",
                "OUTPUT TOKENS",
                "COST (USD)",
                "LAST RUN",
                col_agent = col_agent,
                col_runs = col_runs,
                col_input = col_input,
                col_output = col_output,
                col_cost = col_cost,
            );
            println!("{}", "-".repeat(100));

            for s in &summaries {
                let agent = s["agent"].as_str().unwrap_or("-");
                let agent_short = if agent.len() > col_agent {
                    &agent[..col_agent]
                } else {
                    agent
                };
                let runs = s["total_runs"].as_u64().unwrap_or(0);
                let input_tok = s["total_input_tokens"].as_u64().unwrap_or(0);
                let output_tok = s["total_output_tokens"].as_u64().unwrap_or(0);
                let cost = s["total_cost_usd"].as_f64().unwrap_or(0.0);
                let last_run = s["last_run_at"]
                    .as_str()
                    .map(|d| {
                        let d = d.replace('T', " ");
                        if d.len() >= 16 { d[..16].to_string() } else { d }
                    })
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<col_agent$}  {:>col_runs$}  {:>col_input$}  {:>col_output$}  {:>col_cost$}  {}",
                    agent_short,
                    runs,
                    input_tok,
                    output_tok,
                    format!("${:.4}", cost),
                    last_run,
                    col_agent = col_agent,
                    col_runs = col_runs,
                    col_input = col_input,
                    col_output = col_output,
                    col_cost = col_cost,
                );
            }

            let total_cost: f64 = summaries
                .iter()
                .filter_map(|s| s["total_cost_usd"].as_f64())
                .sum();
            println!("{}", "-".repeat(100));
            println!("Total: ${:.4}", total_cost);
        }
    }

    Ok(())
}

/// `agentix costs agent <name>` — show detailed cost breakdown for one agent.
pub async fn costs_agent(ctx: &CliContext, agent: &str, limit: usize) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);

    let summary = client
        .get_agent_cost_summary(agent)
        .await
        .map_err(|e| {
            if let Some(url) = is_gateway_unreachable(&e) {
                anyhow::anyhow!(
                    "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                    url
                )
            } else if e.to_string().contains("Not found") {
                anyhow::anyhow!("No cost data found for agent '{}'", agent)
            } else {
                e
            }
        })?;

    let runs = client
        .list_agent_run_costs(agent, limit)
        .await
        .unwrap_or_default();

    match ctx.output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "summary": summary,
                    "runs": runs,
                }))?
            );
        }
        OutputFormat::Text => {
            // Summary section
            println!("Cost summary for: {}", agent);
            println!("{}", "=".repeat(60));
            println!(
                "  Total runs:          {}",
                summary["total_runs"].as_u64().unwrap_or(0)
            );
            println!(
                "  Total input tokens:  {}",
                summary["total_input_tokens"].as_u64().unwrap_or(0)
            );
            println!(
                "  Total output tokens: {}",
                summary["total_output_tokens"].as_u64().unwrap_or(0)
            );
            println!(
                "  Total cost (USD):    ${:.4}",
                summary["total_cost_usd"].as_f64().unwrap_or(0.0)
            );
            if let Some(last) = summary["last_run_at"].as_str() {
                let last = last.replace('T', " ");
                let last = if last.len() >= 19 { &last[..19] } else { &last };
                println!("  Last run at:         {}", last);
            }

            if runs.is_empty() {
                println!("\nNo per-run breakdown available.");
                return Ok(());
            }

            // Per-run table
            println!("\nRecent runs (last {}):", runs.len());
            println!("{}", "-".repeat(90));
            let col_run = 14;
            let col_model = 28;
            let col_in = 12;
            let col_out = 13;
            println!(
                "{:<col_run$}  {:<col_model$}  {:>col_in$}  {:>col_out$}  {}",
                "RUN ID", "MODEL", "INPUT TOK", "OUTPUT TOK", "COST (USD)",
                col_run = col_run,
                col_model = col_model,
                col_in = col_in,
                col_out = col_out,
            );
            println!("{}", "-".repeat(90));

            for r in &runs {
                let run_id = r["run_id"].as_str().unwrap_or("-");
                let run_short = if run_id.len() > col_run { &run_id[..col_run] } else { run_id };
                let model = r["model"].as_str().unwrap_or("-");
                let model_short = if model.len() > col_model { &model[..col_model] } else { model };
                let input_tok = r["input_tokens"].as_u64().unwrap_or(0);
                let output_tok = r["output_tokens"].as_u64().unwrap_or(0);
                let cost = r["cost_usd"].as_f64().unwrap_or(0.0);

                println!(
                    "{:<col_run$}  {:<col_model$}  {:>col_in$}  {:>col_out$}  ${:.4}",
                    run_short, model_short, input_tok, output_tok, cost,
                    col_run = col_run,
                    col_model = col_model,
                    col_in = col_in,
                    col_out = col_out,
                );
            }
        }
    }

    Ok(())
}
