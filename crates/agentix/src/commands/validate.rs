//! `agentix validate <path>` — validate agent YAML or agent directory without connecting to gateway.

use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use agentix_core::{DirectoryLoader, FlatYamlLoader};

pub async fn run(path: String) -> Result<()> {
    let p = Path::new(&path);

    if !p.exists() {
        eprintln!("error: path not found: {}", path);
        std::process::exit(1);
    }

    if p.is_dir() {
        validate_directory(p, &path);
    } else {
        // If user passes `<dir>/agent.yaml`, treat the parent directory as the agent
        let is_agent_manifest = p.file_name().map_or(false, |f| f == "agent.yaml");
        if is_agent_manifest {
            if let Some(parent) = p.parent() {
                if parent.join("SOUL.md").exists() {
                    // This is an agent directory's manifest — validate the directory
                    let dir_path = parent.to_string_lossy().to_string();
                    validate_directory(parent, &dir_path);
                    return Ok(());
                }
            }
        }
        validate_flat_yaml(p, &path)?;
    }

    Ok(())
}

fn validate_directory(path: &Path, display_path: &str) {
    match DirectoryLoader::load(path) {
        Ok(def) => {
            let name = def.name.clone();
            let desc = def.description.clone().unwrap_or_default();
            let msg = format!("Agent directory '{}' is valid", display_path);
            println!(
                "{}",
                if colored::control::SHOULD_COLORIZE.should_colorize() {
                    msg.green().to_string()
                } else {
                    msg
                }
            );
            println!("  Name: {}", name);
            if !desc.is_empty() {
                println!("  Description: {}", desc);
            }
        }
        Err(e) => {
            print_validation_error(display_path, &e.to_string(), None);
            std::process::exit(1);
        }
    }
}

fn validate_flat_yaml(path: &Path, display_path: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", display_path, e))?;

    match FlatYamlLoader::load_from_str(&content) {
        Ok(def) => {
            let name = def.name.clone();
            let msg = format!("Agent '{}' is valid", name);
            println!(
                "{}",
                if colored::control::SHOULD_COLORIZE.should_colorize() {
                    msg.green().to_string()
                } else {
                    msg
                }
            );
        }
        Err(e) => {
            print_validation_error(display_path, &e.to_string(), Some(&content));
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print a rustc-style validation error pointing to the offending line.
fn print_validation_error(file: &str, error: &str, content: Option<&str>) {
    eprintln!("error: invalid agent YAML");
    eprintln!("  --> {}", file);

    if let Some(text) = content {
        if let Some(line_info) = find_error_line(text, error) {
            eprintln!("   |");
            eprintln!("{:>3} | {}", line_info.line_num, line_info.line_text);
            eprintln!("   | {}", "^".repeat(line_info.col_width.max(1)));
            eprintln!("   | {}", error);
        } else {
            eprintln!("   |");
            eprintln!("   = {}", error);
        }
    } else {
        eprintln!("   |");
        eprintln!("   = {}", error);
    }

    eprintln!("   |");
    eprintln!("   = note: see https://docs.openagentix.org/spec/agent-yaml-v1");
}

struct ErrorLine {
    line_num: usize,
    line_text: String,
    col_width: usize,
}

fn find_error_line(content: &str, error: &str) -> Option<ErrorLine> {
    let field = extract_field_name(error)?;
    for (idx, line) in content.lines().enumerate() {
        if line.contains(&format!("{}:", field)) || line.contains(&format!("{} :", field)) {
            let col_width = line.len().saturating_sub(line.trim_start().len()) + field.len() + 1;
            return Some(ErrorLine {
                line_num: idx + 1,
                line_text: line.to_string(),
                col_width,
            });
        }
    }
    None
}

fn extract_field_name(error: &str) -> Option<String> {
    // Patterns: "missing field `name`", "Field: name", "field 'max_iterations'"
    if let Some(start) = error.find('`') {
        if let Some(end) = error[start + 1..].find('`') {
            let field = &error[start + 1..start + 1 + end];
            if !field.is_empty() {
                return Some(field.to_string());
            }
        }
    }
    if let Some(start) = error.find('\'') {
        if let Some(end) = error[start + 1..].find('\'') {
            let field = &error[start + 1..start + 1 + end];
            if !field.is_empty() && field.len() < 40 {
                return Some(field.to_string());
            }
        }
    }
    None
}
