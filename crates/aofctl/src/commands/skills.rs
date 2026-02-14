//! Skills management commands for aofctl.
//!
//! Provides kubectl-style commands for managing agentic skills:
//! - `aofctl get skills` - List all skills
//! - `aofctl get skill <name>` - Get a specific skill
//! - `aofctl describe skill <name>` - Describe a skill in detail

use std::path::PathBuf;

use aof_skills::{
    RequirementChecker, Skill, SkillConfig, SkillLoader, SkillRegistry,
    build_skills_prompt,
};
use clap::Subcommand;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};

/// Skills subcommand for aofctl
#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// List all loaded skills
    List {
        /// Output format (table, json, yaml, wide, name)
        #[arg(short, long, default_value = "table")]
        output: String,

        /// Show only eligible skills (requirements met)
        #[arg(long)]
        eligible: bool,

        /// Skills directory to load from
        #[arg(long)]
        skills_dir: Option<String>,
    },

    /// Check if a skill's requirements are met
    Check {
        /// Skill name to check
        name: String,

        /// Skills directory
        #[arg(long)]
        skills_dir: Option<String>,
    },

    /// Show skill content/instructions
    Show {
        /// Skill name
        name: String,

        /// Skills directory
        #[arg(long)]
        skills_dir: Option<String>,
    },

    /// Generate prompt injection for skills
    Prompt {
        /// Skill names to include (comma-separated, or 'all' for eligible skills)
        #[arg(default_value = "all")]
        skills: String,

        /// Skills directory
        #[arg(long)]
        skills_dir: Option<String>,
    },

    /// Search skills by query
    Search {
        /// Search query
        query: String,

        /// Skills directory
        #[arg(long)]
        skills_dir: Option<String>,
    },
}

/// Execute skills commands
pub async fn execute(command: SkillsCommands) -> anyhow::Result<()> {
    match command {
        SkillsCommands::List {
            output,
            eligible,
            skills_dir,
        } => list_skills(&output, eligible, skills_dir).await,
        SkillsCommands::Check { name, skills_dir } => check_skill(&name, skills_dir).await,
        SkillsCommands::Show { name, skills_dir } => show_skill(&name, skills_dir).await,
        SkillsCommands::Prompt { skills, skills_dir } => generate_prompt(&skills, skills_dir).await,
        SkillsCommands::Search { query, skills_dir } => search_skills(&query, skills_dir).await,
    }
}

/// Build skill config from options
fn build_config(skills_dir: Option<String>) -> SkillConfig {
    let mut config = SkillConfig::default();

    // Default bundled skills directory
    let bundled_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("skills"))
        .unwrap_or_else(|| PathBuf::from("skills"));

    config.bundled_dirs.push(bundled_dir);

    // Workspace skills from current directory
    let workspace_skills = PathBuf::from(".claude/skills");
    if workspace_skills.exists() {
        config.workspace_dir = Some(workspace_skills);
    }

    // User-specified directory
    if let Some(dir) = skills_dir {
        config.workspace_dir = Some(PathBuf::from(dir));
    }

    config
}

/// List all skills
async fn list_skills(output: &str, eligible_only: bool, skills_dir: Option<String>) -> anyhow::Result<()> {
    let config = build_config(skills_dir);
    let registry = SkillRegistry::new(config);
    registry.load().await?;

    let skills = if eligible_only {
        registry.eligible().await
    } else {
        registry.all().await
    };

    if skills.is_empty() {
        println!("{}", "No skills found.".yellow());
        return Ok(());
    }

    match output {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&skills)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&skills)?);
        }
        "name" => {
            for skill in &skills {
                println!("{}", skill.name);
            }
        }
        "wide" => {
            print_skills_table_wide(&skills)?;
        }
        _ => {
            print_skills_table(&skills)?;
        }
    }

    Ok(())
}

/// Print skills in a table format
fn print_skills_table(skills: &[Skill]) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["NAME", "DESCRIPTION", "TAGS"]);

    let mut checker = RequirementChecker::new();

    for skill in skills {
        let check = checker.check(skill);
        let name = if check.eligible {
            skill.name.clone()
        } else {
            format!("{} (requires)", skill.name)
        };

        let tags = skill.metadata.tags.join(", ");
        let desc = if skill.description.len() > 50 {
            format!("{}...", &skill.description[..47])
        } else {
            skill.description.clone()
        };

        table.add_row(vec![
            Cell::new(name),
            Cell::new(desc),
            Cell::new(tags),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// Print skills in wide table format
fn print_skills_table_wide(skills: &[Skill]) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["NAME", "DESCRIPTION", "SOURCE", "BINS", "ENV", "ELIGIBLE"]);

    let mut checker = RequirementChecker::new();

    for skill in skills {
        let check = checker.check(skill);

        let source = match &skill.source {
            aof_skills::SkillSource::Bundled => "bundled".to_string(),
            aof_skills::SkillSource::Workspace { path } => format!("workspace:{}", path.display()),
            aof_skills::SkillSource::EnterpriseRegistry { org, .. } => format!("enterprise:{}", org),
            aof_skills::SkillSource::PublicRegistry { .. } => "public".to_string(),
        };

        let bins = skill.metadata.requires.bins.join(", ");
        let env = skill.metadata.requires.env.join(", ");

        let eligible_cell = if check.eligible {
            Cell::new("Yes").fg(Color::Green)
        } else {
            Cell::new("No").fg(Color::Red)
        };

        let desc = if skill.description.len() > 40 {
            format!("{}...", &skill.description[..37])
        } else {
            skill.description.clone()
        };

        table.add_row(vec![
            Cell::new(&skill.name),
            Cell::new(desc),
            Cell::new(source),
            Cell::new(bins),
            Cell::new(env),
            eligible_cell,
        ]);
    }

    println!("{table}");
    Ok(())
}

/// Check skill requirements
async fn check_skill(name: &str, skills_dir: Option<String>) -> anyhow::Result<()> {
    let config = build_config(skills_dir);
    let registry = SkillRegistry::new(config);
    registry.load().await?;

    let check = registry.check_skill(name).await?;

    if check.eligible {
        println!("{} Skill '{}' requirements met!", "✓".green(), name.bold());
    } else {
        println!("{} Skill '{}' requirements NOT met:", "✗".red(), name.bold());
        println!();

        if !check.missing_bins.is_empty() {
            println!("  {} Missing binaries:", "→".yellow());
            for bin in &check.missing_bins {
                println!("      - {}", bin);
            }
        }

        if !check.missing_any_bins.is_empty() {
            println!("  {} Need one of:", "→".yellow());
            for bin in &check.missing_any_bins {
                println!("      - {}", bin);
            }
        }

        if !check.missing_env.is_empty() {
            println!("  {} Missing env vars:", "→".yellow());
            for var in &check.missing_env {
                println!("      - {}", var);
            }
        }

        if !check.missing_config.is_empty() {
            println!("  {} Missing configs:", "→".yellow());
            for cfg in &check.missing_config {
                println!("      - {}", cfg);
            }
        }

        if let Some(ref os) = check.os_mismatch {
            println!("  {} OS mismatch: {}", "→".yellow(), os);
        }
    }

    Ok(())
}

/// Show skill content
async fn show_skill(name: &str, skills_dir: Option<String>) -> anyhow::Result<()> {
    let config = build_config(skills_dir);
    let registry = SkillRegistry::new(config);
    registry.load().await?;

    let skill = registry
        .get(name)
        .await
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found", name))?;

    // Print header
    let emoji = skill.metadata.emoji.as_deref().unwrap_or("📋");
    println!("{} {} {}", emoji, skill.name.bold(), skill.description.dimmed());
    println!();

    if !skill.metadata.tags.is_empty() {
        println!("{}: {}", "Tags".cyan(), skill.metadata.tags.join(", "));
    }

    if let Some(ref homepage) = skill.homepage {
        println!("{}: {}", "Homepage".cyan(), homepage);
    }

    if !skill.metadata.requires.bins.is_empty() {
        println!("{}: {}", "Requires".cyan(), skill.metadata.requires.bins.join(", "));
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!();

    // Print content
    println!("{}", skill.content);

    Ok(())
}

/// Generate prompt injection for skills
async fn generate_prompt(skills_arg: &str, skills_dir: Option<String>) -> anyhow::Result<()> {
    let config = build_config(skills_dir);
    let registry = SkillRegistry::new(config);
    registry.load().await?;

    let skills = if skills_arg == "all" {
        registry.eligible().await
    } else {
        let mut selected = Vec::new();
        for name in skills_arg.split(',') {
            let name = name.trim();
            if let Some(skill) = registry.get(name).await {
                selected.push(skill);
            } else {
                eprintln!("{}: Skill '{}' not found", "Warning".yellow(), name);
            }
        }
        selected
    };

    if skills.is_empty() {
        eprintln!("{}", "No skills found to include in prompt.".yellow());
        return Ok(());
    }

    let prompt = build_skills_prompt(&skills);
    println!("{}", prompt);

    Ok(())
}

/// Search skills by query
async fn search_skills(query: &str, skills_dir: Option<String>) -> anyhow::Result<()> {
    let config = build_config(skills_dir);
    let registry = SkillRegistry::new(config);
    registry.load().await?;

    let results = registry.search(query).await;

    if results.is_empty() {
        println!("{}", "No skills matching query.".yellow());
        return Ok(());
    }

    println!("{} results for '{}':\n", results.len(), query.bold());

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["SKILL", "SCORE", "MATCHES", "DESCRIPTION"]);

    for result in results {
        let score = format!("{:.2}", result.score);
        let matches = result.matches.join(", ");
        let desc = if result.skill.description.len() > 35 {
            format!("{}...", &result.skill.description[..32])
        } else {
            result.skill.description.clone()
        };

        table.add_row(vec![
            Cell::new(&result.skill.name),
            Cell::new(score),
            Cell::new(matches),
            Cell::new(desc),
        ]);
    }

    println!("{table}");

    Ok(())
}
