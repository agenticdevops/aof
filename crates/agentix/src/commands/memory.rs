//! `agentix memory` command handlers.
//!
//! Commands:
//! - `agentix memory list <agent>` — list all stored vector memory entries
//! - `agentix memory clear <agent>` — clear all memory entries (with confirmation)

use anyhow::Result;

use crate::cli::CliContext;
use crate::client::GatewayClient;

/// `agentix memory list <agent>` — list all stored memory entries.
pub async fn memory_list(ctx: &CliContext, agent: &str) -> Result<()> {
    let client = GatewayClient::new(&ctx.gateway_url);
    let entries = client.list_memory(agent).await?;

    if entries.is_empty() {
        println!("No memory entries found for agent: {agent}");
        return Ok(());
    }

    println!(
        "{:<38} {:<12} {:<50} {}",
        "ID", "RUN_ID", "TEXT", "STORED_AT"
    );
    println!("{}", "-".repeat(120));
    for entry in &entries {
        let id = entry["id"].as_str().unwrap_or("-");
        let run_id = entry["run_id"].as_str().unwrap_or("-");
        let text = entry["text"].as_str().unwrap_or("-");
        let stored_at = entry["stored_at"].as_str().unwrap_or("-");
        // Truncate long text snippets for display
        let snippet = if text.len() > 48 { &text[..48] } else { text };
        println!("{id:<38} {run_id:<12} {snippet:<50} {stored_at}");
    }
    println!("\n{} entries", entries.len());
    Ok(())
}

/// `agentix memory clear <agent>` — delete all memory entries (with optional confirmation).
pub async fn memory_clear(ctx: &CliContext, agent: &str, yes: bool) -> Result<()> {
    if !yes {
        print!("Clear all memory for agent '{agent}'? This cannot be undone. [y/N] ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let client = GatewayClient::new(&ctx.gateway_url);
    client.clear_memory(agent).await?;
    println!("Memory cleared for agent: {agent}");
    Ok(())
}
