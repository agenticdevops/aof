//! `agentix logs` — show agent execution logs.

use anyhow::Result;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, agent: String, run_id: Option<String>, follow: bool) -> Result<()> {
    let client = GatewayClient::new(&cli.gateway_url);

    // Resolve run_id: use provided, or pick latest from run list
    let run_id = match run_id {
        Some(id) => id,
        None => {
            let runs = client
                .list_runs(Some(&agent), 1)
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
            runs.first()
                .and_then(|r| r["id"].as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("No runs found for agent '{}'", agent))?
        }
    };

    // Fetch stored logs
    let events = client
        .get_run_logs(&agent, &run_id)
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
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        OutputFormat::Text => {
            for event in &events {
                format_event(event, cli.quiet);
            }
        }
    }

    // Follow mode: simple poll loop for new events
    if follow {
        use std::time::Duration;
        let mut last_count = events.len();
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Ok(new_events) = client.get_run_logs(&agent, &run_id).await {
                for event in new_events.iter().skip(last_count) {
                    match cli.output {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(event)?);
                        }
                        OutputFormat::Text => {
                            format_event(event, cli.quiet);
                        }
                    }
                }
                let new_count = new_events.len();
                // Check if run is complete
                if new_count > 0 {
                    if let Some(status) = new_events.last().and_then(|e| e["type"].as_str()) {
                        if status == "Complete" || status == "Error" {
                            last_count = new_count;
                            break;
                        }
                    }
                }
                last_count = new_count;
            }
        }
    }

    Ok(())
}

/// Format a single ReAct event for text output.
fn format_event(event: &serde_json::Value, quiet: bool) {
    let event_type = event["type"].as_str().unwrap_or("Unknown");

    match event_type {
        "Complete" => {
            if let Some(output) = event["output"].as_str() {
                println!("{}", output);
            }
        }
        "Error" => {
            let msg = event["message"].as_str().unwrap_or("unknown error");
            eprintln!("error: {}", msg);
        }
        "Think" if !quiet => {
            if let Some(thought) = event["thought"].as_str() {
                println!("[think] {}", thought);
            }
        }
        "Act" if !quiet => {
            let tool = event["tool"].as_str().unwrap_or("?");
            println!("[act] {}({})", tool, summarize_args(&event["args"]));
        }
        "Observe" if !quiet => {
            if let Some(obs) = event["observation"].as_str() {
                let preview = if obs.len() > 80 { &obs[..80] } else { obs };
                println!("[observe] {}", preview);
            }
        }
        "Start" if !quiet => {
            println!("[start] run {}", event["run_id"].as_str().unwrap_or("?"));
        }
        _ => {
            if !quiet {
                println!("[{}]", event_type);
            }
        }
    }
}

fn summarize_args(args: &serde_json::Value) -> String {
    match args {
        serde_json::Value::Object(map) => {
            let parts: Vec<String> = map
                .iter()
                .take(2)
                .map(|(k, v)| format!("{k}={}", short_str(v)))
                .collect();
            parts.join(", ")
        }
        _ => String::new(),
    }
}

fn short_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            if s.len() > 20 {
                format!("\"{}...\"", &s[..20])
            } else {
                format!("\"{s}\"")
            }
        }
        other => {
            let s = other.to_string();
            if s.len() > 20 { format!("{}...", &s[..20]) } else { s }
        }
    }
}
