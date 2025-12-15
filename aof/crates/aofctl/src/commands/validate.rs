use anyhow::{Context, Result};
use aof_core::AgentConfig;
use std::fs;
use tracing::info;

/// Validate an agent configuration file
pub async fn execute(file: &str) -> Result<()> {
    info!("Validating agent config: {}", file);

    // Load YAML file
    let config_content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read config file: {}", file))?;

    // Parse as AgentConfig
    let agent_config: AgentConfig = serde_yaml::from_str(&config_content)
        .with_context(|| format!("Failed to parse agent config from: {}", file))?;

    // Validate basic structure
    if agent_config.name.is_empty() {
        anyhow::bail!("Agent name cannot be empty");
    }

    if agent_config.model.is_empty() {
        anyhow::bail!("Agent model cannot be empty");
    }

    if agent_config.max_iterations == 0 {
        anyhow::bail!("max_iterations must be greater than 0");
    }

    if !(0.0..=2.0).contains(&agent_config.temperature) {
        anyhow::bail!("temperature must be between 0.0 and 2.0");
    }

    // Print validation success
    println!(" Configuration is valid");
    println!("\nAgent Details:");
    println!("  Name: {}", agent_config.name);
    println!("  Model: {}", agent_config.model);
    println!("  Max Iterations: {}", agent_config.max_iterations);
    println!("  Temperature: {}", agent_config.temperature);

    if let Some(system_prompt) = &agent_config.system_prompt {
        println!("  System Prompt: {} characters", system_prompt.len());
    }

    if !agent_config.tools.is_empty() {
        println!("  Tools: {}", agent_config.tools.join(", "));
    }

    if let Some(memory) = &agent_config.memory {
        println!("  Memory: {}", memory);
    }

    if let Some(max_tokens) = agent_config.max_tokens {
        println!("  Max Tokens: {}", max_tokens);
    }

    Ok(())
}
