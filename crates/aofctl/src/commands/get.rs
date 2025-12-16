use anyhow::Result;
use crate::resources::ResourceType;

/// Get resource status (kubectl-style: get <resource-type> [name])
pub async fn execute(
    resource_type: &str,
    name: Option<&str>,
    output: &str,
    all_namespaces: bool,
) -> Result<()> {
    // Parse resource type
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    // Build resource list (placeholder until persistent storage is implemented)
    let resources = get_mock_resources(&rt, name, all_namespaces);

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
            print_table_header(&rt, all_namespaces);
            for resource in resources {
                print_table_row(&rt, &resource, all_namespaces);
            }
        }
    }

    Ok(())
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
                println!("\nNAMESPACE    NAME              STATUS    MODEL              AGE");
                println!("{}", "=".repeat(75));
            } else {
                println!("\nNAME              STATUS    MODEL              AGE");
                println!("{}", "=".repeat(60));
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

    let age = "5m"; // Placeholder

    match rt {
        ResourceType::Agent => {
            if all_namespaces {
                println!("{:<12} {:<16} {:<9} {:<18} {}", namespace, name, status, "claude-sonnet-4", age);
            } else {
                println!("{:<16} {:<9} {:<18} {}", name, status, "claude-sonnet-4", age);
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
