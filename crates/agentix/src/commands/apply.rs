//! `agentix apply -f <file>` — validate and push an agent YAML to the gateway.

use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use agentix_core::{DirectoryLoader, FlatYamlLoader};

use crate::cli::CliContext;
use crate::client::{GatewayClient, is_gateway_unreachable};

pub async fn run(cli: &CliContext, file: String) -> Result<()> {
    let path = Path::new(&file);

    if !path.exists() {
        anyhow::bail!("Path not found: {}", file);
    }

    if path.is_dir() {
        // Agent directory format — validate locally, instruct user to use gateway start
        match DirectoryLoader::load(path) {
            Ok(def) => {
                let name = def.name.clone();
                let desc = def.description.clone().unwrap_or_default();
                let msg = format!("Agent directory '{}' is valid", file);
                println!("{}", if colored::control::SHOULD_COLORIZE.should_colorize() { msg.green().to_string() } else { msg });
                println!("  Name: {}", name);
                if !desc.is_empty() {
                    println!("  Description: {}", desc);
                }
                println!();
                println!("Agent directory validated locally.");
                println!(
                    "To register it with the gateway, add its parent directory to your agentix.yaml `spec.agents_dir`"
                );
                println!("and run: agentix gateway start");
            }
            Err(e) => {
                print_validation_error(&file, &e.to_string(), None);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Flat YAML file — validate locally then send to gateway
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", file, e))?;

    // Validate locally first
    match FlatYamlLoader::load_from_str(&content) {
        Ok(def) => {
            let agent_name = def.name.clone();

            // Now send to gateway
            let client = GatewayClient::new(&cli.gateway_url);

            let (status, body) = client
                .register_agent(&content)
                .await
                .map_err(|e| {
                    if let Some(url) = is_gateway_unreachable(&e) {
                        anyhow::anyhow!(
                            "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                            url
                        )
                    } else {
                        e
                    }
                })?;

            if status == 409 {
                // Conflict — update existing agent
                client
                    .update_agent(&agent_name, &content)
                    .await
                    .map_err(|e| {
                        if let Some(url) = is_gateway_unreachable(&e) {
                            anyhow::anyhow!(
                                "Cannot connect to gateway at {}\nStart the gateway with: agentix gateway start",
                                url
                            )
                        } else {
                            e
                        }
                    })?;

                let msg = format!("Agent '{}' updated in gateway at {}", agent_name, cli.gateway_url);
                println!("{}", if colored::control::SHOULD_COLORIZE.should_colorize() { msg.green().to_string() } else { msg });
            } else if status >= 200 && status < 300 {
                let name = body["name"].as_str().unwrap_or(&agent_name);
                let msg = format!("Agent '{}' applied to gateway at {}", name, cli.gateway_url);
                println!("{}", if colored::control::SHOULD_COLORIZE.should_colorize() { msg.green().to_string() } else { msg });
            } else {
                let err = body["error"].as_str().unwrap_or("unknown error");
                anyhow::bail!("Gateway returned {}: {}", status, err);
            }
        }
        Err(e) => {
            print_validation_error(&file, &e.to_string(), Some(&content));
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print a rustc-style validation error.
fn print_validation_error(file: &str, error: &str, content: Option<&str>) {
    eprintln!("error: invalid agent YAML");
    eprintln!("  --> {}", file);

    if let Some(text) = content {
        // Try to find the relevant line
        if let Some(line_info) = find_error_line(text, error) {
            eprintln!("   |");
            eprintln!("{:>3} | {}", line_info.line_num, line_info.line_text);
            eprintln!("   | {}", "^".repeat(line_info.col_width));
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
    // Look for a field name mentioned in the error, then find it in the YAML text
    // Simple heuristic: look for "field:" patterns in the error message
    let field = extract_field_name(error)?;
    for (idx, line) in content.lines().enumerate() {
        if line.contains(&format!("{}:", field)) || line.contains(&format!("{} :", field)) {
            let col_width = line.trim_start().len().min(field.len() + 1);
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
    // Patterns like "field 'name'" or "Field: name" or "missing field `name`"
    for word in error.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if !clean.is_empty() && clean.len() > 2 && clean != "the" && clean != "and" {
            return Some(clean.to_string());
        }
    }
    None
}
