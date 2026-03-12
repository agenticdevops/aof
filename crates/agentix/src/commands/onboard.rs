use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use agentix_core::{
    AgentModelConfig, GatewayConfig, ProviderConfig, WorkspaceConfig, WorkspaceDefaults,
    WorkspaceMetadata, WorkspaceSpec,
};

use crate::commands::init::scaffold_agent_directory;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Execute the `agentix onboard` command.
///
/// Creates a complete workspace:
/// - `agentix.yaml`              — workspace configuration
/// - `<agents_dir>/hello-world/` — starter agent directory
pub async fn execute(non_interactive: bool) -> Result<()> {
    if non_interactive {
        execute_non_interactive()
    } else {
        if !io::stdin().is_terminal() {
            bail!(
                "Not running in an interactive terminal. \
                Use --non-interactive for automated setup."
            );
        }
        execute_interactive()
    }
}

// ---------------------------------------------------------------------------
// Interactive mode
// ---------------------------------------------------------------------------

fn execute_interactive() -> Result<()> {
    // Check if workspace already exists
    if Path::new("./agentix.yaml").exists() {
        println!(
            "Workspace already exists at ./agentix.yaml. \
            Run `agentix init` to add more agents."
        );
        return Ok(());
    }

    // Welcome banner
    println!();
    println!("Welcome to OpenAgentiX!");
    println!("Let's set up your workspace.");
    println!();

    // Workspace name
    let default_workspace_name = current_dir_name().unwrap_or_else(|| "my-workspace".to_string());
    let workspace_name = prompt_with_default(
        &format!(
            "Workspace name (e.g., my-project) [{}]: ",
            default_workspace_name
        ),
        &default_workspace_name,
    )?;
    validate_slug(&workspace_name)?;

    // LLM provider setup
    let (providers, default_model) = setup_providers()?;

    // Gateway port
    let port_str = prompt_with_default("Gateway port [7777]: ", "7777")?;
    let port: u16 = port_str.parse().unwrap_or(7777);

    // Agents directory
    let agents_dir = prompt_with_default("Agents directory [./agents]: ", "./agents")?;

    // Build workspace config
    let workspace = WorkspaceConfig {
        api_version: "openagentix.dev/v1".to_string(),
        kind: "Workspace".to_string(),
        metadata: WorkspaceMetadata {
            name: workspace_name.clone(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
        },
        spec: WorkspaceSpec {
            defaults: WorkspaceDefaults {
                model: default_model.clone(),
                max_iterations: Some(10),
                timeout: Some("5m".to_string()),
                mode: None,
            },
            providers,
            gateway: GatewayConfig {
                host: "127.0.0.1".to_string(),
                port,
            },
            agents_dir: agents_dir.clone(),
        },
    };

    // Write agentix.yaml
    write_workspace_config(&workspace)?;

    // Create hello-world agent directory
    let model_config = default_model.map(|m| AgentModelConfig { preferred: Some(m) });
    let hello_world_soul = hello_world_soul_content();
    scaffold_agent_directory(
        &agents_dir,
        "hello-world",
        Some("A helpful assistant that demonstrates the OpenAgentiX agent directory format"),
        model_config,
        &hello_world_soul,
        None,
    )?;

    // Offer to start gateway
    let start_gateway = prompt_yes_no("Start the gateway now? (y/n): ")?;

    // Print summary
    print_summary(&agents_dir, port, start_gateway);

    if start_gateway {
        println!();
        println!("Starting gateway...");
        println!("(Run `agentix gateway start` to start manually when ready)");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Non-interactive mode
// ---------------------------------------------------------------------------

fn execute_non_interactive() -> Result<()> {
    // Check if workspace already exists
    if Path::new("./agentix.yaml").exists() {
        println!(
            "Workspace already exists at ./agentix.yaml. \
            Run `agentix init` to add more agents."
        );
        return Ok(());
    }

    let workspace_name = current_dir_name().unwrap_or_else(|| "my-workspace".to_string());
    let agents_dir = "./agents".to_string();
    let port: u16 = 7777;

    // Build workspace config with no providers (user must fill in)
    let workspace = WorkspaceConfig {
        api_version: "openagentix.dev/v1".to_string(),
        kind: "Workspace".to_string(),
        metadata: WorkspaceMetadata {
            name: workspace_name.clone(),
            labels: HashMap::new(),
            annotations: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    "OpenAgentiX workspace — edit providers section with your API keys"
                        .to_string(),
                );
                m
            },
        },
        spec: WorkspaceSpec {
            defaults: WorkspaceDefaults {
                model: None,
                max_iterations: Some(10),
                timeout: Some("5m".to_string()),
                mode: None,
            },
            providers: HashMap::new(),
            gateway: GatewayConfig {
                host: "127.0.0.1".to_string(),
                port,
            },
            agents_dir: agents_dir.clone(),
        },
    };

    write_workspace_config(&workspace)?;

    // Create hello-world agent with no model (inherits from workspace — user must fill in)
    let hello_world_soul = hello_world_soul_content();
    scaffold_agent_directory(
        &agents_dir,
        "hello-world",
        Some("A helpful assistant that demonstrates the OpenAgentiX agent directory format"),
        None,
        &hello_world_soul,
        None,
    )?;

    println!("Workspace created!");
    println!("  Config:  ./agentix.yaml");
    println!("  Agents:  {}/hello-world/", agents_dir);
    println!();
    println!("Before starting the gateway, configure providers in agentix.yaml:");
    println!("  spec:");
    println!("    providers:");
    println!("      anthropic:");
    println!("        api_key: \"${{ANTHROPIC_API_KEY}}\"");
    println!("    defaults:");
    println!("      model: anthropic/claude-sonnet-4-6");
    println!();
    println!("Then run: agentix gateway start");

    Ok(())
}

// ---------------------------------------------------------------------------
// Provider setup (interactive)
// ---------------------------------------------------------------------------

/// Interactively collect LLM provider credentials.
///
/// Returns `(providers_map, default_model)`.
fn setup_providers() -> Result<(HashMap<String, ProviderConfig>, Option<String>)> {
    let mut providers = HashMap::new();
    let mut default_model: Option<String> = None;

    println!("LLM Provider Setup");
    println!("  (1) Anthropic");
    println!("  (2) OpenAI");
    println!("  (3) Google");
    println!("  (4) Ollama");
    println!("  (5) Skip");

    loop {
        let choice = prompt("Which LLM provider? (1-5): ")?;
        match choice.trim() {
            "1" => {
                let api_key = prompt(
                    "Anthropic API key (or env var ref like ${ANTHROPIC_API_KEY}): ",
                )?;
                if api_key.contains("${") {
                    println!(
                        "Remember to export ANTHROPIC_API_KEY before starting the gateway."
                    );
                }
                providers.insert(
                    "anthropic".to_string(),
                    ProviderConfig {
                        api_key: Some(api_key),
                        base_url: None,
                    },
                );
                let suggested = "anthropic/claude-sonnet-4-6";
                let model = prompt_with_default(
                    &format!("Default model [{}]: ", suggested),
                    suggested,
                )?;
                default_model = Some(model);
            }
            "2" => {
                let api_key =
                    prompt("OpenAI API key (or env var ref like ${OPENAI_API_KEY}): ")?;
                if api_key.contains("${") {
                    println!(
                        "Remember to export OPENAI_API_KEY before starting the gateway."
                    );
                }
                providers.insert(
                    "openai".to_string(),
                    ProviderConfig {
                        api_key: Some(api_key),
                        base_url: None,
                    },
                );
                let suggested = "openai/gpt-4o";
                let model = prompt_with_default(
                    &format!("Default model [{}]: ", suggested),
                    suggested,
                )?;
                default_model = Some(model);
            }
            "3" => {
                let api_key =
                    prompt("Google API key (or env var ref like ${GOOGLE_API_KEY}): ")?;
                if api_key.contains("${") {
                    println!(
                        "Remember to export GOOGLE_API_KEY before starting the gateway."
                    );
                }
                providers.insert(
                    "google".to_string(),
                    ProviderConfig {
                        api_key: Some(api_key),
                        base_url: None,
                    },
                );
                let suggested = "google/gemini-2.0-flash";
                let model = prompt_with_default(
                    &format!("Default model [{}]: ", suggested),
                    suggested,
                )?;
                default_model = Some(model);
            }
            "4" => {
                let url = prompt_with_default(
                    "Ollama URL [http://localhost:11434]: ",
                    "http://localhost:11434",
                )?;
                providers.insert(
                    "ollama".to_string(),
                    ProviderConfig {
                        api_key: None,
                        base_url: Some(url),
                    },
                );
                let suggested = "ollama/llama3.2";
                let model = prompt_with_default(
                    &format!("Default model [{}]: ", suggested),
                    suggested,
                )?;
                default_model = Some(model);
            }
            "5" | "" => break,
            _ => {
                eprintln!("Please choose 1-5.");
                continue;
            }
        }

        let add_more = prompt_yes_no("Add another provider? (y/n): ")?;
        if !add_more {
            break;
        }
    }

    Ok((providers, default_model))
}

// ---------------------------------------------------------------------------
// agentix.yaml writer
// ---------------------------------------------------------------------------

fn write_workspace_config(workspace: &WorkspaceConfig) -> Result<()> {
    let yaml = serde_yaml::to_string(workspace)
        .map_err(|e| anyhow::anyhow!("Failed to serialize workspace config: {}", e))?;
    std::fs::write("./agentix.yaml", &yaml)?;
    println!("Created: ./agentix.yaml");
    Ok(())
}

// ---------------------------------------------------------------------------
// Summary output
// ---------------------------------------------------------------------------

fn print_summary(agents_dir: &str, port: u16, start_gateway: bool) {
    println!();
    println!("Workspace created!");
    println!("  Config:  ./agentix.yaml");
    println!("  Agents:  {}/hello-world/", agents_dir);
    println!("  Gateway: http://127.0.0.1:{}", port);
    println!();
    println!("Next steps:");
    if start_gateway {
        println!("  agentix agents                 # List loaded agents");
    } else {
        println!("  agentix gateway start          # Start the gateway");
        println!("  agentix agents                 # List loaded agents");
    }
    println!("  agentix init --name my-agent   # Create more agents");
}

// ---------------------------------------------------------------------------
// Hello-world agent SOUL content
// ---------------------------------------------------------------------------

fn hello_world_soul_content() -> String {
    "# Hello World Agent\n\
    \n\
    You are a helpful AI assistant demonstrating the OpenAgentiX agent format.\n\
    \n\
    ## Purpose\n\
    \n\
    Answer questions clearly and helpfully. Demonstrate how agents work in OpenAgentiX.\n\
    \n\
    ## Approach\n\
    \n\
    - Think step by step before answering\n\
    - Be concise and precise\n\
    - If you don't know something, say so clearly\n\
    - For technical questions, provide examples where helpful\n"
        .to_string()
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    let raw = prompt(message)?;
    if raw.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(raw)
    }
}

fn prompt_yes_no(message: &str) -> Result<bool> {
    let answer = prompt(message)?;
    Ok(answer.trim().eq_ignore_ascii_case("y"))
}

fn validate_slug(name: &str) -> Result<()> {
    if name.len() > 63 {
        bail!("Name must be at most 63 characters");
    }
    let valid = name.len() == 1 && name.chars().all(|c| c.is_ascii_lowercase())
        || (name.len() >= 2
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && name.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    if !valid {
        bail!(
            "Workspace name must match ^[a-z][a-z0-9-]*[a-z0-9]$ \
            (lowercase alphanumeric with hyphens), got \"{}\"",
            name
        );
    }
    Ok(())
}

fn current_dir_name() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .map(|name| {
            // Convert to valid slug: lowercase, replace non-alphanumeric with '-'
            let slug: String = name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect();
            // Strip leading/trailing hyphens
            slug.trim_matches('-').to_string()
        })
        .filter(|s| !s.is_empty())
}
