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
//! | GET | /api/v1/runs | List all runs (optionally filtered by ?agent=name) |
//! | POST | /api/v1/agents/:name/trigger | Fire an agent via the agent-to-agent or CLI trigger API |
//! | POST | /api/v1/agents/:name/delegate | Delegate a task to an agent and await the result |
//! | GET  | /api/v1/agents/:name/memory | List all vector memory entries for an agent |
//! | DELETE | /api/v1/agents/:name/memory | Clear all vector memory entries for an agent |
//! | POST | /webhooks/:trigger_id | Receive a webhook payload and fire a trigger |
//! | GET | /api/v1/agents/:name/runs/:run_id/trace | Get execution trace spans for a run |
//! | GET | /api/v1/agents/:name/runs/:run_id/structured-logs | Get structured log entries for a run |
//! | GET | /metrics | Prometheus metrics endpoint |
//! | GET | /api/v1/costs | List cost summaries for all agents |
//! | GET | /api/v1/costs/agents/:name | Get cost summary for a specific agent |
//! | GET | /api/v1/costs/agents/:name/runs | List per-run cost breakdown for an agent |

use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, sse::Event, sse::KeepAlive, Sse},
    routing::{get, post},
};
use agentix_core::TriggerSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        .route("/api/v1/runs", get(list_all_runs))
        .route("/api/v1/agents/:name/trigger", post(trigger_agent))
        .route("/api/v1/agents/:name/delegate", post(delegate_to_agent))
        .route("/api/v1/agents/:name/memory", get(get_agent_memory).delete(clear_agent_memory))
        .route("/api/v1/agents/:name/runs/:run_id/trace", get(get_run_trace))
        .route("/api/v1/agents/:name/runs/:run_id/structured-logs", get(get_run_structured_logs))
        .route("/metrics", get(prometheus_metrics))
        .route("/api/v1/costs", get(list_all_costs))
        .route("/api/v1/costs/agents/:name", get(get_agent_costs))
        .route("/api/v1/costs/agents/:name/runs", get(get_agent_run_costs))
        .route("/webhooks/:trigger_id", post(receive_webhook))
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

/// Query params for the global runs listing endpoint.
#[derive(Debug, Deserialize)]
pub struct RunsQuery {
    /// Filter by agent name.
    pub agent: Option<String>,
    /// Maximum number of runs to return (default: 20).
    pub limit: Option<usize>,
}

/// Query params for the per-agent run cost listing endpoint.
#[derive(Debug, Deserialize)]
pub struct CostRunsQuery {
    /// Maximum number of runs to return (default: 20).
    pub limit: Option<usize>,
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

/// Body for POST /api/v1/agents/:name/trigger
#[derive(Debug, Deserialize)]
pub struct TriggerBody {
    /// The task payload to pass to the target agent.
    pub payload: serde_json::Value,
    /// Name of the agent (or system) that is firing this trigger.
    pub caller: Option<String>,
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

/// GET /api/v1/runs
///
/// List all runs across all agents, optionally filtered by agent name.
/// Returns most recent runs first (up to `limit`, default 20).
async fn list_all_runs(
    State(manager): State<Arc<AgentManager>>,
    Query(query): Query<RunsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let agent_filter = query.agent.as_deref();

    match manager.run_store.list(agent_filter, limit) {
        Ok(records) => {
            let json_records: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| serde_json::json!({
                    "id": r.id,
                    "agent": r.agent_name,
                    "status": r.status,
                    "started_at": r.started_at.to_rfc3339(),
                    "ended_at": r.ended_at.map(|d| d.to_rfc3339()),
                    "duration_ms": r.duration_ms,
                    "trigger_source": r.trigger_source,
                    "trigger_id": r.trigger_id,
                    "iterations": r.iterations,
                    "input_summary": r.input_summary,
                    "output_summary": r.output_summary,
                }))
                .collect();
            Json(json_records).into_response()
        }
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

/// POST /api/v1/agents/:name/trigger
///
/// Fire an agent programmatically — from another agent, from the CLI, or from
/// external orchestration systems.
///
/// This endpoint accepts a `TriggerBody` with a `payload` and an optional `caller`
/// name, creates a `TriggerEvent{source: Agent}`, and fires the target agent
/// asynchronously.
///
/// Returns 202 Accepted with the generated run_id.
async fn trigger_agent(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
    Json(body): Json<TriggerBody>,
) -> Response {
    let caller = body.caller.as_deref().unwrap_or("api");
    let trigger_id = format!("{}-agent-trigger", name);

    let event = agentix_core::TriggerEvent::new(
        TriggerSource::Agent,
        body.payload,
        &trigger_id,
    )
    .with_context("caller_agent", caller);

    match manager.run_agent_with_trigger(&name, event).await {
        Ok(run_id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "accepted": true,
                "run_id": run_id,
                "agent": name,
                "trigger_id": trigger_id,
            })),
        )
            .into_response(),
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

/// POST /webhooks/:trigger_id
///
/// Receives a webhook payload and dispatches it to the agent that registered
/// the trigger with this `trigger_id`.
///
/// The `trigger_id` is auto-generated at agent load time as `{agent_name}-{type}-{index}`.
/// Example: `POST /webhooks/my-agent-webhook-0`
///
/// Determines the trigger source from the incoming headers:
/// - `x-github-event` present → `TriggerSource::GitHub`
/// - `x-jira-event` or body has `webhookEvent` field → `TriggerSource::Jira`
/// - otherwise → `TriggerSource::Webhook`
///
/// Returns 202 Accepted if the event is queued, 404 if no agent owns the trigger_id.
async fn receive_webhook(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(trigger_id): AxumPath<String>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // Determine trigger source from headers
    let source = if headers.contains_key("x-github-event") {
        TriggerSource::GitHub
    } else if headers.contains_key("x-jira-event")
        || body.get("webhookEvent").is_some()
    {
        TriggerSource::Jira
    } else {
        TriggerSource::Webhook
    };

    // Collect headers into context map
    let context: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            let key = k.as_str().to_lowercase();
            let val = v.to_str().ok().map(|s| s.to_string());
            val.map(|v| (key, v))
        })
        .collect();

    match manager
        .dispatch_webhook_payload(&trigger_id, body, source, context)
        .await
    {
        Ok(agent_name) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "status": "queued",
                "trigger_id": trigger_id,
                "agent": agent_name,
            })),
        )
            .into_response(),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("No agent registered") {
                error_response(StatusCode::NOT_FOUND, msg)
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Delegation handler (COORD-02, COORD-03, COORD-04, COORD-05)
// ---------------------------------------------------------------------------

/// Request body for POST /api/v1/agents/:name/delegate
#[derive(Debug, Deserialize)]
pub struct DelegateRequest {
    pub from_agent: String,
    pub task: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Response for POST /api/v1/agents/:name/delegate
#[derive(Debug, Serialize)]
pub struct DelegateResponse {
    pub delegation_id: String,
    pub output: String,
    pub status: String,
    pub from_agent: String,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/v1/agents/:name/delegate
///
/// Delegates a task to the named agent from a coordinator. Returns the delegation result.
async fn delegate_to_agent(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
    Json(req): Json<DelegateRequest>,
) -> Response {
    match manager.delegate_task(&req.from_agent, &name, &req.task, req.payload).await {
        Ok(result) => {
            let resp = DelegateResponse {
                delegation_id: result.delegation_id,
                output: result.output,
                status: format!("{:?}", result.status).to_lowercase(),
                from_agent: result.from_agent,
                completed_at: result.completed_at,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, msg)
            } else if msg.contains("inbox full") || msg.contains("channel closed") {
                error_response(StatusCode::SERVICE_UNAVAILABLE, msg)
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Memory handlers (MEM-01 through MEM-04)
// ---------------------------------------------------------------------------

/// GET /api/v1/agents/:name/memory
///
/// Returns all vector memory entries for the named agent as a JSON array.
/// Returns an empty array if memory is disabled for the agent.
async fn get_agent_memory(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    match manager.list_agent_memory(&name).await {
        Ok(entries) => {
            let json_entries: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "agent_id": e.agent_id,
                        "run_id": e.run_id,
                        "text": e.text,
                        "metadata": e.metadata,
                        "stored_at": e.stored_at.to_rfc3339(),
                    })
                })
                .collect();
            let count = json_entries.len();
            (StatusCode::OK, Json(serde_json::json!({
                "agent": name,
                "entries": json_entries,
                "count": count
            }))).into_response()
        }
        Err(e) if e.to_string().contains("not found") => {
            error_response(StatusCode::NOT_FOUND, format!("Agent '{}' not found", name))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

/// DELETE /api/v1/agents/:name/memory
///
/// Clears all vector memory entries for the named agent.
async fn clear_agent_memory(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    match manager.clear_agent_memory(&name).await {
        Ok(count) => {
            (StatusCode::OK, Json(serde_json::json!({
                "cleared": true,
                "agent": name,
                "entries_deleted": count
            }))).into_response()
        }
        Err(e) if e.to_string().contains("not found") => {
            error_response(StatusCode::NOT_FOUND, format!("Agent '{}' not found", name))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Telemetry handlers (TELE-01 through TELE-05)
// ---------------------------------------------------------------------------

/// GET /api/v1/agents/:name/runs/:run_id/trace
///
/// Returns the span tree for a run as a JSON array of span objects, ordered by start_time.
async fn get_run_trace(
    State(manager): State<Arc<AgentManager>>,
    AxumPath((name, run_id)): AxumPath<(String, String)>,
) -> Response {
    let _ = name; // Agent name validated by path; spans are keyed by run_id
    match manager.trace_store.get_run_trace(&run_id) {
        Ok(spans) if spans.is_empty() => {
            error_response(StatusCode::NOT_FOUND, format!("No trace data found for run '{}'", run_id))
        }
        Ok(spans) => {
            let json: Vec<serde_json::Value> = spans
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "span_id": s.span_id,
                        "parent_span_id": s.parent_span_id,
                        "trace_id": s.trace_id,
                        "name": s.name,
                        "kind": s.kind,
                        "start_time": s.start_time.to_rfc3339(),
                        "end_time": s.end_time.map(|t| t.to_rfc3339()),
                        "duration_ms": s.duration_ms,
                        "status": s.status,
                        "attributes": s.attributes,
                    })
                })
                .collect();
            Json(json).into_response()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /api/v1/agents/:name/runs/:run_id/structured-logs
///
/// Returns structured log entries for a run as a JSON array, ordered by timestamp.
async fn get_run_structured_logs(
    State(manager): State<Arc<AgentManager>>,
    AxumPath((name, run_id)): AxumPath<(String, String)>,
) -> Response {
    let _ = name;
    match manager.trace_store.get_run_logs(&run_id) {
        Ok(logs) => {
            let json: Vec<serde_json::Value> = logs
                .into_iter()
                .map(|l| {
                    serde_json::json!({
                        "timestamp": l.timestamp.to_rfc3339(),
                        "level": l.level,
                        "message": l.message,
                        "trace_id": l.trace_id,
                        "span_id": l.span_id,
                        "agent": l.agent,
                        "run_id": l.run_id,
                        "fields": l.fields,
                    })
                })
                .collect();
            Json(json).into_response()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /metrics
///
/// Returns Prometheus text format metrics for all agent execution counters.
async fn prometheus_metrics(State(_manager): State<Arc<AgentManager>>) -> Response {
    // Return placeholder metrics format — full AofMetrics integration is deferred
    // until metrics are wired into the run lifecycle.
    let body = "# HELP agentix_info OpenAgentiX gateway information\n\
                # TYPE agentix_info gauge\n\
                agentix_info{version=\"2.0.0-alpha.6\"} 1\n";
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Cost handlers (COST-01 through COST-07, CLI-09)
// ---------------------------------------------------------------------------

/// GET /api/v1/costs
///
/// Returns a cost summary for every agent that has recorded LLM usage,
/// sorted by total_cost_usd descending.
async fn list_all_costs(State(manager): State<Arc<AgentManager>>) -> impl IntoResponse {
    let summaries = manager.get_all_cost_summaries();
    let json: Vec<serde_json::Value> = summaries
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "agent": s.agent_name,
                "total_runs": s.total_runs,
                "total_input_tokens": s.total_input_tokens,
                "total_output_tokens": s.total_output_tokens,
                "total_cost_usd": s.total_cost_usd,
                "last_run_at": s.last_run_at.map(|d| d.to_rfc3339()),
            })
        })
        .collect();
    Json(json)
}

/// GET /api/v1/costs/agents/:name
///
/// Returns the cost summary for a specific agent.
/// Returns 404 if no cost data exists for that agent.
async fn get_agent_costs(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    match manager.get_agent_cost_summary(&name) {
        Some(s) => Json(serde_json::json!({
            "agent": s.agent_name,
            "total_runs": s.total_runs,
            "total_input_tokens": s.total_input_tokens,
            "total_output_tokens": s.total_output_tokens,
            "total_cost_usd": s.total_cost_usd,
            "last_run_at": s.last_run_at.map(|d| d.to_rfc3339()),
        }))
        .into_response(),
        None => error_response(
            StatusCode::NOT_FOUND,
            format!("No cost data found for agent '{}'", name),
        ),
    }
}

/// GET /api/v1/costs/agents/:name/runs
///
/// Returns a paginated per-run cost breakdown for the named agent.
/// Use `?limit=N` to control the number of results (default: 20).
async fn get_agent_run_costs(
    State(manager): State<Arc<AgentManager>>,
    AxumPath(name): AxumPath<String>,
    Query(query): Query<CostRunsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let runs = manager.get_agent_run_cost_summaries(&name, limit);
    let json: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "run_id": r.run_id,
                "agent": r.agent_name,
                "model": r.model,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "cost_usd": r.cost_usd,
                "started_at": r.started_at.to_rfc3339(),
            })
        })
        .collect();
    Json(json)
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
