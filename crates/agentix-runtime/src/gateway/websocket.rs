//! WebSocket handler for the OpenAgentiX gateway.
//!
//! Exposes a `/ws` endpoint that broadcasts typed JSON events to all connected
//! WebSocket clients in real time. The frontend Command Center subscribes here
//! instead of polling the REST API.
//!
//! # Event types
//!
//! Every message is a JSON object with a `"type"` discriminant:
//!
//! | type               | fields                                                           |
//! |--------------------|------------------------------------------------------------------|
//! | `connected`        | `message: String`                                                |
//! | `agent_status`     | `agent_name`, `status`                                           |
//! | `run_started`      | `agent_name`, `run_id`                                           |
//! | `run_completed`    | `agent_name`, `run_id`, `status`, `duration_ms?`                 |
//! | `approval_requested` | `id`, `agent_name`, `action`                                   |
//! | `approval_decided` | `id`, `decision`                                                 |
//! | `cost_update`      | `agent_name`, `total_cost_usd`                                   |
//!
//! # Architecture
//!
//! `EventBroadcaster` wraps a `tokio::sync::broadcast` channel.  Each new
//! WebSocket connection subscribes with `broadcaster.subscribe()`.  REST
//! handlers that change state call `broadcaster.send(event)` to fan-out to
//! all connected clients.

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

/// All events that can be broadcast over the WebSocket connection.
///
/// Serialized with `#[serde(tag = "type", rename_all = "snake_case")]` so that
/// each variant produces `{"type":"variant_name", ...fields}`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayEvent {
    /// Sent immediately after a client connects.
    Connected { message: String },
    /// An agent's status changed (idle → running → done, etc.).
    AgentStatus { agent_name: String, status: String },
    /// An agent run was initiated.
    RunStarted { agent_name: String, run_id: String },
    /// An agent run finished (success, failed, or stopped).
    RunCompleted {
        agent_name: String,
        run_id: String,
        status: String,
        duration_ms: Option<u64>,
    },
    /// An approval request is waiting for a human decision.
    ApprovalRequested {
        id: String,
        agent_name: String,
        action: String,
    },
    /// An approval request was resolved (approved or denied).
    ApprovalDecided { id: String, decision: String },
    /// An agent's cumulative LLM cost has been updated.
    CostUpdate {
        agent_name: String,
        total_cost_usd: f64,
    },
}

// ---------------------------------------------------------------------------
// EventBroadcaster
// ---------------------------------------------------------------------------

/// Thin wrapper around a `tokio::sync::broadcast` channel.
///
/// Clone-safe via `Arc`.  REST handlers hold an `Arc<EventBroadcaster>` and
/// call `.send()` after state changes; WebSocket clients call `.subscribe()`
/// on connection.
pub struct EventBroadcaster {
    tx: broadcast::Sender<GatewayEvent>,
}

impl EventBroadcaster {
    /// Create a new broadcaster with the given channel capacity.
    ///
    /// A capacity of 64 is more than enough for typical agent workloads;
    /// slow clients will be dropped (lagged) rather than blocking senders.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Subscribe to the broadcast channel.  Each subscriber receives a copy
    /// of every event sent *after* the subscription is created.
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.tx.subscribe()
    }

    /// Broadcast an event to all connected subscribers.
    ///
    /// Ignores the error when there are no receivers (normal when no clients
    /// are connected).
    pub fn send(&self, event: GatewayEvent) {
        // `send` fails only when there are 0 receivers — that's fine.
        let _ = self.tx.send(event);
    }

    /// Return the number of active subscribers (connected WebSocket clients).
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl std::fmt::Debug for EventBroadcaster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBroadcaster")
            .field("receivers", &self.tx.receiver_count())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// WebSocket handler
// ---------------------------------------------------------------------------

/// GET /ws
///
/// Upgrades the HTTP connection to WebSocket.  After upgrade the handler:
///
/// 1. Sends a `Connected` message to the new client.
/// 2. Subscribes to the broadcast channel.
/// 3. Forwards every event as a JSON text frame.
/// 4. Handles client disconnect gracefully.
/// 5. Ignores any incoming messages from the client (read-only stream).
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<Arc<EventBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

async fn handle_socket(mut socket: WebSocket, broadcaster: Arc<EventBroadcaster>) {
    // 1. Send Connected greeting
    let connected = GatewayEvent::Connected {
        message: "Connected to OpenAgentiX gateway".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&connected) {
        if socket.send(Message::Text(json)).await.is_err() {
            // Client disconnected before we could say hello
            return;
        }
    }

    // 2. Subscribe to the broadcast channel
    let mut rx = broadcaster.subscribe();

    // 3. Fan-out loop: forward events, handle disconnects
    loop {
        tokio::select! {
            // Incoming broadcast event → serialize and send to this client
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                if socket.send(Message::Text(json)).await.is_err() {
                                    debug!("WebSocket client disconnected");
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to serialize GatewayEvent: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged, dropped {} events", n);
                        // Continue — the client is still connected, just slow
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Broadcaster shut down (gateway stopping)
                        break;
                    }
                }
            }

            // Incoming message from client → we ignore content, but detect close
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) | None => {
                        debug!("WebSocket client closed connection");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore ping, pong, text, binary from client
                    }
                    Some(Err(e)) => {
                        debug!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
