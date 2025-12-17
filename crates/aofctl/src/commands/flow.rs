//! Flow CLI commands for workflow orchestration
//!
//! Commands:
//! - aofctl flow apply -f flow.yaml       - Load flow configuration
//! - aofctl flow get [name]               - List/get flows
//! - aofctl flow describe <name>          - Show flow details
//! - aofctl flow run <name> [--input]     - Execute flow
//! - aofctl flow status <run-id>          - Get execution status
//! - aofctl flow logs <run-id>            - View execution logs
//! - aofctl flow visualize <name>         - ASCII graph output
//! - aofctl flow delete <name>            - Remove flow

use anyhow::{Context, Result};
use aof_core::{NextStep, Workflow};
use aof_runtime::executor::{Runtime, WorkflowEvent, WorkflowExecutor};
use clap::Subcommand;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::info;

/// Flow subcommands
#[derive(Subcommand, Debug)]
pub enum FlowCommands {
    /// Apply flow configuration from file
    Apply {
        /// Configuration file (YAML)
        #[arg(short, long)]
        file: String,
    },

    /// List or get flow(s)
    Get {
        /// Flow name (optional - lists all if omitted)
        name: Option<String>,

        /// Output format (json, yaml, wide)
        #[arg(short, long, default_value = "wide")]
        output: String,
    },

    /// Describe flow in detail
    Describe {
        /// Flow name or config file
        name: String,
    },

    /// Execute a flow
    Run {
        /// Flow name or config file
        name: String,

        /// Input data (JSON)
        #[arg(short, long)]
        input: Option<String>,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        output: String,

        /// Run as daemon (long-running service with triggers)
        #[arg(long)]
        daemon: bool,

        /// Port for webhook triggers (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,
    },

    /// Get execution status
    Status {
        /// Run ID
        run_id: String,
    },

    /// View execution logs
    Logs {
        /// Run ID
        run_id: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Visualize flow as ASCII graph
    Visualize {
        /// Flow name or config file
        name: String,

        /// Output format (ascii, dot)
        #[arg(short, long, default_value = "ascii")]
        format: String,
    },

    /// Delete/remove a flow
    Delete {
        /// Flow name
        name: String,
    },
}

/// Registry of loaded flows (in-memory for now)
static FLOW_REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Registry of run states (in-memory for now)
static RUN_REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, RunState>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
struct RunState {
    workflow_name: String,
    status: String,
    events: Vec<String>,
}

/// Execute flow subcommand
pub async fn execute(cmd: FlowCommands) -> Result<()> {
    match cmd {
        FlowCommands::Apply { file } => apply_flow(&file).await,
        FlowCommands::Get { name, output } => get_flows(name.as_deref(), &output).await,
        FlowCommands::Describe { name } => describe_flow(&name).await,
        FlowCommands::Run { name, input, output, daemon, port } => {
            if daemon {
                run_flow_daemon(&name, port).await
            } else {
                run_flow(&name, input.as_deref(), &output).await
            }
        }
        FlowCommands::Status { run_id } => status_flow(&run_id).await,
        FlowCommands::Logs { run_id, follow } => logs_flow(&run_id, follow).await,
        FlowCommands::Visualize { name, format } => visualize_flow(&name, &format).await,
        FlowCommands::Delete { name } => delete_flow(&name).await,
    }
}

/// Apply flow configuration from file
async fn apply_flow(file: &str) -> Result<()> {
    info!("Applying flow configuration from: {}", file);

    // Read and parse flow config
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read flow config: {}", file))?;

    let workflow: Workflow = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse flow config: {}", file))?;

    let flow_name = workflow.metadata.name.clone();
    let step_count = workflow.spec.steps.len();
    let entrypoint = &workflow.spec.entrypoint;

    // Register in our simple registry
    {
        let mut registry = FLOW_REGISTRY.lock().unwrap();
        registry.insert(flow_name.clone(), file.to_string());
    }

    println!("workflow.aof.dev/{} configured", flow_name);
    println!("  Steps: {}", step_count);
    println!("  Entrypoint: {}", entrypoint);

    Ok(())
}

/// List or get flow(s)
async fn get_flows(name: Option<&str>, output: &str) -> Result<()> {
    let registry = FLOW_REGISTRY.lock().unwrap();

    if let Some(flow_name) = name {
        // Get specific flow
        if let Some(file_path) = registry.get(flow_name) {
            let content = fs::read_to_string(file_path)?;
            let workflow: Workflow = serde_yaml::from_str(&content)?;

            match output {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&workflow)?);
                }
                "yaml" => {
                    println!("{}", serde_yaml::to_string(&workflow)?);
                }
                _ => {
                    print_flow_wide(&workflow);
                }
            }
        } else {
            println!("Flow '{}' not found", flow_name);
        }
    } else {
        // List all flows
        if registry.is_empty() {
            println!("No flows configured. Use 'aofctl flow apply -f <file>' to add one.");
            return Ok(());
        }

        match output {
            "json" => {
                let mut flows = Vec::new();
                for (_, file_path) in registry.iter() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        if let Ok(workflow) = serde_yaml::from_str::<Workflow>(&content) {
                            flows.push(workflow);
                        }
                    }
                }
                println!("{}", serde_json::to_string_pretty(&flows)?);
            }
            _ => {
                println!(
                    "{:<20} {:<15} {:<20} {:<10}",
                    "NAME", "ENTRYPOINT", "ERROR HANDLER", "STEPS"
                );
                for (_, file_path) in registry.iter() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        if let Ok(workflow) = serde_yaml::from_str::<Workflow>(&content) {
                            let error_handler = workflow
                                .spec
                                .error_handler
                                .as_deref()
                                .unwrap_or("-");
                            println!(
                                "{:<20} {:<15} {:<20} {:<10}",
                                workflow.metadata.name,
                                workflow.spec.entrypoint,
                                error_handler,
                                workflow.spec.steps.len()
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Describe flow in detail
async fn describe_flow(name: &str) -> Result<()> {
    // Check if name is a file path
    let workflow = if Path::new(name).exists() {
        let content = fs::read_to_string(name)?;
        serde_yaml::from_str::<Workflow>(&content)?
    } else {
        // Look up in registry
        let registry = FLOW_REGISTRY.lock().unwrap();
        let file_path = registry
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?;
        let content = fs::read_to_string(file_path)?;
        serde_yaml::from_str::<Workflow>(&content)?
    };

    println!("Name:         {}", workflow.metadata.name);
    println!("API Version:  {}", workflow.api_version);
    println!("Kind:         {}", workflow.kind);

    if !workflow.metadata.labels.is_empty() {
        println!("Labels:");
        for (k, v) in &workflow.metadata.labels {
            println!("  {}: {}", k, v);
        }
    }

    println!("\nSpec:");
    println!("  Entrypoint: {}", workflow.spec.entrypoint);

    if let Some(error_handler) = &workflow.spec.error_handler {
        println!("  Error Handler: {}", error_handler);
    }

    if let Some(retry) = &workflow.spec.retry {
        println!("  Retry:");
        println!("    Max Attempts: {}", retry.max_attempts);
    }

    println!("\nSteps ({}):", workflow.spec.steps.len());
    for step in &workflow.spec.steps {
        println!("  - {} ({:?})", step.name, step.step_type);

        // Show next steps
        if let Some(next) = &step.next {
            match next {
                NextStep::Simple(target) => {
                    println!("    Next: {}", target);
                }
                NextStep::Conditional(conditions) => {
                    for cond in conditions {
                        if let Some(condition) = &cond.condition {
                            println!("    Next: {} [when {}]", cond.target, condition);
                        } else {
                            println!("    Next: {}", cond.target);
                        }
                    }
                }
            }
        }

        if let Some(timeout) = &step.timeout {
            println!("    Timeout: {}", timeout);
        }
    }

    // Show connections
    println!("\nConnections:");
    for step in &workflow.spec.steps {
        if let Some(next) = &step.next {
            match next {
                NextStep::Simple(target) => {
                    println!("  {} -> {}", step.name, target);
                }
                NextStep::Conditional(conditions) => {
                    for cond in conditions {
                        if let Some(condition) = &cond.condition {
                            println!("  {} -> {} [{}]", step.name, cond.target, condition);
                        } else {
                            println!("  {} -> {}", step.name, cond.target);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Execute a flow
async fn run_flow(name: &str, input: Option<&str>, output: &str) -> Result<()> {
    // Check if name is a file path
    let file_path = if Path::new(name).exists() {
        name.to_string()
    } else {
        // Look up in registry
        let registry = FLOW_REGISTRY.lock().unwrap();
        registry
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?
    };

    info!("Loading flow from: {}", file_path);

    // Create runtime
    let runtime = Arc::new(Runtime::new());

    // Create workflow executor
    let mut executor = WorkflowExecutor::from_file(&file_path, runtime)
        .await
        .context("Failed to load flow")?;

    // Create event channel
    let (event_tx, mut event_rx) = mpsc::channel(100);
    executor = executor.with_event_channel(event_tx);

    // Parse input
    let initial_state: serde_json::Value = if let Some(inp) = input {
        serde_json::from_str(inp).unwrap_or_else(|_| serde_json::json!({ "input": inp }))
    } else {
        serde_json::json!({})
    };

    // Collect events in background
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();
    let event_collector = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // Print events in real-time
            let event_str = match &event {
                WorkflowEvent::Started { run_id, workflow_name } => {
                    format!("[FLOW] Started: {} (run_id: {})", workflow_name, run_id)
                }
                WorkflowEvent::StepStarted { step_name } => {
                    format!("[STEP] Started: {}", step_name)
                }
                WorkflowEvent::StepCompleted { step_name, duration_ms } => {
                    format!("[STEP] Completed: {} ({}ms)", step_name, duration_ms)
                }
                WorkflowEvent::StepFailed { step_name, error } => {
                    format!("[STEP] Failed: {} - {}", step_name, error)
                }
                WorkflowEvent::StateUpdated { key, value } => {
                    format!("[STATE] Updated: {} = {}", key, value)
                }
                WorkflowEvent::WaitingApproval { step_name, approvers } => {
                    format!("[APPROVAL] Waiting: {} (approvers: {:?})", step_name, approvers)
                }
                WorkflowEvent::WaitingInput { step_name, prompt } => {
                    format!("[INPUT] Waiting: {} - {}", step_name, prompt)
                }
                WorkflowEvent::Completed { run_id, status } => {
                    format!("[FLOW] Completed: {} ({:?})", run_id, status)
                }
                WorkflowEvent::Error { message } => {
                    format!("[ERROR] {}", message)
                }
            };
            eprintln!("{}", event_str);
            events_clone.lock().unwrap().push(event_str);
        }
    });

    // Execute workflow
    let state = executor
        .execute(initial_state)
        .await
        .context("Failed to execute flow")?;

    // Wait for events
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    drop(event_collector);

    // Store run state
    {
        let mut run_registry = RUN_REGISTRY.lock().unwrap();
        run_registry.insert(
            state.run_id.clone(),
            RunState {
                workflow_name: state.workflow_name.clone(),
                status: format!("{:?}", state.status),
                events: events.lock().unwrap().clone(),
            },
        );
    }

    // Output result
    match output {
        "json" => {
            let output = serde_json::json!({
                "run_id": state.run_id,
                "workflow": state.workflow_name,
                "status": format!("{:?}", state.status),
                "completed_steps": state.completed_steps,
                "data": state.data,
                "error": state.error
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            let output = serde_json::json!({
                "run_id": state.run_id,
                "workflow": state.workflow_name,
                "status": format!("{:?}", state.status),
                "completed_steps": state.completed_steps,
                "data": state.data,
                "error": state.error
            });
            println!("{}", serde_yaml::to_string(&output)?);
        }
        "text" | _ => {
            println!();
            println!("Run ID:    {}", state.run_id);
            println!("Workflow:  {}", state.workflow_name);
            println!("Status:    {:?}", state.status);
            println!("Steps:     {} completed", state.completed_steps.len());

            if !state.completed_steps.is_empty() {
                println!("Completed: {}", state.completed_steps.join(" -> "));
            }

            if let Some(error) = &state.error {
                println!("Error:     {:?}", error);
            }

            if state.data != serde_json::json!({}) {
                println!("Data:");
                println!("{}", serde_json::to_string_pretty(&state.data)?);
            }
        }
    }

    Ok(())
}

/// Get execution status
async fn status_flow(run_id: &str) -> Result<()> {
    let run_registry = RUN_REGISTRY.lock().unwrap();

    if let Some(state) = run_registry.get(run_id) {
        println!("Run ID:   {}", run_id);
        println!("Workflow: {}", state.workflow_name);
        println!("Status:   {}", state.status);
    } else {
        println!("Run '{}' not found", run_id);
        println!("Note: Run state is only stored in-memory for the current session.");
    }

    Ok(())
}

/// View execution logs
async fn logs_flow(run_id: &str, _follow: bool) -> Result<()> {
    let run_registry = RUN_REGISTRY.lock().unwrap();

    if let Some(state) = run_registry.get(run_id) {
        println!("Logs for run: {}", run_id);
        println!("Workflow: {}", state.workflow_name);
        println!();
        for event in &state.events {
            println!("{}", event);
        }
    } else {
        println!("Run '{}' not found", run_id);
        println!("Note: Run logs are only stored in-memory for the current session.");
    }

    Ok(())
}

/// Helper to get all next step targets from a NextStep enum
fn get_next_targets(next: &NextStep) -> Vec<String> {
    match next {
        NextStep::Simple(target) => vec![target.clone()],
        NextStep::Conditional(conditions) => {
            conditions.iter().map(|c| c.target.clone()).collect()
        }
    }
}

/// Visualize flow as ASCII graph
async fn visualize_flow(name: &str, format: &str) -> Result<()> {
    // Check if name is a file path
    let workflow = if Path::new(name).exists() {
        let content = fs::read_to_string(name)?;
        serde_yaml::from_str::<Workflow>(&content)?
    } else {
        // Look up in registry
        let registry = FLOW_REGISTRY.lock().unwrap();
        let file_path = registry
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?;
        let content = fs::read_to_string(file_path)?;
        serde_yaml::from_str::<Workflow>(&content)?
    };

    match format {
        "dot" => {
            // Generate DOT format for Graphviz
            println!("digraph {} {{", workflow.metadata.name.replace('-', "_"));
            println!("  rankdir=TB;");
            println!("  node [shape=box];");
            println!();

            // Mark entrypoint
            println!("  start [shape=circle, label=\"\", width=0.3, style=filled, fillcolor=green];");
            println!("  start -> {};", workflow.spec.entrypoint.replace('-', "_"));

            // Add nodes and edges
            for step in &workflow.spec.steps {
                let node_name = step.name.replace('-', "_");
                let label = format!("{} ({:?})", step.name, step.step_type);
                println!("  {} [label=\"{}\"];", node_name, label);

                if let Some(next) = &step.next {
                    match next {
                        NextStep::Simple(target) => {
                            let target_name = target.replace('-', "_");
                            println!("  {} -> {};", node_name, target_name);
                        }
                        NextStep::Conditional(conditions) => {
                            for cond in conditions {
                                let target_name = cond.target.replace('-', "_");
                                if let Some(condition) = &cond.condition {
                                    println!("  {} -> {} [label=\"{}\"];", node_name, target_name, condition);
                                } else {
                                    println!("  {} -> {};", node_name, target_name);
                                }
                            }
                        }
                    }
                } else {
                    // Terminal step
                    println!("  {} -> end;", node_name);
                }
            }

            println!("  end [shape=doublecircle, label=\"\", width=0.3, style=filled, fillcolor=red];");
            println!("}}");
        }
        "ascii" | _ => {
            // Generate ASCII representation
            println!("Flow: {}", workflow.metadata.name);
            println!("======{}", "=".repeat(workflow.metadata.name.len()));
            println!();

            // Find entrypoint
            println!("  [START]");
            println!("     |");
            println!("     v");

            // Build step map for traversal
            let step_map: HashMap<&str, &aof_core::WorkflowStep> = workflow
                .spec
                .steps
                .iter()
                .map(|s| (s.name.as_str(), s))
                .collect();

            // Simple DFS visualization
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![workflow.spec.entrypoint.as_str()];

            while let Some(step_name) = stack.pop() {
                if visited.contains(step_name) {
                    continue;
                }
                visited.insert(step_name);

                if let Some(step) = step_map.get(step_name) {
                    let type_str = format!("{:?}", step.step_type);
                    println!("  +{}+", "-".repeat(type_str.len() + 4));
                    println!("  | {} |", step.name);
                    println!("  | ({}) |", type_str);
                    println!("  +{}+", "-".repeat(type_str.len() + 4));

                    if let Some(next) = &step.next {
                        let targets = get_next_targets(next);
                        if targets.is_empty() {
                            println!("     |");
                            println!("     v");
                            println!("   [END]");
                        } else if targets.len() == 1 {
                            println!("     |");
                            println!("     v");
                            if !visited.contains(targets[0].as_str()) {
                                stack.push(Box::leak(targets[0].clone().into_boxed_str()));
                            }
                        } else {
                            // Multiple branches
                            println!("     |");
                            let branch_count = targets.len();
                            println!("  +--{}--+", "--+--".repeat(branch_count - 1));
                            println!("  |  {}  |", targets.join("  |  "));
                            println!("  v  {}  v", "  v  ".repeat(branch_count - 1));

                            for target in &targets {
                                if !visited.contains(target.as_str()) {
                                    stack.push(Box::leak(target.clone().into_boxed_str()));
                                }
                            }
                        }
                    } else {
                        println!("     |");
                        println!("     v");
                        println!("   [END]");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Run flow as a daemon with trigger support
async fn run_flow_daemon(name: &str, port: u16) -> Result<()> {
    use tokio::signal;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::net::SocketAddr;

    // Check if name is a file path
    let file_path = if Path::new(name).exists() {
        name.to_string()
    } else {
        // Look up in registry
        let registry = FLOW_REGISTRY.lock().unwrap();
        registry
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?
    };

    // Load workflow to get trigger info
    let content = fs::read_to_string(&file_path)?;
    let workflow: Workflow = serde_yaml::from_str(&content)?;
    let flow_name = workflow.metadata.name.clone();

    println!("Starting flow daemon: {}", flow_name);
    println!("  File: {}", file_path);
    println!("  Entrypoint: {}", workflow.spec.entrypoint);
    println!("  Steps: {}", workflow.spec.steps.len());

    let addr = format!("0.0.0.0:{}", port);
    println!();
    println!("Listening for triggers on http://{}", addr);
    println!("  Webhook endpoint: http://localhost:{}/webhook/{}",
             port,
             flow_name);
    println!();
    println!("Press Ctrl+C to stop");

    let socket_addr: SocketAddr = addr.parse().context("Invalid address")?;
    let listener = TcpListener::bind(socket_addr).await.context("Failed to bind")?;

    let file_path_clone = file_path.clone();

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((mut socket, peer_addr)) => {
                        let file_path = file_path_clone.clone();
                        let flow_name = flow_name.clone();

                        tokio::spawn(async move {
                            let mut buffer = [0u8; 4096];
                            if let Ok(n) = socket.read(&mut buffer).await {
                                let request = String::from_utf8_lossy(&buffer[..n]);

                                // Parse basic HTTP request
                                let lines: Vec<&str> = request.lines().collect();
                                if let Some(first_line) = lines.first() {
                                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        let method = parts[0];
                                        let path = parts[1];

                                        let now = chrono::Utc::now();
                                        eprintln!("[{}] {} {} from {}",
                                                  now.format("%H:%M:%S"),
                                                  method, path, peer_addr);

                                        // Check if this is a webhook request for our flow
                                        let expected_path = format!("/webhook/{}", flow_name);
                                        if path == expected_path || path == "/webhook" {
                                            // Extract body (after empty line)
                                            let body_start = request.find("\r\n\r\n")
                                                .map(|i| i + 4)
                                                .unwrap_or(0);
                                            let body = &request[body_start..];

                                            eprintln!("[TRIGGER] Webhook received, executing flow...");

                                            // Execute the flow
                                            match execute_flow_from_trigger(&file_path, body).await {
                                                Ok(result) => {
                                                    let response = format!(
                                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                                                        serde_json::to_string(&result).unwrap_or_default()
                                                    );
                                                    let _ = socket.write_all(response.as_bytes()).await;
                                                }
                                                Err(e) => {
                                                    eprintln!("[ERROR] Flow execution failed: {}", e);
                                                    let response = format!(
                                                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{{\"error\": \"{}\"}}",
                                                        e.to_string().replace('"', "'")
                                                    );
                                                    let _ = socket.write_all(response.as_bytes()).await;
                                                }
                                            }
                                        } else if path == "/health" {
                                            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\": \"healthy\"}";
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        } else {
                                            let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot Found";
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Accept failed: {}", e);
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!();
                println!("Shutting down flow daemon...");
                break;
            }
        }
    }

    Ok(())
}

/// Execute flow from trigger data
async fn execute_flow_from_trigger(file_path: &str, trigger_data: &str) -> Result<serde_json::Value> {
    // Create runtime
    let runtime = Arc::new(Runtime::new());

    // Create workflow executor
    let mut executor = WorkflowExecutor::from_file(file_path, runtime)
        .await
        .context("Failed to load flow")?;

    // Parse trigger data as JSON
    let initial_state: serde_json::Value = if !trigger_data.is_empty() {
        serde_json::from_str(trigger_data)
            .unwrap_or_else(|_| serde_json::json!({ "trigger_data": trigger_data }))
    } else {
        serde_json::json!({})
    };

    // Execute workflow
    let state = executor
        .execute(initial_state)
        .await
        .context("Failed to execute flow")?;

    // Return result
    Ok(serde_json::json!({
        "run_id": state.run_id,
        "workflow": state.workflow_name,
        "status": format!("{:?}", state.status),
        "completed_steps": state.completed_steps,
        "data": state.data,
        "error": state.error
    }))
}

/// Delete/remove a flow
async fn delete_flow(name: &str) -> Result<()> {
    let mut registry = FLOW_REGISTRY.lock().unwrap();

    if registry.remove(name).is_some() {
        println!("workflow.aof.dev/{} deleted", name);
    } else {
        println!("Flow '{}' not found in registry", name);
    }

    Ok(())
}

/// Print flow in wide format
fn print_flow_wide(workflow: &Workflow) {
    let step_names: Vec<&str> = workflow.spec.steps.iter().map(|s| s.name.as_str()).collect();
    let error_handler = workflow.spec.error_handler.as_deref().unwrap_or("-");

    println!(
        "{:<20} {:<15} {:<15} {:<10} {}",
        "NAME", "ENTRYPOINT", "ERROR HANDLER", "STEPS", "STEP NAMES"
    );
    println!(
        "{:<20} {:<15} {:<15} {:<10} {}",
        workflow.metadata.name,
        workflow.spec.entrypoint,
        error_handler,
        workflow.spec.steps.len(),
        step_names.join(", ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_registry() {
        let mut registry = FLOW_REGISTRY.lock().unwrap();
        registry.insert("test-flow".to_string(), "/tmp/test.yaml".to_string());
        assert!(registry.contains_key("test-flow"));
        registry.remove("test-flow");
    }
}
