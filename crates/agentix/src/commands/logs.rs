//! `agentix logs` — show agent execution logs.

use anyhow::Result;
use std::collections::HashMap;

use crate::cli::{CliContext, OutputFormat};
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, agent: String, run_id: Option<String>, follow: bool, trace: bool) -> Result<()> {
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

    // Trace mode: fetch and render trace spans
    if trace {
        let spans = client
            .get_run_trace(&agent, &run_id)
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
                println!("{}", serde_json::to_string_pretty(&spans)?);
            }
            OutputFormat::Text => {
                render_trace_waterfall(&spans, &agent, &run_id);
            }
        }
        return Ok(());
    }

    // Normal log mode
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

// ---------------------------------------------------------------------------
// Trace waterfall renderer
// ---------------------------------------------------------------------------

/// Render a waterfall view of trace spans showing execution hierarchy.
fn render_trace_waterfall(spans: &[serde_json::Value], agent: &str, run_id: &str) {
    if spans.is_empty() {
        println!("No trace spans found for run {}", run_id);
        return;
    }

    // Extract trace_id from the first span
    let trace_id = spans[0]["trace_id"].as_str().unwrap_or("unknown");
    println!();
    println!(
        "\x1b[1mTrace:\x1b[0m {}  \x1b[1mAgent:\x1b[0m {}  \x1b[1mRun:\x1b[0m {}",
        trace_id, agent, run_id
    );
    println!();

    // Build a tree: group spans by parent_span_id
    let mut children_map: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
    let mut roots: Vec<&serde_json::Value> = Vec::new();

    for span in spans {
        match span["parent_span_id"].as_str() {
            Some(parent_id) if !parent_id.is_empty() => {
                children_map
                    .entry(parent_id.to_string())
                    .or_default()
                    .push(span);
            }
            _ => {
                roots.push(span);
            }
        }
    }

    // Render roots, then recurse into children
    for root in &roots {
        render_span(root, 0, &children_map);
    }
    println!();
}

/// Recursively render a span and its children with indentation.
fn render_span(
    span: &serde_json::Value,
    depth: usize,
    children_map: &HashMap<String, Vec<&serde_json::Value>>,
) {
    let indent = "  ".repeat(depth);
    let name = span["name"].as_str().unwrap_or("?");
    let kind = span["kind"].as_str().unwrap_or("");
    let duration_ms = span["duration_ms"].as_i64().unwrap_or(0);
    let status = span["status"].as_str().unwrap_or("ok");

    // Format duration
    let duration_str = if duration_ms >= 1000 {
        format!("{:.1}s", duration_ms as f64 / 1000.0)
    } else {
        format!("{}ms", duration_ms)
    };

    // Status color: green for ok, red for error
    let (status_color, status_reset) = match status {
        "error" => ("\x1b[31m", "\x1b[0m"), // red
        _ => ("\x1b[32m", "\x1b[0m"),        // green
    };

    // Build attribute suffix based on span kind
    let attr_suffix = build_attr_suffix(span, kind);

    // Name in cyan for visual hierarchy
    println!(
        "{}\x1b[36m{}\x1b[0m{}  {}  {}{}{} {}",
        indent,
        name,
        if attr_suffix.is_empty() {
            String::new()
        } else {
            format!("  {}", attr_suffix)
        },
        duration_str,
        status_color,
        status.to_uppercase(),
        status_reset,
        if status == "error" {
            span["status_message"]
                .as_str()
                .map(|m| format!("({})", m))
                .unwrap_or_default()
        } else {
            String::new()
        },
    );

    // Render children
    if let Some(span_id) = span["span_id"].as_str() {
        if let Some(kids) = children_map.get(span_id) {
            for child in kids {
                render_span(child, depth + 1, children_map);
            }
        }
    }
}

/// Build an attribute suffix string based on span kind.
fn build_attr_suffix(span: &serde_json::Value, kind: &str) -> String {
    let attrs = &span["attributes"];
    match kind {
        "llm_call" => {
            let model = attrs["model"].as_str().unwrap_or("");
            let input = attrs["input_tokens"].as_str().unwrap_or("");
            let output = attrs["output_tokens"].as_str().unwrap_or("");
            if !model.is_empty() {
                if !input.is_empty() && !output.is_empty() {
                    format!("{} ({} in / {} out)", model, input, output)
                } else {
                    model.to_string()
                }
            } else {
                String::new()
            }
        }
        "tool_call" => {
            attrs["tool_name"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        "iteration" => {
            attrs["iteration_number"]
                .as_str()
                .map(|n| format!("#{}", n))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Event formatters (regular log mode)
// ---------------------------------------------------------------------------

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
