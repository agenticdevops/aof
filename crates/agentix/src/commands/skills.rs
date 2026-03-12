//! `agentix skills` command — list and inspect built-in skill packs.

use agentix_core::skills::SkillRegistry;

use crate::cli::OutputFormat;

/// Subcommands for `agentix skills`.
#[derive(clap::Subcommand, Debug)]
pub enum SkillsCommands {
    /// List all available built-in skill packs
    List,
    /// Show the content of a specific skill pack
    Show {
        /// Skill pack name (e.g., "kubernetes")
        name: String,
    },
}

/// Handle `agentix skills <subcommand>`.
pub async fn run(command: &SkillsCommands, output: &OutputFormat) -> anyhow::Result<()> {
    match command {
        SkillsCommands::List => list_skills(output).await,
        SkillsCommands::Show { name } => show_skill(name, output).await,
    }
}

async fn list_skills(output: &OutputFormat) -> anyhow::Result<()> {
    let registry = SkillRegistry::new();
    let packs = registry.builtin_packs();

    match output {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = packs
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "description": p.description,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Text => {
            println!("Built-in Skill Packs\n");
            println!("{:<20} {}", "NAME", "DESCRIPTION");
            println!("{}", "─".repeat(72));
            for pack in packs {
                println!("{:<20} {}", pack.name, pack.description);
            }
            println!("\n{} skill packs available", packs.len());
            println!("\nUsage: activate a built-in skill pack for your agent:");
            println!("  mkdir -p agents/my-agent/skills/<pack-name>");
            println!("\nCustom skills: place SKILL.md in agents/<name>/skills/<skill-name>/SKILL.md");
        }
    }
    Ok(())
}

async fn show_skill(name: &str, output: &OutputFormat) -> anyhow::Result<()> {
    let registry = SkillRegistry::new();
    match registry.get_builtin(name) {
        Some(pack) => match output {
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "name": pack.name,
                    "description": pack.description,
                    "content": pack.content,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            OutputFormat::Text => {
                println!("Skill Pack: {}", pack.name);
                println!("Description: {}", pack.description);
                println!("\n{}", pack.content);
            }
        },
        None => {
            eprintln!("Error: skill pack '{}' not found.", name);
            eprintln!("Run `agentix skills list` to see available packs.");
            std::process::exit(1);
        }
    }
    Ok(())
}
