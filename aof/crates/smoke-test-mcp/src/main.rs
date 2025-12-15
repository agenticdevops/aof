//! Comprehensive MCP Test Server - Full MCP implementation for testing
//!
//! This server implements the MCP protocol and provides tools for testing:
//! - echo: Returns the input string (basic connectivity)
//! - add: Adds two numbers (parameter passing)
//! - get_system_info: System information
//! - kv_set/kv_get/kv_list: Key-value store operations
//! - delay: Simulates slow operations (timeout testing)
//! - error_trigger: Triggers various error types
//! - batch_process: Processes multiple items
//! - file_read/file_write: Simulated file operations
//!
//! Run with: cargo run --release --bin smoke-test-mcp

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

const MCP_VERSION: &str = "2024-11-05";

/// In-memory key-value store for testing
type KvStore = Arc<RwLock<HashMap<String, Value>>>;

/// Simulated file system
type FileSystem = Arc<RwLock<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    // Initialize logging to stderr
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(io::stderr)
        .init();

    info!("Starting Comprehensive MCP Test Server v0.2.0");

    // Initialize stores
    let kv_store: KvStore = Arc::new(RwLock::new(HashMap::new()));
    let file_system: FileSystem = Arc::new(RwLock::new(HashMap::new()));

    // Pre-populate some test data
    {
        let mut fs = file_system.write().unwrap();
        fs.insert("/tmp/test.txt".to_string(), "Hello from MCP test server!".to_string());
        fs.insert("/tmp/config.json".to_string(), r#"{"name":"test","version":"1.0"}"#.to_string());
    }

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
                        let req_id = request.get("id").cloned().unwrap_or(json!(id_counter));
                        id_counter += 1;

                        let response = match method {
                            "initialize" => handle_initialize(&request, &req_id),
                            "tools/list" => handle_list_tools(&req_id),
                            "tools/call" => handle_tool_call(&request, &req_id, &kv_store, &file_system).await,
                            "notifications/initialized" => {
                                // Client notification, no response needed
                                continue;
                            }
                            _ => {
                                json!({
                                    "jsonrpc": "2.0",
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
                        "jsonrpc": "2.0",
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
fn handle_initialize(request: &Value, req_id: &Value) -> Value {
    let client_info = request
        .get("params")
        .and_then(|p| p.get("clientInfo"))
        .cloned()
        .unwrap_or(json!({}));

    info!("Received initialize request from client: {:?}", client_info);

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "protocolVersion": MCP_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "smoke-test-mcp",
                "version": "0.2.0"
            }
        }
    })
}

/// List available tools - comprehensive test suite
fn handle_list_tools(req_id: &Value) -> Value {
    info!("Listing available tools");

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "tools": [
                // Basic connectivity test
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
                // Math operation
                {
                    "name": "add",
                    "description": "Add two numbers together",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "a": { "type": "number", "description": "First number" },
                            "b": { "type": "number", "description": "Second number" }
                        },
                        "required": ["a", "b"]
                    }
                },
                // System info
                {
                    "name": "get_system_info",
                    "description": "Get basic system information",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                // Key-value store operations
                {
                    "name": "kv_set",
                    "description": "Set a key-value pair in the in-memory store",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": { "type": "string", "description": "The key to set" },
                            "value": { "description": "The value to store (any JSON)" }
                        },
                        "required": ["key", "value"]
                    }
                },
                {
                    "name": "kv_get",
                    "description": "Get a value from the in-memory store",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": { "type": "string", "description": "The key to retrieve" }
                        },
                        "required": ["key"]
                    }
                },
                {
                    "name": "kv_list",
                    "description": "List all keys in the in-memory store",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prefix": { "type": "string", "description": "Optional key prefix filter" }
                        }
                    }
                },
                {
                    "name": "kv_delete",
                    "description": "Delete a key from the in-memory store",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": { "type": "string", "description": "The key to delete" }
                        },
                        "required": ["key"]
                    }
                },
                // Simulated file operations
                {
                    "name": "file_read",
                    "description": "Read contents of a simulated file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to read" }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "file_write",
                    "description": "Write contents to a simulated file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to write" },
                            "content": { "type": "string", "description": "Content to write" }
                        },
                        "required": ["path", "content"]
                    }
                },
                {
                    "name": "file_list",
                    "description": "List all simulated files",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                // Delay tool for timeout testing
                {
                    "name": "delay",
                    "description": "Wait for specified milliseconds (for timeout testing)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "ms": { "type": "integer", "description": "Milliseconds to wait" }
                        },
                        "required": ["ms"]
                    }
                },
                // Error simulation
                {
                    "name": "error_trigger",
                    "description": "Trigger a specific error type for testing error handling",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "error_type": {
                                "type": "string",
                                "enum": ["not_found", "invalid_input", "internal", "permission", "timeout"],
                                "description": "Type of error to trigger"
                            }
                        },
                        "required": ["error_type"]
                    }
                },
                // Batch processing
                {
                    "name": "batch_process",
                    "description": "Process multiple items in batch",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "items": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Array of items to process"
                            },
                            "operation": {
                                "type": "string",
                                "enum": ["uppercase", "lowercase", "reverse", "length"],
                                "description": "Operation to perform on each item"
                            }
                        },
                        "required": ["items", "operation"]
                    }
                },
                // JSON transform
                {
                    "name": "json_transform",
                    "description": "Transform JSON data",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "data": { "description": "JSON data to transform" },
                            "operation": {
                                "type": "string",
                                "enum": ["flatten", "keys", "values", "stringify", "pretty"],
                                "description": "Transformation operation"
                            }
                        },
                        "required": ["data", "operation"]
                    }
                }
            ]
        }
    })
}

/// Handle tool call
async fn handle_tool_call(request: &Value, req_id: &Value, kv_store: &KvStore, file_system: &FileSystem) -> Value {
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
        "echo" => tool_echo(&arguments),
        "add" => tool_add(&arguments),
        "get_system_info" => tool_system_info(),
        "kv_set" => tool_kv_set(&arguments, kv_store),
        "kv_get" => tool_kv_get(&arguments, kv_store),
        "kv_list" => tool_kv_list(&arguments, kv_store),
        "kv_delete" => tool_kv_delete(&arguments, kv_store),
        "file_read" => tool_file_read(&arguments, file_system),
        "file_write" => tool_file_write(&arguments, file_system),
        "file_list" => tool_file_list(file_system),
        "delay" => tool_delay(&arguments).await,
        "error_trigger" => tool_error_trigger(&arguments, req_id),
        "batch_process" => tool_batch_process(&arguments),
        "json_transform" => tool_json_transform(&arguments),
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

    // Check if result is an error
    if result.get("error").is_some() {
        return result;
    }

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "content": [{
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }]
        }
    })
}

// === Tool Implementations ===

fn tool_echo(args: &Value) -> Value {
    let message = args.get("message").and_then(|m| m.as_str()).unwrap_or("(no message)");
    info!("Echo tool called with: {}", message);
    json!({
        "message": message,
        "timestamp": chrono::Local::now().to_rfc3339()
    })
}

fn tool_add(args: &Value) -> Value {
    let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let sum = a + b;
    info!("Add tool called: {} + {} = {}", a, b, sum);
    json!({
        "result": sum,
        "inputs": { "a": a, "b": b },
        "operation": "addition"
    })
}

fn tool_system_info() -> Value {
    info!("System info tool called");
    json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "server": "smoke-test-mcp",
        "version": "0.2.0",
        "features": ["echo", "math", "kv_store", "file_sim", "delay", "errors", "batch"],
        "timestamp": chrono::Local::now().to_rfc3339()
    })
}

fn tool_kv_set(args: &Value, store: &KvStore) -> Value {
    let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("");
    let value = args.get("value").cloned().unwrap_or(json!(null));

    if key.is_empty() {
        return json!({"error": "Key cannot be empty"});
    }

    let mut store = store.write().unwrap();
    store.insert(key.to_string(), value.clone());
    info!("KV set: {} = {:?}", key, value);
    json!({
        "success": true,
        "key": key,
        "value": value
    })
}

fn tool_kv_get(args: &Value, store: &KvStore) -> Value {
    let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("");
    let store = store.read().unwrap();

    if let Some(value) = store.get(key) {
        info!("KV get: {} = {:?}", key, value);
        json!({
            "found": true,
            "key": key,
            "value": value
        })
    } else {
        json!({
            "found": false,
            "key": key,
            "value": null
        })
    }
}

fn tool_kv_list(args: &Value, store: &KvStore) -> Value {
    let prefix = args.get("prefix").and_then(|p| p.as_str()).unwrap_or("");
    let store = store.read().unwrap();

    let keys: Vec<&String> = store.keys()
        .filter(|k| k.starts_with(prefix))
        .collect();

    info!("KV list: {} keys (prefix: '{}')", keys.len(), prefix);
    json!({
        "keys": keys,
        "count": keys.len()
    })
}

fn tool_kv_delete(args: &Value, store: &KvStore) -> Value {
    let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("");
    let mut store = store.write().unwrap();

    let removed = store.remove(key).is_some();
    info!("KV delete: {} (removed: {})", key, removed);
    json!({
        "deleted": removed,
        "key": key
    })
}

fn tool_file_read(args: &Value, fs: &FileSystem) -> Value {
    let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
    let fs = fs.read().unwrap();

    if let Some(content) = fs.get(path) {
        info!("File read: {} ({} bytes)", path, content.len());
        json!({
            "success": true,
            "path": path,
            "content": content,
            "size": content.len()
        })
    } else {
        warn!("File not found: {}", path);
        json!({
            "success": false,
            "path": path,
            "error": "File not found"
        })
    }
}

fn tool_file_write(args: &Value, fs: &FileSystem) -> Value {
    let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
    let content = args.get("content").and_then(|c| c.as_str()).unwrap_or("");

    if path.is_empty() {
        return json!({"success": false, "error": "Path cannot be empty"});
    }

    let mut fs = fs.write().unwrap();
    fs.insert(path.to_string(), content.to_string());
    info!("File write: {} ({} bytes)", path, content.len());
    json!({
        "success": true,
        "path": path,
        "size": content.len()
    })
}

fn tool_file_list(fs: &FileSystem) -> Value {
    let fs = fs.read().unwrap();
    let files: Vec<_> = fs.iter()
        .map(|(path, content)| json!({
            "path": path,
            "size": content.len()
        }))
        .collect();

    info!("File list: {} files", files.len());
    json!({
        "files": files,
        "count": files.len()
    })
}

async fn tool_delay(args: &Value) -> Value {
    let ms = args.get("ms").and_then(|m| m.as_u64()).unwrap_or(100);
    let ms = ms.min(30000); // Cap at 30 seconds

    info!("Delay tool: waiting {}ms", ms);
    tokio::time::sleep(Duration::from_millis(ms)).await;

    json!({
        "delayed_ms": ms,
        "completed": true,
        "timestamp": chrono::Local::now().to_rfc3339()
    })
}

fn tool_error_trigger(args: &Value, req_id: &Value) -> Value {
    let error_type = args.get("error_type").and_then(|e| e.as_str()).unwrap_or("internal");

    warn!("Error trigger: {}", error_type);

    let (code, message) = match error_type {
        "not_found" => (-32001, "Resource not found"),
        "invalid_input" => (-32602, "Invalid parameters"),
        "internal" => (-32603, "Internal error"),
        "permission" => (-32002, "Permission denied"),
        "timeout" => (-32003, "Operation timed out"),
        _ => (-32000, "Unknown error"),
    };

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {
            "code": code,
            "message": message,
            "data": {
                "error_type": error_type,
                "triggered_at": chrono::Local::now().to_rfc3339()
            }
        }
    })
}

fn tool_batch_process(args: &Value) -> Value {
    let items: Vec<&str> = args.get("items")
        .and_then(|i| i.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let operation = args.get("operation").and_then(|o| o.as_str()).unwrap_or("uppercase");

    info!("Batch process: {} items with operation '{}'", items.len(), operation);

    let results: Vec<Value> = items.iter().map(|item| {
        let processed = match operation {
            "uppercase" => item.to_uppercase(),
            "lowercase" => item.to_lowercase(),
            "reverse" => item.chars().rev().collect(),
            "length" => item.len().to_string(),
            _ => item.to_string(),
        };
        json!({
            "input": item,
            "output": processed
        })
    }).collect();

    json!({
        "operation": operation,
        "count": results.len(),
        "results": results
    })
}

fn tool_json_transform(args: &Value) -> Value {
    let data = args.get("data").cloned().unwrap_or(json!({}));
    let operation = args.get("operation").and_then(|o| o.as_str()).unwrap_or("stringify");

    info!("JSON transform: operation '{}'", operation);

    let result = match operation {
        "flatten" => {
            // Simple flatten for objects
            if let Some(obj) = data.as_object() {
                let mut flat = HashMap::new();
                flatten_json("", &data, &mut flat);
                json!(flat)
            } else {
                data
            }
        }
        "keys" => {
            if let Some(obj) = data.as_object() {
                json!(obj.keys().collect::<Vec<_>>())
            } else {
                json!([])
            }
        }
        "values" => {
            if let Some(obj) = data.as_object() {
                json!(obj.values().collect::<Vec<_>>())
            } else if let Some(arr) = data.as_array() {
                json!(arr)
            } else {
                json!([data])
            }
        }
        "stringify" => {
            json!(serde_json::to_string(&data).unwrap_or_default())
        }
        "pretty" => {
            json!(serde_json::to_string_pretty(&data).unwrap_or_default())
        }
        _ => data,
    };

    json!({
        "operation": operation,
        "result": result
    })
}

fn flatten_json(prefix: &str, value: &Value, result: &mut HashMap<String, Value>) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(&new_prefix, v, result);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten_json(&new_prefix, v, result);
            }
        }
        _ => {
            result.insert(prefix.to_string(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_echo() {
        let args = json!({"message": "hello"});
        let result = tool_echo(&args);
        assert_eq!(result.get("message").unwrap(), "hello");
    }

    #[test]
    fn test_tool_add() {
        let args = json!({"a": 5, "b": 3});
        let result = tool_add(&args);
        assert_eq!(result.get("result").unwrap(), 8.0);
    }

    #[test]
    fn test_tool_batch_process() {
        let args = json!({
            "items": ["hello", "world"],
            "operation": "uppercase"
        });
        let result = tool_batch_process(&args);
        assert_eq!(result.get("count").unwrap(), 2);
    }

    #[test]
    fn test_kv_operations() {
        let store: KvStore = Arc::new(RwLock::new(HashMap::new()));

        // Set
        let set_args = json!({"key": "test", "value": "hello"});
        let set_result = tool_kv_set(&set_args, &store);
        assert!(set_result.get("success").unwrap().as_bool().unwrap());

        // Get
        let get_args = json!({"key": "test"});
        let get_result = tool_kv_get(&get_args, &store);
        assert!(get_result.get("found").unwrap().as_bool().unwrap());
        assert_eq!(get_result.get("value").unwrap(), "hello");

        // Delete
        let del_args = json!({"key": "test"});
        let del_result = tool_kv_delete(&del_args, &store);
        assert!(del_result.get("deleted").unwrap().as_bool().unwrap());
    }
}
