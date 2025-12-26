use anyhow::Result;
use crate::resources::ResourceType;

/// Get resource status (kubectl-style: get <resource-type> [name])
pub async fn execute(
    resource_type: &str,
    name: Option<&str>,
    output: &str,
    all_namespaces: bool,
    library: bool,
) -> Result<()> {
    // Parse resource type
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    // Build resource list - either from library or mock data
    let resources = if library {
        get_library_resources(&rt, name)?
    } else {
        get_mock_resources(&rt, name, all_namespaces)
    };

    // When listing library, always show domains (namespaces)
    let show_namespaces = all_namespaces || library;

    // Format and display output
    match output {
        "json" => {
            let output = serde_json::json!({
                "apiVersion": rt.api_version(),
                "kind": format!("{}List", rt.kind()),
                "items": resources
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            let output = serde_json::json!({
                "apiVersion": rt.api_version(),
                "kind": format!("{}List", rt.kind()),
                "items": resources
            });
            println!("{}", serde_yaml::to_string(&output)?);
        }
        "name" => {
            // Just print resource names
            for resource in resources {
                if let Some(resource_name) = resource.get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str()) {
                    println!("{}/{}", rt, resource_name);
                }
            }
        }
        "wide" | _ => {
            // Table format (default)
            print_table_header(&rt, show_namespaces);
            for resource in resources {
                print_table_row(&rt, &resource, show_namespaces);
            }
        }
    }

    Ok(())
}

/// Get resources from the built-in library directory
fn get_library_resources(
    rt: &ResourceType,
    name: Option<&str>,
) -> Result<Vec<serde_json::Value>> {
    // Only agents are in the library currently
    if !matches!(rt, ResourceType::Agent) {
        return Ok(vec![]);
    }

    // Find the library directory - check common locations
    let library_path = find_library_path()?;
    let mut resources = Vec::new();

    // Scan all subdirectories (domains) for agent YAML files
    if let Ok(entries) = std::fs::read_dir(&library_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let domain = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Scan for YAML files in each domain
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.flatten() {
                        let file_path = file.path();
                        if file_path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                            if let Ok(content) = std::fs::read_to_string(&file_path) {
                                if let Ok(agent) = serde_yaml::from_str::<serde_json::Value>(&content) {
                                    let agent_name = agent
                                        .get("metadata")
                                        .and_then(|m| m.get("name"))
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");

                                    // Filter by name if provided
                                    if let Some(filter) = name {
                                        if agent_name != filter {
                                            continue;
                                        }
                                    }

                                    // Add domain to metadata
                                    let mut enriched = agent.clone();
                                    if let Some(metadata) = enriched.get_mut("metadata") {
                                        if let Some(obj) = metadata.as_object_mut() {
                                            obj.insert("namespace".to_string(), serde_json::json!(domain));
                                        }
                                    }

                                    // Add status for display
                                    if let Some(obj) = enriched.as_object_mut() {
                                        obj.insert("status".to_string(), serde_json::json!({
                                            "phase": "Available"
                                        }));
                                    }

                                    resources.push(enriched);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(resources)
}

/// Find the library directory - check multiple possible locations
fn find_library_path() -> Result<std::path::PathBuf> {
    // Check common locations
    let candidates = [
        std::path::PathBuf::from("library"),
        std::path::PathBuf::from("./library"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("library")))
            .unwrap_or_default(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().and_then(|p| p.parent()).map(|p| p.join("library")))
            .unwrap_or_default(),
    ];

    for candidate in candidates {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err(anyhow::anyhow!(
        "Library directory not found. Make sure you're running from the project root or the library is installed."
    ))
}

fn get_mock_resources(
    rt: &ResourceType,
    name: Option<&str>,
    _all_namespaces: bool,
) -> Vec<serde_json::Value> {
    // If a specific name is requested, return only that resource
    if let Some(n) = name {
        return vec![create_mock_resource(rt, n, "default")];
    }

    // Return a few mock resources for demonstration
    match rt {
        ResourceType::Agent => {
            vec![
                create_mock_resource(rt, "researcher-agent", "default"),
                create_mock_resource(rt, "coder-agent", "default"),
                create_mock_resource(rt, "reviewer-agent", "default"),
            ]
        }
        ResourceType::Workflow => {
            vec![
                create_mock_resource(rt, "data-pipeline", "default"),
                create_mock_resource(rt, "review-cycle", "default"),
            ]
        }
        _ => {
            vec![create_mock_resource(rt, "example", "default")]
        }
    }
}

fn create_mock_resource(rt: &ResourceType, name: &str, namespace: &str) -> serde_json::Value {
    serde_json::json!({
        "apiVersion": rt.api_version(),
        "kind": rt.kind(),
        "metadata": {
            "name": name,
            "namespace": namespace,
            "creationTimestamp": "2025-12-11T14:49:02Z",
            "labels": {
                "app": name.split('-').next().unwrap_or(name)
            }
        },
        "status": {
            "phase": "Running",
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "lastTransitionTime": "2025-12-11T14:49:02Z"
                }
            ]
        }
    })
}

fn print_table_header(rt: &ResourceType, all_namespaces: bool) {
    match rt {
        ResourceType::Agent => {
            if all_namespaces {
                println!("\n{:<14} {:<24} {:<11} {:<24} {}", "DOMAIN", "NAME", "STATUS", "MODEL", "AGE");
                println!("{}", "=".repeat(80));
            } else {
                println!("\n{:<24} {:<11} {:<24} {}", "NAME", "STATUS", "MODEL", "AGE");
                println!("{}", "=".repeat(65));
            }
        }
        ResourceType::Workflow => {
            if all_namespaces {
                println!("\nNAMESPACE    NAME              STATUS    STEPS    AGE");
                println!("{}", "=".repeat(65));
            } else {
                println!("\nNAME              STATUS    STEPS    AGE");
                println!("{}", "=".repeat(50));
            }
        }
        ResourceType::Tool => {
            if all_namespaces {
                println!("\nNAMESPACE    NAME              TYPE      SERVER         AGE");
                println!("{}", "=".repeat(70));
            } else {
                println!("\nNAME              TYPE      SERVER         AGE");
                println!("{}", "=".repeat(55));
            }
        }
        _ => {
            if all_namespaces {
                println!("\nNAMESPACE    NAME              STATUS    AGE");
                println!("{}", "=".repeat(60));
            } else {
                println!("\nNAME              STATUS    AGE");
                println!("{}", "=".repeat(45));
            }
        }
    }
}

fn print_table_row(rt: &ResourceType, resource: &serde_json::Value, all_namespaces: bool) {
    let name = resource
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let namespace = resource
        .get("metadata")
        .and_then(|m| m.get("namespace"))
        .and_then(|n| n.as_str())
        .unwrap_or("default");

    let status = resource
        .get("status")
        .and_then(|s| s.get("phase"))
        .and_then(|p| p.as_str())
        .unwrap_or("Unknown");

    // Get model from spec if available (for library agents)
    let model = resource
        .get("spec")
        .and_then(|s| s.get("model"))
        .and_then(|m| m.as_str())
        .unwrap_or("claude-sonnet-4");

    // Truncate model name if too long
    let model_display = if model.len() > 24 {
        format!("{}...", &model[..21])
    } else {
        model.to_string()
    };

    let age = "-"; // Library agents don't have age

    match rt {
        ResourceType::Agent => {
            if all_namespaces {
                println!("{:<14} {:<24} {:<11} {:<24} {}", namespace, name, status, model_display, age);
            } else {
                println!("{:<24} {:<11} {:<24} {}", name, status, model_display, age);
            }
        }
        ResourceType::Workflow => {
            if all_namespaces {
                println!("{:<12} {:<16} {:<9} {:<8} {}", namespace, name, status, "3/5", age);
            } else {
                println!("{:<16} {:<9} {:<8} {}", name, status, "3/5", age);
            }
        }
        ResourceType::Tool => {
            if all_namespaces {
                println!("{:<12} {:<16} {:<9} {:<14} {}", namespace, name, "MCP", "claude-flow", age);
            } else {
                println!("{:<16} {:<9} {:<14} {}", name, "MCP", "claude-flow", age);
            }
        }
        _ => {
            if all_namespaces {
                println!("{:<12} {:<16} {:<9} {}", namespace, name, status, age);
            } else {
                println!("{:<16} {:<9} {}", name, status, age);
            }
        }
    }
}
