use anyhow::{Context, Result};
use aof_mcp::McpClientBuilder;
use tracing::{info, warn};

/// List available MCP tools from a server
pub async fn execute(server: &str, args: &[String]) -> Result<()> {
    info!("Connecting to MCP server: {}", server);
    info!("Server args: {:?}", args);

    // Create MCP client with stdio transport
    let client = McpClientBuilder::new()
        .stdio(server, args.to_vec())
        .build()
        .context("Failed to create MCP client")?;

    // Initialize connection and list tools
    client
        .initialize()
        .await
        .context("Failed to initialize MCP connection")?;

    let tools = client
        .list_tools()
        .await
        .context("Failed to list tools from MCP server")?;

    if tools.is_empty() {
        warn!("No tools available from server");
        println!("No tools available");
        return Ok(());
    }

    println!("\n Available MCP Tools ({}):\n", tools.len());
    println!("{}", "=".repeat(80));

    for tool in tools {
        println!("\n Tool: {}", tool.name);
        println!(" Description: {}", tool.description);

        if let Some(params) = tool.parameters.as_object() {
            if !params.is_empty() {
                println!(" Parameters:");
                if let Some(properties) = params.get("properties").and_then(|p| p.as_object()) {
                    for (param_name, param_info) in properties {
                        let param_type = param_info
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown");
                        let description = param_info
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");

                        println!("   - {} ({}): {}", param_name, param_type, description);
                    }
                }
            }
        }
        println!("{}", "-".repeat(80));
    }

    // Cleanup
    client.shutdown().await.ok();

    Ok(())
}
