/// Example demonstrating HTTP transport usage for MCP
///
/// This example shows how to:
/// 1. Create an HTTP transport with custom configuration
/// 2. Initialize the transport
/// 3. Send JSON-RPC requests over HTTP
/// 4. Handle responses and errors
/// 5. Properly shutdown the transport
///
/// Run with: cargo run --example http_transport_example --features http

use aof_mcp::transport::{http::HttpTransport, McpRequest, McpTransport};
use serde_json::json;
use std::time::Duration;

#[cfg(feature = "http")]
use aof_mcp::transport::http::HttpConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== HTTP Transport Example ===\n");

    // Example 1: Basic HTTP transport with default configuration
    println!("1. Creating HTTP transport with default config...");
    let mut transport = HttpTransport::new("http://localhost:8080/mcp");

    // Initialize the transport (validates endpoint and builds HTTP client)
    transport.init().await?;
    println!("   ✓ Transport initialized\n");

    // Example 2: HTTP transport with custom configuration
    println!("2. Creating HTTP transport with custom config...");
    let custom_config = HttpConfig {
        timeout: Duration::from_secs(60),
        pool_max_idle_per_host: 20,
        pool_idle_timeout: Duration::from_secs(120),
        http2_prior_knowledge: false,
    };

    let mut custom_transport = HttpTransport::with_config(
        "https://api.example.com/mcp",
        custom_config
    );
    custom_transport.init().await?;
    println!("   ✓ Custom transport initialized\n");

    // Example 3: Sending a request
    println!("3. Sending MCP request...");
    let request = McpRequest::new(
        "tools/list",
        json!({
            "category": "file_operations"
        })
    );

    println!("   Request: method={}, id={}", request.method, request.id);

    // Note: This will fail unless you have an actual MCP server running
    // In production, handle the response:
    // let response = transport.request(&request).await?;
    // println!("   Response: {:?}", response);

    // Example 4: Error handling
    println!("\n4. Error handling examples:");

    // Invalid endpoint - will fail on init
    let mut invalid_transport = HttpTransport::new("ftp://invalid.com");
    match invalid_transport.init().await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Invalid scheme detected: {}", e),
    }

    // Invalid URL - will fail on init
    let mut malformed_transport = HttpTransport::new("not a url");
    match malformed_transport.init().await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Invalid URL detected: {}", e),
    }

    // Example 5: Shutdown
    println!("\n5. Shutting down transports...");
    transport.shutdown().await?;
    custom_transport.shutdown().await?;
    println!("   ✓ All transports shutdown successfully\n");

    println!("=== Example Complete ===");

    Ok(())
}

#[cfg(not(feature = "http"))]
fn main() {
    eprintln!("This example requires the 'http' feature to be enabled.");
    eprintln!("Run with: cargo run --example http_transport_example --features http");
    std::process::exit(1);
}
