//! HTTP API handlers for the OpenAgentiX gateway.
//!
//! All handlers return JSON responses. Errors are always structured:
//! ```json
//! {"error": "human-readable message", "field": "optional.field.path"}
//! ```
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | /healthz | Health check |
//! | GET | /api/v1/agents | List all loaded agents |
//! | POST | /api/v1/agents | Register an agent from YAML |
//! | GET | /api/v1/agents/:name | Get agent details |
//! | PUT | /api/v1/agents/:name | Update agent YAML |
//! | POST | /api/v1/agents/:name/run | Run an agent (SSE stream) |
//! | GET | /api/v1/agents/:name/runs | List runs for an agent |
//! | GET | /api/v1/agents/:name/runs/:run_id | Get run details |
//! | GET | /api/v1/agents/:name/runs/:run_id/logs | Get run logs |
//! | DELETE | /api/v1/agents/:name/runs/:run_id | Stop a running agent |

use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, sse::Event, sse::KeepAlive, Sse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

use super::agent_manager::AgentManager;
use crate::streaming::SseEncoder;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the axum router with all gateway endpoints.
pub fn create_router(manager: Arc<AgentManager>) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/agents", get(list_agents).post(register_agent))
        .route("/api/v1/agents/:name", get(get_agent).put(update_agent))
        .route("/api/v1/agents/:name/run", post(run_agent))
        .route("/api/v1/agents/:name/runs", get(list_runs))
        .route("/api/v1/agents/:name/runs/:run_id", get(get_run).delete(stop_run))
        .route("/api/v1/agents/:name/runs/:run_id/logs", get(get_run_logs))
        .layer(CorsLayer::permissive())
        .with_state(manager)
}

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

/// Error response body returned by all handlers on failure.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

/// Query params for the run endpoint.
#[derive(Debug, Deserialize)]
pub struct RunQuery {
    /// Output format: "sse" (default) or "json" (NDJSON).
    pub format: Option<String>,
}

/// Body for POST/PUT agent endpoints.
#[derive(Debug, Deserialize)]
pub struct AgentYamlBody {
    pub yaml: String,
}

/// Body for POST /api/v1/agents/:name/run
#[derive(Debug, Deserialize)]
pub struct RunBody {
    pub input: String,
}

// ---------------------------------------------------------------------------
// Helper: build error response
// ---------------------------------------------------------------------------

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    let body = Json(ErrorResponse {
        error: message.into(),
        field: None,
    });
    (status, body).into_response()
}

fn error_response_with_field(
    status: StatusCode,
    message: impl Into<String>,
    field: impl Into<String>,
) -> Response {
    let body = Json(ErrorResponse {
        error: message.into(),
        field: Some(field.into()),
    });
    (status, body).into_response()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /healthz
///
/// Returns `200 OK` with `{"status":"ok","version":"<crate version>"}`.
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /api/v1/agents
///
/// Returns JSON array of all loaded agents.
async fn list_agents(State(manager): State<Arc<AgentManager>>) -> impl IntoResponse {
    Json(manager.list_agents())
}

/// POST /api/v1/agents
///
/// Register a new agent from a YAML body.
/// Returns 201 on success, 409 if already exists, 400 on invalid YAML.
async fn register_agent(
    State(manager): State<Arc<AgentManager>>,
    Json(body): Json<AgentYamlBody>,
) -> Response {
    match manager.register_from_yaml(&body.yaml) {
        Ok(name) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "name": name, "status": "ready" })),
        )
            .into_response(),
        Err(err) => {
            let msg = err.to_string();
            // 409 if duplicate, 400 otherwise
            if msg.contains("already exists") {
                error_response(StatusCode::CONFLICT, msg)
            } else {
                // Try to extract field info from yaml_parse errors
                let field = extract_yaml_error_field(&msg);
                error_response_with_field(StatusCode::BAD_REQUEST, msg, field)
            }
        }
    }
}

/// GET /api/v1/agents/:name
///
/// Returns agent summary or 404 if not found.
async fn get_agent(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    match manager.get_agent(&name) {
        Some(summary) => Json(summary).into_response(),
        None => error_response(
            StatusCode::NOT_FOUND,
            format!("Agent '{}' not found", name),
        ),
    }
}

/// PUT /api/v1/agents/:name
///
/// Update an existing agent from YAML body.
/// Returns 200 on success, 404 if not found, 400 on invalid YAML.
async fn update_agent(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
    Json(body): Json<AgentYamlBody>,
) -> Response {
    match manager.update_from_yaml(&name, &body.yaml) {
        Ok(()) => Json(serde_json::json!({ "name": name, "status": "updated" })).into_response(),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, msg)
            } else {
                let field = extract_yaml_error_field(&msg);
                error_response_with_field(StatusCode::BAD_REQUEST, msg, field)
            }
        }
    }
}

/// POST /api/v1/agents/:name/run
///
/// Start an agent run. Streams `ReActEvent`s as Server-Sent Events.
/// Pass `?format=json` for NDJSON streaming instead of SSE.
/// Returns 404 if agent not found.
async fn run_agent(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
    Query(query): Query<RunQuery>,
    Json(body): Json<RunBody>,
) -> Response {
    match manager.start_run(&name, &body.input).await {
        Ok((_run_id, event_rx)) => {
            let use_json = query.format.as_deref() == Some("json");

            // Convert broadcast receiver to a stream
            let stream = BroadcastStream::new(event_rx)
                .take_while(|r| r.is_ok())
                .filter_map(|r| r.ok())
                .map(move |event| {
                    let data = if use_json {
                        crate::streaming::JsonFormatter::format(&event)
                    } else {
                        SseEncoder::encode_data(&event)
                    };
                    let event_name = SseEncoder::encode_event_name(&event);
                    Ok::<Event, Infallible>(
                        Event::default().data(data).event(event_name)
                    )
                });

            Sse::new(stream)
                .keep_alive(KeepAlive::default())
                .into_response()
        }
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, msg)
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }
}

/// GET /api/v1/agents/:name/runs
///
/// List all runs for the given agent.
async fn list_runs(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
) -> impl IntoResponse {
    Json(manager.list_runs(&name))
}

/// GET /api/v1/agents/:name/runs/:run_id
///
/// Get details of a specific run.
async fn get_run(
    State(manager): State<Arc<AgentManager>>,
    AxumPath((_name, run_id)): AxumPath<(String, String)>,
) -> Response {
    match manager.get_run_events(&run_id) {
        Some(summary) => Json(summary).into_response(),
        None => error_response(StatusCode::NOT_FOUND, format!("Run '{}' not found", run_id)),
    }
}

/// GET /api/v1/agents/:name/runs/:run_id/logs
///
/// Get the logged events for a run (same as get_run for now; Phase 14 will add structured logs).
async fn get_run_logs(
    State(manager): State<Arc<AgentManager>>,
    AxumPath((_name, run_id)): AxumPath<(String, String)>,
) -> Response {
    match manager.get_run_events(&run_id) {
        Some(summary) => Json(serde_json::json!({
            "run_id": run_id,
            "output": summary.output,
            "iterations": summary.iterations,
            "status": summary.status,
        }))
        .into_response(),
        None => error_response(StatusCode::NOT_FOUND, format!("Run '{}' not found", run_id)),
    }
}

/// DELETE /api/v1/agents/:name/runs/:run_id
///
/// Stop a running agent.
async fn stop_run(
    State(manager): State<Arc<AgentManager>>,
    AxumPath((_name, run_id)): AxumPath<(String, String)>,
) -> Response {
    match manager.stop_run(&run_id) {
        Ok(()) => Json(serde_json::json!({ "message": "Run stopped" })).into_response(),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, msg)
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a field path from a YAML parse error message.
///
/// Returns an empty string if no field can be detected.
fn extract_yaml_error_field(msg: &str) -> String {
    // Look for "field: X" or "Field: X" patterns from serde_path_to_error
    if let Some(idx) = msg.to_lowercase().find("field:") {
        let rest = &msg[idx + 6..];
        let field = rest.trim().split('\n').next().unwrap_or("").trim();
        return field.to_string();
    }
    String::new()
}
