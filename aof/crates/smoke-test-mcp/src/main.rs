//! Smoke Test MCP Server - Minimal MCP implementation for testing
//!
//! This server implements the MCP protocol and provides two simple tools:
//! - echo: Returns the input string (for basic connectivity testing)
//! - add: Adds two numbers together (for parameter passing testing)
//!
//! Run with: cargo run --release --bin smoke-test-mcp
//! Or from aof runtime: npx ./smoke-test-mcp (after building)

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{debug, info};

const MCP_VERSION: &str = "2024-11-05";

#[tokio::main]
async fn main() {
    // Initialize logging to stderr
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(io::stderr)
        .init();

    info!("Starting Smoke Test MCP Server");

    let mut id_counter = 1;
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut lines = reader.lines();

    // Main request loop
    loop {
        if let Some(Ok(line)) = lines.next() {
            if line.is_empty() {
                continue;
            }

            debug!("Received: {}", line);

            match serde_json::from_str::<Value>(&line) {
                Ok(request) => {
                    if let Some(method) = request.get("method").and_then(|m| m.as_str()) {
                        let req_id = request.get("id").and_then(|id| id.as_i64()).unwrap_or(id_counter as i64);
                        id_counter += 1;

                        let response = match method {
                            "initialize" => handle_initialize(&request),
                            "tools/list" => handle_list_tools(),
                            "tools/call" => handle_tool_call(&request),
                            _ => {
                                json!({
                                    "id": req_id,
                                    "error": {
                                        "code": -32601,
                                        "message": format!("Method not found: {}", method)
                                    }
                                })
                            }
                        };

                        if let Ok(json_str) = serde_json::to_string(&response) {
                            println!("{}", json_str);
                            let _ = io::stdout().flush();
                        }
                    }
                }
                Err(e) => {
                    let error_response = json!({
                        "id": id_counter,
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        }
                    });
                    println!("{}", serde_json::to_string(&error_response).unwrap_or_default());
                    let _ = io::stdout().flush();
                    id_counter += 1;
                }
            }
        }
    }
}

/// Handle initialize request
fn handle_initialize(request: &Value) -> Value {
    let req_id = request.get("id").and_then(|id| id.as_i64()).unwrap_or(0);

    info!("Received initialize request");

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "protocolVersion": MCP_VERSION,
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "smoke-test-mcp",
                "version": "0.1.0"
            }
        }
    })
}

/// List available tools
fn handle_list_tools() -> Value {
    info!("Listing available tools");

    json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": [
                {
                    "name": "echo",
                    "description": "Echo the input string - tests basic connectivity",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "The message to echo"
                            }
                        },
                        "required": ["message"]
                    }
                },
                {
                    "name": "add",
                    "description": "Add two numbers together",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "a": {
                                "type": "number",
                                "description": "First number"
                            },
                            "b": {
                                "type": "number",
                                "description": "Second number"
                            }
                        },
                        "required": ["a", "b"]
                    }
                },
                {
                    "name": "get_system_info",
                    "description": "Get basic system information",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        }
    })
}

/// Handle tool call
fn handle_tool_call(request: &Value) -> Value {
    let req_id = request.get("id").and_then(|id| id.as_i64()).unwrap_or(0);

    let tool_name = request
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let arguments = request
        .get("params")
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or(json!({}));

    debug!("Calling tool: {} with args: {:?}", tool_name, arguments);

    let result = match tool_name {
        "echo" => {
            let message = arguments
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("(no message)");

            info!("Echo tool called with: {}", message);

            json!({
                "message": message,
                "timestamp": chrono::Local::now().to_rfc3339()
            })
        }
        "add" => {
            let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let sum = a + b;

            info!("Add tool called: {} + {} = {}", a, b, sum);

            json!({
                "result": sum,
                "inputs": { "a": a, "b": b }
            })
        }
        "get_system_info" => {
            info!("System info tool called");

            json!({
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "server": "smoke-test-mcp",
                "version": "0.1.0",
                "timestamp": chrono::Local::now().to_rfc3339()
            })
        }
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {
                    "code": -32601,
                    "message": format!("Tool not found: {}", tool_name)
                }
            });
        }
    };

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": result
    })
}
