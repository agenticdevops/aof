//! Unit tests for aof-mcp transport implementations

use aof_core::AofResult;
use aof_mcp::transport::{McpError, McpRequest, McpResponse, TransportType};

#[test]
fn test_mcp_request_creation() {
    let request = McpRequest::new("test/method", serde_json::json!({"param": "value"}));

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test/method");
    assert_eq!(request.params, serde_json::json!({"param": "value"}));
    assert!(!request.id.is_empty());
}

#[test]
fn test_mcp_request_serialization() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "test-id".to_string(),
        method: "test/method".to_string(),
        params: serde_json::json!({"key": "value"}),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test/method"));
    assert!(json.contains("test-id"));

    let deserialized: McpRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.method, "test/method");
    assert_eq!(deserialized.id, "test-id");
}

#[test]
fn test_mcp_response_success() {
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: "req-123".to_string(),
        result: Some(serde_json::json!({"status": "ok"})),
        error: None,
    };

    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_mcp_response_error() {
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: "req-456".to_string(),
        result: None,
        error: Some(McpError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }),
    };

    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
}

#[test]
fn test_mcp_error_serialization() {
    let error = McpError {
        code: -32601,
        message: "Method not found".to_string(),
        data: Some(serde_json::json!({"method": "unknown"})),
    };

    let json = serde_json::to_string(&error).unwrap();
    let deserialized: McpError = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.code, -32601);
    assert_eq!(deserialized.message, "Method not found");
    assert!(deserialized.data.is_some());
}

#[test]
fn test_transport_type_variants() {
    let types = vec![
        TransportType::Stdio,
        TransportType::Sse,
        TransportType::Http,
    ];

    for transport_type in types {
        let json = serde_json::to_string(&transport_type).unwrap();
        let deserialized: TransportType = serde_json::from_str(&json).unwrap();
        assert_eq!(transport_type, deserialized);
    }
}

#[test]
fn test_transport_type_serialization() {
    assert_eq!(
        serde_json::to_string(&TransportType::Stdio).unwrap(),
        "\"stdio\""
    );
    assert_eq!(
        serde_json::to_string(&TransportType::Sse).unwrap(),
        "\"sse\""
    );
    assert_eq!(
        serde_json::to_string(&TransportType::Http).unwrap(),
        "\"http\""
    );
}

#[test]
fn test_mcp_response_serialization() {
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: "123".to_string(),
        result: Some(serde_json::json!({"data": "test"})),
        error: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: McpResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "123");
    assert!(deserialized.result.is_some());
    assert!(deserialized.error.is_none());
}

#[test]
fn test_mcp_request_unique_ids() {
    let req1 = McpRequest::new("test", serde_json::json!({}));
    let req2 = McpRequest::new("test", serde_json::json!({}));

    // IDs should be unique (UUIDs)
    assert_ne!(req1.id, req2.id);
}

#[test]
fn test_mcp_error_with_data() {
    let error = McpError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(serde_json::json!({
            "expected": "string",
            "received": "number"
        })),
    };

    assert_eq!(error.code, -32602);
    assert!(error.data.is_some());
}

#[test]
fn test_mcp_error_without_data() {
    let error = McpError {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    };

    let json = serde_json::to_string(&error).unwrap();
    assert!(!json.contains("\"data\""));
}

// Mock tests for transport trait implementations would go here
// Since we can't test actual stdio/SSE/HTTP without running processes,
// we focus on data structure tests above
