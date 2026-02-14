//! Metrics API endpoints
//!
//! Provides JSON endpoints for agent reliability metrics.
//! Metrics are computed from CoordinationEvent history and cached
//! in a ReliabilityCache for efficient concurrent access.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use aof_personas::ReliabilityCache;

/// Shared state for metrics API
#[derive(Clone)]
pub struct MetricsState {
    pub cache: Arc<ReliabilityCache>,
}

impl MetricsState {
    pub fn new(cache: Arc<ReliabilityCache>) -> Self {
        Self { cache }
    }
}

/// JSON response shape for /api/agents/:id/metrics
#[derive(Serialize)]
pub struct MetricsResponse {
    /// Agent identifier
    pub agent_id: String,

    /// Uptime percentage (null if insufficient data)
    pub uptime_percent: Option<f32>,

    /// Success rate percentage (null if insufficient data)
    pub success_rate: Option<f32>,

    /// Total number of events processed
    pub event_count: usize,

    /// ISO 8601 timestamp of last metric update
    pub last_update: String,

    /// ISO 8601 timestamp of last error (null if no errors)
    pub last_error: Option<String>,
}

/// GET /api/agents/:id/metrics — Returns reliability metrics for an agent
///
/// Response headers include X-Metrics-Version for cache invalidation.
///
/// Returns:
/// - 200 with metrics JSON if agent has events
/// - 404 if agent_id not found in event history
pub async fn get_agent_metrics(
    State(state): State<MetricsState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let metrics = state.cache.get_metrics(&agent_id).await;

    let version = state.cache.version();

    match metrics {
        Some(m) => {
            let response = MetricsResponse {
                agent_id: m.agent_id,
                uptime_percent: m.uptime_percent,
                success_rate: m.success_rate,
                event_count: m.event_count,
                last_update: m.last_update.to_rfc3339(),
                last_error: m.last_error.map(|dt| dt.to_rfc3339()),
            };

            let mut headers = HeaderMap::new();
            if let Ok(v) = version.to_string().parse() {
                headers.insert(
                    HeaderName::from_static("x-metrics-version"),
                    v,
                );
            }

            (StatusCode::OK, headers, Json(response)).into_response()
        }
        None => {
            let body = serde_json::json!({
                "error": format!("No metrics found for agent: {}", agent_id)
            });
            (StatusCode::NOT_FOUND, Json(body)).into_response()
        }
    }
}
