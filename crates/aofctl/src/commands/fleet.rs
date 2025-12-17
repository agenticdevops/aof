//! Fleet CLI commands for multi-agent coordination
//!
//! Commands:
//! - aofctl fleet apply -f fleet.yaml    - Load fleet configuration
//! - aofctl fleet get [name]             - List/get fleets
//! - aofctl fleet describe <name>        - Show fleet details
//! - aofctl fleet status <name>          - Show runtime status
//! - aofctl fleet run <name> -i "query"  - Execute task on fleet
//! - aofctl fleet scale <name> --replicas N - Scale agent replicas
//! - aofctl fleet delete <name>          - Remove fleet

use anyhow::{Context, Result};
use aof_core::AgentFleet;
use aof_runtime::fleet::FleetCoordinator;
use clap::Subcommand;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tracing::info;

/// Fleet subcommands
///
/// Note: To run a fleet, use `aofctl run fleet <config>` instead.
/// The fleet subcommand is for management operations (apply, get, describe, etc.)
#[derive(Subcommand, Debug)]
pub enum FleetCommands {
    /// Apply fleet configuration from file
    Apply {
        /// Configuration file (YAML)
        #[arg(short, long)]
        file: String,
    },

    /// List or get fleet(s)
    Get {
        /// Fleet name (optional - lists all if omitted)
        name: Option<String>,

        /// Output format (json, yaml, wide)
        #[arg(short, long, default_value = "wide")]
        output: String,
    },

    /// Describe fleet in detail
    Describe {
        /// Fleet name or config file
        name: String,
    },

    /// Show fleet runtime status
    Status {
        /// Fleet name or config file
        name: String,
    },

    /// Scale fleet agent replicas
    Scale {
        /// Fleet name or config file
        name: String,

        /// Number of replicas
        #[arg(long)]
        replicas: u32,

        /// Specific agent to scale (optional - scales all if omitted)
        #[arg(long)]
        agent: Option<String>,
    },

    /// Delete/stop a fleet
    Delete {
        /// Fleet name
        name: String,
    },
}

/// Registry of loaded fleets (in-memory for now)
static FLEET_REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Execute fleet subcommand
pub async fn execute(cmd: FleetCommands) -> Result<()> {
    match cmd {
        FleetCommands::Apply { file } => apply_fleet(&file).await,
        FleetCommands::Get { name, output } => get_fleets(name.as_deref(), &output).await,
        FleetCommands::Describe { name } => describe_fleet(&name).await,
        FleetCommands::Status { name } => status_fleet(&name).await,
        FleetCommands::Scale {
            name,
            replicas,
            agent,
        } => scale_fleet(&name, replicas, agent.as_deref()).await,
        FleetCommands::Delete { name } => delete_fleet(&name).await,
    }
}

/// Apply fleet configuration from file
async fn apply_fleet(file: &str) -> Result<()> {
    info!("Applying fleet configuration from: {}", file);

    // Read and parse fleet config
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read fleet config: {}", file))?;

    let fleet: AgentFleet = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse fleet config: {}", file))?;

    // Validate fleet configuration
    fleet.validate().context("Fleet validation failed")?;

    let fleet_name = fleet.metadata.name.clone();
    let agent_count = fleet.spec.agents.len();
    let total_replicas: u32 = fleet.spec.agents.iter().map(|a| a.replicas).sum();

    // Register in our simple registry
    {
        let mut registry = FLEET_REGISTRY.lock().unwrap();
        registry.insert(fleet_name.clone(), file.to_string());
    }

    println!("fleet.aof.dev/{} configured", fleet_name);
    println!(
        "  Agents: {} ({} total replicas)",
        agent_count, total_replicas
    );
    println!("  Coordination: {:?}", fleet.spec.coordination.mode);

    Ok(())
}

/// List or get fleet(s)
async fn get_fleets(name: Option<&str>, output: &str) -> Result<()> {
    let registry = FLEET_REGISTRY.lock().unwrap();

    if let Some(fleet_name) = name {
        // Get specific fleet
        if let Some(file_path) = registry.get(fleet_name) {
            let content = fs::read_to_string(file_path)?;
            let fleet: AgentFleet = serde_yaml::from_str(&content)?;

            match output {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&fleet)?);
                }
                "yaml" => {
                    println!("{}", serde_yaml::to_string(&fleet)?);
                }
                _ => {
                    print_fleet_wide(&fleet);
                }
            }
        } else {
            println!("Fleet '{}' not found", fleet_name);
        }
    } else {
        // List all fleets
        if registry.is_empty() {
            println!("No fleets configured. Use 'aofctl fleet apply -f <file>' to add one.");
            return Ok(());
        }

        match output {
            "json" => {
                let mut fleets = Vec::new();
                for (_, file_path) in registry.iter() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        if let Ok(fleet) = serde_yaml::from_str::<AgentFleet>(&content) {
                            fleets.push(fleet);
                        }
                    }
                }
                println!("{}", serde_json::to_string_pretty(&fleets)?);
            }
            _ => {
                println!(
                    "{:<20} {:<15} {:<10} {:<15}",
                    "NAME", "COORDINATION", "AGENTS", "REPLICAS"
                );
                for (_, file_path) in registry.iter() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        if let Ok(fleet) = serde_yaml::from_str::<AgentFleet>(&content) {
                            let total_replicas: u32 =
                                fleet.spec.agents.iter().map(|a| a.replicas).sum();
                            println!(
                                "{:<20} {:<15} {:<10} {:<15}",
                                fleet.metadata.name,
                                format!("{:?}", fleet.spec.coordination.mode),
                                fleet.spec.agents.len(),
                                total_replicas
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Describe fleet in detail
async fn describe_fleet(name: &str) -> Result<()> {
    // Check if name is a file path
    let fleet = if Path::new(name).exists() {
        let content = fs::read_to_string(name)?;
        serde_yaml::from_str::<AgentFleet>(&content)?
    } else {
        // Look up in registry
        let registry = FLEET_REGISTRY.lock().unwrap();
        let file_path = registry
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Fleet '{}' not found", name))?;
        let content = fs::read_to_string(file_path)?;
        serde_yaml::from_str::<AgentFleet>(&content)?
    };

    println!("Name:         {}", fleet.metadata.name);
    println!("API Version:  {}", fleet.api_version);
    println!("Kind:         {}", fleet.kind);

    if !fleet.metadata.labels.is_empty() {
        println!("Labels:");
        for (k, v) in &fleet.metadata.labels {
            println!("  {}: {}", k, v);
        }
    }

    println!("\nCoordination:");
    println!("  Mode:         {:?}", fleet.spec.coordination.mode);
    println!(
        "  Distribution: {:?}",
        fleet.spec.coordination.distribution
    );

    if let Some(consensus) = &fleet.spec.coordination.consensus {
        println!("  Consensus:");
        if let Some(min_votes) = consensus.min_votes {
            println!("    Min Votes: {}", min_votes);
        }
        if let Some(timeout_ms) = consensus.timeout_ms {
            println!("    Timeout:   {}ms", timeout_ms);
        }
    }

    println!("\nAgents:");
    for agent in &fleet.spec.agents {
        let role_str = format!("{:?}", agent.role);
        println!("  - {} (x{}, role: {})", agent.name, agent.replicas, role_str);

        if let Some(spec) = &agent.spec {
            println!("    Model: {}", spec.model);
            if !spec.tools.is_empty() {
                println!("    Tools: {:?}", spec.tools);
            }
        }
        if let Some(config) = &agent.config {
            println!("    Config: {}", config);
        }
    }

    Ok(())
}

/// Show fleet runtime status
async fn status_fleet(name: &str) -> Result<()> {
    // Check if name is a file path
    let file_path = if Path::new(name).exists() {
        name.to_string()
    } else {
        // Look up in registry
        let registry = FLEET_REGISTRY.lock().unwrap();
        registry
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Fleet '{}' not found", name))?
    };

    // Create coordinator to get status
    let coordinator = FleetCoordinator::from_file(&file_path)
        .await
        .context("Failed to load fleet")?;

    let state = coordinator.state().await;
    let metrics = coordinator.metrics().await;

    println!("Fleet: {}", state.fleet_name);
    println!("Status: {:?}", state.status);
    println!();
    println!("Metrics:");
    println!("  Total Agents:    {}", metrics.total_agents);
    println!("  Active Agents:   {}", metrics.active_agents);
    println!("  Total Tasks:     {}", metrics.total_tasks);
    println!("  Completed Tasks: {}", metrics.completed_tasks);
    println!("  Failed Tasks:    {}", metrics.failed_tasks);
    println!("  Avg Duration:    {:.1}ms", metrics.avg_task_duration_ms);

    if !state.agents.is_empty() {
        println!();
        println!("Agent Instances:");
        println!(
            "{:<25} {:<15} {:<10} {:<15}",
            "INSTANCE", "STATUS", "TASKS", "LAST ACTIVITY"
        );
        for (instance_id, agent_state) in &state.agents {
            let last_activity = agent_state
                .last_activity
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "-".to_string());
            println!(
                "{:<25} {:<15} {:<10} {:<15}",
                instance_id,
                format!("{:?}", agent_state.status),
                agent_state.tasks_processed,
                last_activity
            );
        }
    }

    Ok(())
}

/// Scale fleet agent replicas
async fn scale_fleet(name: &str, replicas: u32, agent: Option<&str>) -> Result<()> {
    // Check if name is a file path
    let file_path = if Path::new(name).exists() {
        name.to_string()
    } else {
        // Look up in registry
        let registry = FLEET_REGISTRY.lock().unwrap();
        registry
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Fleet '{}' not found", name))?
    };

    // Read current config
    let content = fs::read_to_string(&file_path)?;
    let mut fleet: AgentFleet = serde_yaml::from_str(&content)?;

    if let Some(agent_name) = agent {
        // Scale specific agent
        let mut found = false;
        for fleet_agent in &mut fleet.spec.agents {
            if fleet_agent.name == agent_name {
                let old_replicas = fleet_agent.replicas;
                fleet_agent.replicas = replicas;
                println!(
                    "Scaled agent '{}' from {} to {} replicas",
                    agent_name, old_replicas, replicas
                );
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!("Agent '{}' not found in fleet", agent_name);
        }
    } else {
        // Scale all agents
        for fleet_agent in &mut fleet.spec.agents {
            let old_replicas = fleet_agent.replicas;
            fleet_agent.replicas = replicas;
            println!(
                "Scaled agent '{}' from {} to {} replicas",
                fleet_agent.name, old_replicas, replicas
            );
        }
    }

    // Write back
    let new_content = serde_yaml::to_string(&fleet)?;
    fs::write(&file_path, new_content)?;

    println!("Fleet configuration updated: {}", file_path);
    println!("Note: Restart the fleet to apply changes.");

    Ok(())
}

/// Delete/stop a fleet
async fn delete_fleet(name: &str) -> Result<()> {
    let mut registry = FLEET_REGISTRY.lock().unwrap();

    if registry.remove(name).is_some() {
        println!("fleet.aof.dev/{} deleted", name);
    } else {
        println!("Fleet '{}' not found in registry", name);
    }

    Ok(())
}

/// Print fleet in wide format
fn print_fleet_wide(fleet: &AgentFleet) {
    let total_replicas: u32 = fleet.spec.agents.iter().map(|a| a.replicas).sum();
    let agent_names: Vec<&str> = fleet.spec.agents.iter().map(|a| a.name.as_str()).collect();

    println!(
        "{:<20} {:<15} {:<10} {:<10} {}",
        "NAME", "COORDINATION", "AGENTS", "REPLICAS", "AGENT NAMES"
    );
    println!(
        "{:<20} {:<15} {:<10} {:<10} {}",
        fleet.metadata.name,
        format!("{:?}", fleet.spec.coordination.mode),
        fleet.spec.agents.len(),
        total_replicas,
        agent_names.join(", ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_registry() {
        let mut registry = FLEET_REGISTRY.lock().unwrap();
        registry.insert("test-fleet".to_string(), "/tmp/test.yaml".to_string());
        assert!(registry.contains_key("test-fleet"));
        registry.remove("test-fleet");
    }
}
