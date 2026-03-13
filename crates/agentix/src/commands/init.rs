use anyhow::{bail, Result};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use agentix_core::{AgentManifest, AgentModelConfig, DirectoryLoader};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Execute the `agentix init` command.
///
/// Creates a GitAgent-compatible agent directory:
/// ```text
/// <output_dir>/<name>/
/// ├── agent.yaml    # Minimal manifest
/// └── SOUL.md       # System prompt / identity
/// ```
/// Optionally creates RULES.md if the user provides constraints.
pub async fn execute(
    name: Option<String>,
    output_dir: &str,
    non_interactive: bool,
) -> Result<()> {
    if non_interactive {
        execute_non_interactive(name, output_dir)
    } else {
        if !io::stdin().is_terminal() {
            bail!(
                "Not running in an interactive terminal. \
                Use --non-interactive for automated setup."
            );
        }
        execute_interactive(name, output_dir)
    }
}

// ---------------------------------------------------------------------------
// Interactive mode
// ---------------------------------------------------------------------------

fn execute_interactive(provided_name: Option<String>, output_dir: &str) -> Result<()> {
    let name = match provided_name {
        Some(n) => {
            validate_agent_name(&n)?;
            n
        }
        None => prompt_agent_name()?,
    };

    let model = prompt_optional("Model (e.g., anthropic/claude-sonnet-4-6, press Enter to use workspace default): ")?;
    let description = prompt_optional("Brief description (optional): ")?;

    println!("What is this agent's identity and purpose? (This becomes SOUL.md)");
    println!("Enter text, then press Enter twice to finish:");
    let soul_text = read_multiline()?;

    println!("Any hard constraints (RULES.md)? Enter rules or press Enter to skip:");
    println!("Enter text, then press Enter twice to finish:");
    let rules_text = read_multiline()?;

    let soul_content = if soul_text.trim().is_empty() {
        default_soul_content(&name)
    } else {
        soul_text
    };

    let rules_content = if rules_text.trim().is_empty() {
        None
    } else {
        Some(rules_text)
    };

    let model_config = model.map(|m| AgentModelConfig {
        preferred: Some(m),
    });

    scaffold_agent_directory(
        output_dir,
        &name,
        description.as_deref(),
        model_config,
        &soul_content,
        rules_content.as_deref(),
    )
}

// ---------------------------------------------------------------------------
// Non-interactive mode
// ---------------------------------------------------------------------------

fn execute_non_interactive(name: Option<String>, output_dir: &str) -> Result<()> {
    let name = match name {
        Some(n) => {
            validate_agent_name(&n)?;
            n
        }
        None => bail!(
            "Non-interactive mode requires --name. \
            Usage: agentix init --non-interactive --name <agent-name>"
        ),
    };

    // Try to read workspace default model from ./agentix.yaml
    let model_config = read_workspace_model();

    let soul_content = default_soul_content(&name);

    scaffold_agent_directory(
        output_dir,
        &name,
        None,
        model_config,
        &soul_content,
        None, // No RULES.md in non-interactive mode
    )
}

// ---------------------------------------------------------------------------
// Core scaffold logic (shared by both modes)
// ---------------------------------------------------------------------------

/// Scaffold a complete agent directory at `<output_dir>/<name>/`.
///
/// Writes `agent.yaml`, `SOUL.md`, and optionally `RULES.md`.
/// Validates the directory after writing using `DirectoryLoader::load`.
pub fn scaffold_agent_directory(
    output_dir: &str,
    name: &str,
    description: Option<&str>,
    model: Option<AgentModelConfig>,
    soul_content: &str,
    rules_content: Option<&str>,
) -> Result<()> {
    let agent_dir = PathBuf::from(output_dir).join(name);

    // Check for existing directory
    if agent_dir.exists() {
        if io::stdin().is_terminal() {
            print!(
                "Directory {} already exists. Overwrite? (y/n): ",
                agent_dir.display()
            );
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().lock().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                println!("Skipping — agent directory already exists.");
                return Ok(());
            }
        } else {
            eprintln!(
                "Warning: agent directory {} already exists, skipping.",
                agent_dir.display()
            );
            return Ok(());
        }
    }

    // Create the directory
    std::fs::create_dir_all(&agent_dir)?;

    // Build and serialize the manifest
    let manifest = AgentManifest {
        spec_version: "0.1.0".to_string(),
        name: name.to_string(),
        version: Some("0.1.0".to_string()),
        description: description.map(str::to_string),
        model,
        extends: None,
        dependencies: vec![],
        mcp_servers: vec![],
        triggers: vec![],
        notifications: vec![],
        vector_memory: agentix_core::VectorMemoryConfig::default(),
        research_phase: agentix_core::ResearchPhaseConfig::default(),
    };

    let manifest_yaml = serde_yaml::to_string(&manifest)
        .map_err(|e| anyhow::anyhow!("Failed to serialize agent manifest: {}", e))?;

    // Write agent.yaml
    let manifest_path = agent_dir.join("agent.yaml");
    std::fs::write(&manifest_path, &manifest_yaml)?;

    // Write SOUL.md
    let soul_path = agent_dir.join("SOUL.md");
    std::fs::write(&soul_path, soul_content)?;

    // Write RULES.md (optional)
    if let Some(rules) = rules_content {
        let rules_path = agent_dir.join("RULES.md");
        let rules_md = format!("# Rules\n\n{}\n", rules.trim());
        std::fs::write(&rules_path, &rules_md)?;
    }

    // Validate the directory using DirectoryLoader
    if let Err(e) = DirectoryLoader::load(&agent_dir) {
        eprintln!("Error: generated agent directory failed validation: {}", e);
        eprintln!("Removing directory: {}", agent_dir.display());
        let _ = std::fs::remove_dir_all(&agent_dir);
        bail!("Agent directory validation failed: {}", e);
    }

    // Print success output
    print_success(&agent_dir, name, rules_content.is_some());

    Ok(())
}

fn print_success(agent_dir: &Path, _name: &str, has_rules: bool) {
    println!();
    println!(
        "Created agent directory: {}/",
        agent_dir.display()
    );
    println!("  agent.yaml    (manifest)");
    println!("  SOUL.md       (identity)");
    if has_rules {
        println!("  RULES.md      (constraints)");
    }
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/SOUL.md to define your agent", agent_dir.display());
    println!("  2. Add tools to {}/tools/ (optional)", agent_dir.display());
    println!("  3. agentix validate {}", agent_dir.display());
    println!("  4. agentix gateway start");
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

fn prompt_agent_name() -> Result<String> {
    loop {
        print!("Agent name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim().to_string();
        if input.is_empty() {
            eprintln!("Agent name cannot be empty.");
            continue;
        }
        match validate_agent_name(&input) {
            Ok(()) => return Ok(input),
            Err(e) => eprintln!("Invalid name: {}", e),
        }
    }
}

fn prompt_optional(prompt: &str) -> Result<Option<String>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}

/// Read multiple lines until the user enters a blank line.
fn read_multiline() -> Result<String> {
    let stdin = io::stdin();
    let mut lines = Vec::new();
    let mut consecutive_blank = 0;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            consecutive_blank += 1;
            if consecutive_blank >= 1 {
                break;
            }
        } else {
            consecutive_blank = 0;
            lines.push(line);
        }
    }
    Ok(lines.join("\n"))
}

/// Validate an agent name against the spec pattern.
fn validate_agent_name(name: &str) -> Result<()> {
    if name.len() > 63 {
        bail!("Agent name must be at most 63 characters");
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
            "Agent name must match ^[a-z][a-z0-9-]*[a-z0-9]$ \
            (lowercase alphanumeric with hyphens), got \"{}\"",
            name
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Content templates
// ---------------------------------------------------------------------------

fn default_soul_content(name: &str) -> String {
    format!(
        "# {title}\n\
        \n\
        You are a helpful AI assistant.\n\
        \n\
        ## Purpose\n\
        \n\
        [Describe what this agent does and its goals here.]\n\
        \n\
        ## Approach\n\
        \n\
        Think step by step. Be concise and precise in your responses.\n",
        title = to_title_case(name),
    )
}

fn to_title_case(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Workspace default model (optional — best-effort)
// ---------------------------------------------------------------------------

fn read_workspace_model() -> Option<AgentModelConfig> {
    let content = std::fs::read_to_string("./agentix.yaml").ok()?;
    let workspace = agentix_core::WorkspaceConfig::from_yaml(&content).ok()?;
    let model_str = workspace.spec.defaults.model?;
    Some(AgentModelConfig {
        preferred: Some(model_str),
    })
}
