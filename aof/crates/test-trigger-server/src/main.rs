//! Test Trigger Server - A mock platform server for testing AOF triggers
//!
//! This server simulates messaging platforms (Telegram, Slack, Discord, WhatsApp)
//! for automated testing of AOF trigger workflows. It provides:
//!
//! - Mock webhook endpoints for each platform
//! - Event logging and verification
//! - Test utilities for workflow validation
//!
//! Usage:
//!   cargo run -p test-trigger-server
//!   # Server starts on http://localhost:3333

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

/// Test event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvent {
    pub id: String,
    pub platform: String,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
}

/// Server state
#[derive(Clone)]
struct AppState {
    events: Arc<RwLock<Vec<TestEvent>>>,
    responses: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState::new();

    let app = Router::new()
        // Health and info endpoints
        .route("/", get(root_handler))
        .route("/health", get(health_handler))

        // Platform webhook simulation endpoints
        .route("/webhook/telegram", post(telegram_webhook))
        .route("/webhook/slack", post(slack_webhook))
        .route("/webhook/discord", post(discord_webhook))
        .route("/webhook/whatsapp", post(whatsapp_webhook))

        // Test utilities
        .route("/events", get(list_events))
        .route("/events/:platform", get(list_platform_events))
        .route("/events/clear", post(clear_events))
        .route("/responses/:id", get(get_response))
        .route("/responses/:id", post(set_response))

        // Workflow trigger simulation
        .route("/trigger/:platform", post(trigger_workflow))

        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));
    info!("Test trigger server starting on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /           - Server info");
    info!("  GET  /health     - Health check");
    info!("  POST /webhook/{{platform}} - Receive webhook from platform");
    info!("  GET  /events     - List all recorded events");
    info!("  POST /trigger/{{platform}} - Trigger a workflow");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// HTTP Handlers
// ============================================================================

async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "test-trigger-server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Mock platform server for AOF trigger testing",
        "platforms": ["telegram", "slack", "discord", "whatsapp"],
        "endpoints": {
            "webhooks": "/webhook/{platform}",
            "events": "/events",
            "trigger": "/trigger/{platform}"
        }
    }))
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Telegram webhook handler
async fn telegram_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    record_event(&state, "telegram", "webhook", payload.clone(), &headers).await;

    // Return Telegram-style response
    Json(serde_json::json!({
        "ok": true
    }))
}

/// Slack webhook handler
async fn slack_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    record_event(&state, "slack", "webhook", payload.clone(), &headers).await;

    // Handle Slack URL verification challenge
    if let Some(challenge) = payload.get("challenge") {
        return Json(serde_json::json!({
            "challenge": challenge
        }));
    }

    // Return Slack-style acknowledgment
    Json(serde_json::json!({
        "ok": true
    }))
}

/// Discord webhook handler
async fn discord_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    record_event(&state, "discord", "webhook", payload.clone(), &headers).await;

    // Handle Discord ping
    if payload.get("type") == Some(&serde_json::json!(1)) {
        return Json(serde_json::json!({
            "type": 1
        }));
    }

    // Return Discord-style response
    Json(serde_json::json!({
        "type": 4,
        "data": {
            "content": "Message received"
        }
    }))
}

/// WhatsApp webhook handler
async fn whatsapp_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    record_event(&state, "whatsapp", "webhook", payload.clone(), &headers).await;

    // Return WhatsApp-style acknowledgment
    Json(serde_json::json!({
        "messaging_product": "whatsapp",
        "status": "accepted"
    }))
}

/// List all recorded events
async fn list_events(State(state): State<AppState>) -> impl IntoResponse {
    let events = state.events.read().await;
    Json(serde_json::json!({
        "count": events.len(),
        "events": *events
    }))
}

/// List events for a specific platform
async fn list_platform_events(
    State(state): State<AppState>,
    Path(platform): Path<String>,
) -> impl IntoResponse {
    let events = state.events.read().await;
    let platform_events: Vec<_> = events
        .iter()
        .filter(|e| e.platform == platform)
        .cloned()
        .collect();

    Json(serde_json::json!({
        "platform": platform,
        "count": platform_events.len(),
        "events": platform_events
    }))
}

/// Clear all events
async fn clear_events(State(state): State<AppState>) -> impl IntoResponse {
    let mut events = state.events.write().await;
    let count = events.len();
    events.clear();

    Json(serde_json::json!({
        "cleared": count
    }))
}

/// Get a stored response
async fn get_response(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let responses = state.responses.read().await;

    match responses.get(&id) {
        Some(response) => Json(serde_json::json!({
            "id": id,
            "response": response
        })),
        None => Json(serde_json::json!({
            "id": id,
            "response": null,
            "error": "Response not found"
        })),
    }
}

/// Set a response for testing
async fn set_response(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(response): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut responses = state.responses.write().await;
    responses.insert(id.clone(), response);

    Json(serde_json::json!({
        "id": id,
        "status": "stored"
    }))
}

/// Trigger a workflow by simulating platform message
async fn trigger_workflow(
    State(state): State<AppState>,
    Path(platform): Path<String>,
    Json(payload): Json<TriggerRequest>,
) -> impl IntoResponse {
    debug!("Triggering workflow on platform: {}", platform);

    // Generate platform-specific payload
    let platform_payload = match platform.as_str() {
        "telegram" => generate_telegram_payload(&payload),
        "slack" => generate_slack_payload(&payload),
        "discord" => generate_discord_payload(&payload),
        "whatsapp" => generate_whatsapp_payload(&payload),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Unknown platform: {}", platform)
                })),
            );
        }
    };

    // Record the triggered event
    let event_id = uuid::Uuid::new_v4().to_string();
    let event = TestEvent {
        id: event_id.clone(),
        platform: platform.clone(),
        event_type: "trigger".to_string(),
        timestamp: chrono::Utc::now(),
        payload: platform_payload.clone(),
        headers: HashMap::new(),
    };

    state.events.write().await.push(event);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "triggered",
            "event_id": event_id,
            "platform": platform,
            "payload": platform_payload
        })),
    )
}

/// Trigger request body
#[derive(Debug, Deserialize)]
struct TriggerRequest {
    user_id: String,
    channel_id: String,
    text: String,
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(default)]
    metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Platform Payload Generators
// ============================================================================

fn generate_telegram_payload(req: &TriggerRequest) -> serde_json::Value {
    serde_json::json!({
        "update_id": rand_update_id(),
        "message": {
            "message_id": rand_message_id(),
            "from": {
                "id": req.user_id.parse::<i64>().unwrap_or(123456789),
                "is_bot": false,
                "first_name": "Test",
                "username": "test_user"
            },
            "chat": {
                "id": req.channel_id.parse::<i64>().unwrap_or(-1001234567890_i64),
                "title": "Test Chat",
                "type": "group"
            },
            "date": chrono::Utc::now().timestamp(),
            "text": req.text
        }
    })
}

fn generate_slack_payload(req: &TriggerRequest) -> serde_json::Value {
    serde_json::json!({
        "token": "test_token",
        "team_id": "T0001",
        "api_app_id": "A0001",
        "event": {
            "type": "message",
            "channel": req.channel_id,
            "user": req.user_id,
            "text": req.text,
            "ts": format!("{}.{}", chrono::Utc::now().timestamp(), rand_ts_suffix()),
            "thread_ts": req.thread_id
        },
        "type": "event_callback",
        "event_id": format!("Ev{}", rand_event_id()),
        "event_time": chrono::Utc::now().timestamp()
    })
}

fn generate_discord_payload(req: &TriggerRequest) -> serde_json::Value {
    serde_json::json!({
        "type": 0,
        "channel_id": req.channel_id,
        "content": req.text,
        "author": {
            "id": req.user_id,
            "username": "test_user",
            "discriminator": "0001",
            "bot": false
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "id": rand_snowflake()
    })
}

fn generate_whatsapp_payload(req: &TriggerRequest) -> serde_json::Value {
    serde_json::json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
            "changes": [{
                "value": {
                    "messaging_product": "whatsapp",
                    "metadata": {
                        "display_phone_number": "15550000000",
                        "phone_number_id": "PHONE_NUMBER_ID"
                    },
                    "contacts": [{
                        "profile": {
                            "name": "Test User"
                        },
                        "wa_id": req.user_id
                    }],
                    "messages": [{
                        "from": req.user_id,
                        "id": format!("wamid.{}", rand_wamid()),
                        "timestamp": chrono::Utc::now().timestamp().to_string(),
                        "text": {
                            "body": req.text
                        },
                        "type": "text"
                    }]
                },
                "field": "messages"
            }]
        }]
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn record_event(
    state: &AppState,
    platform: &str,
    event_type: &str,
    payload: serde_json::Value,
    headers: &HeaderMap,
) {
    let mut header_map = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(key.to_string(), value_str.to_string());
        }
    }

    let event = TestEvent {
        id: uuid::Uuid::new_v4().to_string(),
        platform: platform.to_string(),
        event_type: event_type.to_string(),
        timestamp: chrono::Utc::now(),
        payload,
        headers: header_map,
    };

    debug!("Recording event: {} - {}", platform, event_type);
    state.events.write().await.push(event);
}

fn rand_update_id() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64 % 1_000_000_000
}

fn rand_message_id() -> i64 {
    rand_update_id() % 10000
}

fn rand_ts_suffix() -> String {
    format!("{:06}", rand_update_id() % 1_000_000)
}

fn rand_event_id() -> String {
    format!("{:012}", rand_update_id())
}

fn rand_snowflake() -> String {
    format!("{}", rand_update_id())
}

fn rand_wamid() -> String {
    format!("HBgL{}{}", rand_update_id(), rand_update_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_payload_generation() {
        let req = TriggerRequest {
            user_id: "123456".to_string(),
            channel_id: "-100123456".to_string(),
            text: "/run agent test".to_string(),
            thread_id: None,
            metadata: HashMap::new(),
        };

        let payload = generate_telegram_payload(&req);
        assert!(payload.get("message").is_some());
        assert_eq!(payload["message"]["text"], "/run agent test");
    }

    #[test]
    fn test_slack_payload_generation() {
        let req = TriggerRequest {
            user_id: "U123456".to_string(),
            channel_id: "C123456".to_string(),
            text: "/run agent test".to_string(),
            thread_id: None,
            metadata: HashMap::new(),
        };

        let payload = generate_slack_payload(&req);
        assert!(payload.get("event").is_some());
        assert_eq!(payload["event"]["text"], "/run agent test");
    }

    #[test]
    fn test_discord_payload_generation() {
        let req = TriggerRequest {
            user_id: "123456789".to_string(),
            channel_id: "987654321".to_string(),
            text: "/run agent test".to_string(),
            thread_id: None,
            metadata: HashMap::new(),
        };

        let payload = generate_discord_payload(&req);
        assert_eq!(payload["content"], "/run agent test");
        assert_eq!(payload["author"]["id"], "123456789");
    }

    #[test]
    fn test_whatsapp_payload_generation() {
        let req = TriggerRequest {
            user_id: "15551234567".to_string(),
            channel_id: "chat123".to_string(),
            text: "/run agent test".to_string(),
            thread_id: None,
            metadata: HashMap::new(),
        };

        let payload = generate_whatsapp_payload(&req);
        assert!(payload.get("entry").is_some());
        assert_eq!(
            payload["entry"][0]["changes"][0]["value"]["messages"][0]["text"]["body"],
            "/run agent test"
        );
    }
}
