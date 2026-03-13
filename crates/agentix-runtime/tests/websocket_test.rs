//! Integration tests for the WebSocket endpoint at /ws.
//!
//! These tests spin up a real axum TCP server on an ephemeral port, connect
//! via tokio-tungstenite, and assert that the JSON events arrive as expected.
//!
//! TDD RED phase: these tests are written before the implementation exists.

use std::sync::Arc;
use std::time::Duration;

use agentix_runtime::gateway::AgentManager;
use agentix_runtime::gateway::api::create_router;
use agentix_runtime::gateway::websocket::{EventBroadcaster, GatewayEvent};

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Bind a random port and return (listener, ws_url).
async fn bind_test_listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://127.0.0.1:{}/ws", addr.port());
    (listener, url)
}

/// Spawn the axum server with a broadcaster in the background.
/// Returns the URL and the broadcaster for injecting events.
async fn start_test_server() -> (String, Arc<EventBroadcaster>) {
    let manager = Arc::new(AgentManager::new(None));
    let broadcaster = Arc::new(EventBroadcaster::new(64));

    let (listener, ws_url) = bind_test_listener().await;

    let app = create_router(manager).with_broadcaster(Arc::clone(&broadcaster));

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Brief settle time so the server is ready before we connect
    tokio::time::sleep(Duration::from_millis(10)).await;

    (ws_url, broadcaster)
}

// ---------------------------------------------------------------------------
// Test 1: WebSocket connection and Connected message
// ---------------------------------------------------------------------------

/// Connects to /ws and verifies the first message is a Connected event.
#[tokio::test]
async fn test_websocket_connection() {
    let (ws_url, _broadcaster) = start_test_server().await;

    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("WebSocket connection should succeed");

    // The server sends a Connected message on connect
    let msg = tokio::time::timeout(Duration::from_secs(3), ws_stream.next())
        .await
        .expect("timed out waiting for Connected message")
        .expect("stream ended prematurely")
        .expect("WS error receiving message");

    assert!(msg.is_text(), "expected text frame, got: {:?}", msg);

    let text = msg.into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(json["type"], "connected", "expected type=connected, got: {}", json);
    assert!(json["message"].is_string(), "connected message should have a message field");

    ws_stream.close(None).await.ok();
}

// ---------------------------------------------------------------------------
// Test 2: AgentStatus event is broadcast to connected clients
// ---------------------------------------------------------------------------

/// Sends an AgentStatus event and verifies the WS client receives it.
#[tokio::test]
async fn test_websocket_receives_agent_status_event() {
    let (ws_url, broadcaster) = start_test_server().await;

    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("WebSocket connection should succeed");

    // Consume the Connected message first
    let _connected = tokio::time::timeout(Duration::from_secs(3), ws_stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    // Broadcast an AgentStatus event
    broadcaster.send(GatewayEvent::AgentStatus {
        agent_name: "test-agent".to_string(),
        status: "running".to_string(),
    });

    // Wait for the event to arrive
    let msg = tokio::time::timeout(Duration::from_secs(3), ws_stream.next())
        .await
        .expect("timed out waiting for AgentStatus event")
        .expect("stream ended prematurely")
        .expect("WS error receiving message");

    let text = msg.into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(json["type"], "agent_status", "expected type=agent_status, got: {}", json);
    assert_eq!(json["agent_name"], "test-agent");
    assert_eq!(json["status"], "running");

    ws_stream.close(None).await.ok();
}

// ---------------------------------------------------------------------------
// Test 3: RunCompleted event is broadcast to connected clients
// ---------------------------------------------------------------------------

/// Sends a RunCompleted event and verifies the WS client receives the correct JSON.
#[tokio::test]
async fn test_websocket_receives_run_event() {
    let (ws_url, broadcaster) = start_test_server().await;

    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("WebSocket connection should succeed");

    // Consume the Connected message first
    let _connected = tokio::time::timeout(Duration::from_secs(3), ws_stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    // Broadcast a RunCompleted event
    broadcaster.send(GatewayEvent::RunCompleted {
        agent_name: "test-agent".to_string(),
        run_id: "run-abc-123".to_string(),
        status: "success".to_string(),
        duration_ms: Some(1500),
    });

    // Wait for the event to arrive
    let msg = tokio::time::timeout(Duration::from_secs(3), ws_stream.next())
        .await
        .expect("timed out waiting for RunCompleted event")
        .expect("stream ended prematurely")
        .expect("WS error receiving message");

    let text = msg.into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(json["type"], "run_completed", "expected type=run_completed, got: {}", json);
    assert_eq!(json["agent_name"], "test-agent");
    assert_eq!(json["run_id"], "run-abc-123");
    assert_eq!(json["status"], "success");
    assert_eq!(json["duration_ms"], 1500);

    ws_stream.close(None).await.ok();
}

// ---------------------------------------------------------------------------
// Test 4: Multiple clients all receive broadcast events
// ---------------------------------------------------------------------------

/// Verifies fan-out: two connected clients both receive the same broadcast event.
#[tokio::test]
async fn test_websocket_broadcast_fan_out() {
    let (ws_url, broadcaster) = start_test_server().await;

    // Connect two clients
    let (mut ws1, _) = connect_async(&ws_url).await.unwrap();
    let (mut ws2, _) = connect_async(&ws_url).await.unwrap();

    // Consume Connected messages
    let _c1 = tokio::time::timeout(Duration::from_secs(3), ws1.next()).await.unwrap().unwrap().unwrap();
    let _c2 = tokio::time::timeout(Duration::from_secs(3), ws2.next()).await.unwrap().unwrap().unwrap();

    // Broadcast one event
    broadcaster.send(GatewayEvent::RunStarted {
        agent_name: "fan-out-agent".to_string(),
        run_id: "run-fanout-999".to_string(),
    });

    // Both clients should receive it
    let msg1 = tokio::time::timeout(Duration::from_secs(3), ws1.next())
        .await.unwrap().unwrap().unwrap();
    let msg2 = tokio::time::timeout(Duration::from_secs(3), ws2.next())
        .await.unwrap().unwrap().unwrap();

    let json1: serde_json::Value = serde_json::from_str(&msg1.into_text().unwrap()).unwrap();
    let json2: serde_json::Value = serde_json::from_str(&msg2.into_text().unwrap()).unwrap();

    assert_eq!(json1["type"], "run_started");
    assert_eq!(json2["type"], "run_started");
    assert_eq!(json1["run_id"], "run-fanout-999");
    assert_eq!(json2["run_id"], "run-fanout-999");

    ws1.close(None).await.ok();
    ws2.close(None).await.ok();
}
