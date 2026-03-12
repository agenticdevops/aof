//! `agentix gateway` subcommands — start and inspect the gateway server.

use std::path::Path;

use anyhow::{anyhow, Result};

use agentix_core::{GatewayConfig, WorkspaceConfig};
use agentix_runtime::gateway::Gateway;

use crate::cli::GatewayCommands;
use crate::client::GatewayClient;

pub async fn run(command: GatewayCommands) -> Result<()> {
    match command {
        GatewayCommands::Start {
            config,
            agents_dir,
            port,
        } => start(config, agents_dir, port).await,
        GatewayCommands::Status => status().await,
    }
}

async fn start(config_path: String, agents_dir_override: Option<String>, port_override: Option<u16>) -> Result<()> {
    // Read and parse workspace config
    let workspace_config: Option<WorkspaceConfig> = if Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read config '{}': {}", config_path, e))?;
        let mut ws = WorkspaceConfig::from_yaml(&content)
            .map_err(|e| anyhow!("Invalid workspace config '{}': {}", config_path, e))?;
        ws.expand_env_vars()
            .map_err(|e| anyhow!("Environment variable error: {}", e))?;
        Some(ws)
    } else {
        if config_path != "./agentix.yaml" {
            // User specified a non-default path that doesn't exist
            return Err(anyhow!("Config file not found: {}", config_path));
        }
        // Default path missing — proceed without workspace config
        None
    };

    // Determine agents directory
    let agents_dir = agents_dir_override
        .or_else(|| {
            workspace_config
                .as_ref()
                .map(|ws| ws.spec.agents_dir.clone())
        })
        .unwrap_or_else(|| "./agents".to_string());

    // Build gateway config from workspace or defaults
    let mut gateway_config: GatewayConfig = workspace_config
        .as_ref()
        .map(|ws| ws.spec.gateway.clone())
        .unwrap_or_default();

    if let Some(p) = port_override {
        gateway_config.port = p;
    }

    println!("Starting OpenAgentiX Gateway...");
    if let Some(ws) = &workspace_config {
        println!("  Workspace: {}", ws.metadata.name);
    }
    println!("  Agents dir: {}", agents_dir);
    println!("  Listen: http://{}:{}", gateway_config.host, gateway_config.port);

    let agents_path = Path::new(&agents_dir);

    Gateway::start(gateway_config, workspace_config, agents_path)
        .await
        .map_err(|e| anyhow!("Gateway error: {}", e))
}

async fn status() -> Result<()> {
    // Use default gateway URL for status — this is independent of --gateway-url global
    let url = std::env::var("AGENTIX_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:7777".to_string());
    let client = GatewayClient::new(&url);

    if client.health().await? {
        println!("Gateway is running at {}", url);
    } else {
        eprintln!("Gateway not reachable at {}", url);
        std::process::exit(1);
    }
    Ok(())
}
