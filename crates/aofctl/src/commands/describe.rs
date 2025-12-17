//! Describe resources in detail (kubectl describe compatible)

use anyhow::{Context, Result};
use aof_core::workflow::NextStep;
use aof_core::AgentFlow;
use std::fs;
use std::path::Path;

use crate::resources::ResourceType;

/// Describe a resource in detail
pub async fn execute(resource_type: &str, name: &str) -> Result<()> {
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    match rt {
        ResourceType::Fleet => describe_fleet(name).await,
        ResourceType::Flow | ResourceType::Workflow => describe_flow(name).await,
        ResourceType::Agent => describe_agent(name).await,
        _ => {
            println!("Describe for {} - detailed view not yet implemented", resource_type);
            println!("Resource type: {}", resource_type);
            println!("Name: {}", name);
            Ok(())
        }
    }
}

/// Describe fleet in detail
async fn describe_fleet(name: &str) -> Result<()> {
    use aof_core::AgentFleet;

    // Check if name is a file path or registry lookup
    let fleet: AgentFleet = if Path::new(name).exists() {
        let content = fs::read_to_string(name)
            .with_context(|| format!("Failed to read fleet config: {}", name))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse fleet config: {}", name))?
    } else {
        anyhow::bail!(
            "Fleet '{}' not found. Provide a config file path or use 'aofctl get fleets' to list.",
            name
        )
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
        println!(
            "  - {} (x{}, role: {})",
            agent.name, agent.replicas, role_str
        );

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

/// Describe flow/workflow in detail
/// Auto-detects between AgentFlow (trigger/nodes) and Workflow (entrypoint/steps)
async fn describe_flow(name: &str) -> Result<()> {
    use aof_core::workflow::Workflow;

    // Check if name is a file path
    if !Path::new(name).exists() {
        anyhow::bail!(
            "Flow '{}' not found. Provide a config file path or use 'aofctl get flows' to list.",
            name
        )
    }

    let content = fs::read_to_string(name)
        .with_context(|| format!("Failed to read flow config: {}", name))?;

    // Try to detect the kind from the YAML content
    let kind = detect_yaml_kind(&content);

    match kind.as_deref() {
        Some("AgentFlow") => describe_agentflow(&content, name).await,
        Some("Workflow") | _ => describe_workflow(&content, name).await,
    }
}

/// Detect the 'kind' field from a YAML document
fn detect_yaml_kind(content: &str) -> Option<String> {
    // Simple parsing: find 'kind:' line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("kind:") {
            let value = trimmed.strip_prefix("kind:")?.trim();
            // Remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'');
            return Some(value.to_string());
        }
    }
    None
}

/// Describe AgentFlow resource (trigger-based event-driven flow)
async fn describe_agentflow(content: &str, name: &str) -> Result<()> {
    let agentflow: AgentFlow = serde_yaml::from_str(content)
        .with_context(|| format!("Failed to parse AgentFlow config: {}", name))?;

    println!("Name:         {}", agentflow.metadata.name);
    println!("API Version:  {}", agentflow.api_version);
    println!("Kind:         {}", agentflow.kind);

    if let Some(ns) = &agentflow.metadata.namespace {
        println!("Namespace:    {}", ns);
    }

    if !agentflow.metadata.labels.is_empty() {
        println!("Labels:");
        for (k, v) in &agentflow.metadata.labels {
            println!("  {}: {}", k, v);
        }
    }

    // Trigger info
    println!("\nTrigger:");
    println!("  Type:   {:?}", agentflow.spec.trigger.trigger_type);
    let config = &agentflow.spec.trigger.config;
    if !config.events.is_empty() {
        println!("  Events: {}", config.events.join(", "));
    }

    // Additional triggers
    if !agentflow.spec.triggers.is_empty() {
        println!("\nAdditional Triggers ({}):", agentflow.spec.triggers.len());
        for trigger in &agentflow.spec.triggers {
            println!("  - {:?}", trigger.trigger_type);
        }
    }

    // Nodes
    println!("\nNodes ({}):", agentflow.spec.nodes.len());
    for node in &agentflow.spec.nodes {
        println!("  - {}", node.id);
        println!("    Type: {:?}", node.node_type);
        let node_config = &node.config;
        if let Some(agent) = &node_config.agent {
            println!("    Agent: {}", agent);
        }
        if let Some(channel) = &node_config.channel {
            println!("    Channel: {}", channel);
        }
        if let Some(timeout) = &node_config.timeout_seconds {
            println!("    Timeout: {}s", timeout);
        }
    }

    // Connections
    println!("\nConnections ({}):", agentflow.spec.connections.len());
    for conn in &agentflow.spec.connections {
        let condition = conn.when.as_ref().map(|w| format!(" when '{}'", w)).unwrap_or_default();
        println!("  {} -> {}{}", conn.from, conn.to, condition);
    }

    // Config
    if let Some(flow_config) = &agentflow.spec.config {
        println!("\nConfig:");
        if let Some(timeout) = flow_config.default_timeout_seconds {
            println!("  Default Timeout: {}s", timeout);
        }
        if flow_config.verbose {
            println!("  Verbose: true");
        }
        if let Some(retry) = &flow_config.retry {
            println!("  Retry:");
            println!("    Max Attempts: {}", retry.max_attempts);
        }
    }

    Ok(())
}

/// Describe Workflow resource (step-based sequential flow)
async fn describe_workflow(content: &str, name: &str) -> Result<()> {
    use aof_core::workflow::Workflow;

    let workflow: Workflow = serde_yaml::from_str(content)
        .with_context(|| format!("Failed to parse Workflow config: {}", name))?;

    println!("Name:         {}", workflow.metadata.name);
    println!("API Version:  {}", workflow.api_version);
    println!("Kind:         {}", workflow.kind);

    if let Some(ns) = &workflow.metadata.namespace {
        println!("Namespace:    {}", ns);
    }

    if !workflow.metadata.labels.is_empty() {
        println!("Labels:");
        for (k, v) in &workflow.metadata.labels {
            println!("  {}: {}", k, v);
        }
    }

    println!("\nEntrypoint:   {}", workflow.spec.entrypoint);

    println!("\nSteps ({}):", workflow.spec.steps.len());
    for step in &workflow.spec.steps {
        println!("  - {}", step.name);
        println!("    Type:   {:?}", step.step_type);
        if let Some(agent) = &step.agent {
            println!("    Agent:  {}", agent);
        }
        if let Some(next) = &step.next {
            let next_targets = get_next_targets(next);
            if !next_targets.is_empty() {
                println!("    Next:   {}", next_targets.join(", "));
            }
        }
        if let Some(timeout) = &step.timeout {
            println!("    Timeout: {}", timeout);
        }
    }

    if let Some(error_handler) = &workflow.spec.error_handler {
        println!("\nError Handler: {}", error_handler);
    }

    if let Some(retry) = &workflow.spec.retry {
        println!("\nRetry Config:");
        println!("  Max Attempts: {}", retry.max_attempts);
    }

    Ok(())
}

/// Describe agent in detail
async fn describe_agent(name: &str) -> Result<()> {
    use aof_core::AgentConfig;

    // Check if name is a file path
    let agent: AgentConfig = if Path::new(name).exists() {
        let content = fs::read_to_string(name)
            .with_context(|| format!("Failed to read agent config: {}", name))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse agent config: {}", name))?
    } else {
        anyhow::bail!(
            "Agent '{}' not found. Provide a config file path or use 'aofctl get agents' to list.",
            name
        )
    };

    println!("Name:         {}", agent.name);
    println!("Model:        {}", agent.model);

    if let Some(provider) = &agent.provider {
        println!("Provider:     {}", provider);
    }

    if let Some(system_prompt) = &agent.system_prompt {
        println!("\nSystem Prompt:");
        for line in system_prompt.lines() {
            println!("  {}", line);
        }
    }

    if !agent.tools.is_empty() {
        println!("\nTools ({}):", agent.tools.len());
        for tool in &agent.tools {
            println!("  - {:?}", tool);
        }
    }

    if !agent.mcp_servers.is_empty() {
        println!("\nMCP Servers ({}):", agent.mcp_servers.len());
        for server in &agent.mcp_servers {
            let cmd = server.command.as_deref().unwrap_or("N/A");
            println!("  - {} ({:?}: {})", server.name, server.transport, cmd);
        }
    }

    Ok(())
}

/// Helper to extract next step targets
fn get_next_targets(next: &NextStep) -> Vec<String> {
    match next {
        NextStep::Simple(target) => vec![target.clone()],
        NextStep::Conditional(conditions) => conditions.iter().map(|c| c.target.clone()).collect(),
    }
}
