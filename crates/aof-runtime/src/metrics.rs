//! Prometheus metrics registry for AOF runtime
//!
//! This module provides a centralized metrics registry that tracks:
//! - Agent execution (count, duration, active agents)
//! - Event system (emitted events, broadcast latency)
//! - WebSocket connections (active clients, messages sent/failed)
//! - LLM calls (requests, tokens, latency)
//! - Coordination overhead (heartbeats, failures, overhead %)
//! - System metrics (uptime, sessions)

use prometheus::{
    Counter, CounterVec, Gauge, Histogram, HistogramOpts, Opts, Registry,
    Encoder, TextEncoder,
};
use std::sync::Arc;
use crate::AofResult;

/// Centralized Prometheus metrics registry for AOF subsystems
pub struct AofMetrics {
    /// Prometheus registry
    pub registry: Registry,

    // Agent metrics
    /// Total agent executions (labels: agent_id, status)
    pub agent_executions_total: CounterVec,
    /// Agent execution duration histogram
    pub agent_execution_duration: Histogram,
    /// Currently active agents
    pub agents_active: Gauge,

    // Event metrics
    /// Total events emitted
    pub events_emitted_total: Counter,
    /// Event broadcast latency in seconds
    pub event_broadcast_latency: Histogram,

    // WebSocket metrics
    /// Currently connected WebSocket clients
    pub websocket_clients: Gauge,
    /// Total WebSocket messages sent
    pub websocket_messages_sent_total: Counter,
    /// Total WebSocket messages that failed
    pub websocket_messages_failed_total: Counter,

    // LLM metrics
    /// Total LLM requests (labels: provider, model)
    pub llm_requests_total: CounterVec,
    /// Total LLM tokens (labels: provider, type=input/output)
    pub llm_tokens_total: CounterVec,
    /// LLM API call latency in seconds
    pub llm_latency: Histogram,

    // Coordination metrics (Phase 7)
    /// Total heartbeat health checks performed
    pub heartbeat_checks_total: Counter,
    /// Total heartbeat check failures
    pub heartbeat_failures_total: Counter,
    /// Current coordination overhead as percentage
    pub coordination_overhead_percent: Gauge,

    // System metrics
    /// Daemon uptime in seconds
    pub uptime_seconds: Gauge,
    /// Number of active sessions
    pub session_count: Gauge,
}

impl AofMetrics {
    /// Create a new metrics registry with all AOF subsystem metrics
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let registry = Registry::new();

        // Agent metrics
        let agent_executions_total = CounterVec::new(
            Opts::new(
                "aof_agent_executions_total",
                "Total number of agent executions"
            ),
            &["agent_id", "status"]  // status: success, error, timeout
        )?;
        registry.register(Box::new(agent_executions_total.clone()))?;

        let agent_execution_duration = Histogram::with_opts(
            HistogramOpts::new(
                "aof_agent_execution_duration_seconds",
                "Agent execution duration in seconds"
            ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
        )?;
        registry.register(Box::new(agent_execution_duration.clone()))?;

        let agents_active = Gauge::new(
            "aof_agents_active",
            "Number of currently executing agents"
        )?;
        registry.register(Box::new(agents_active.clone()))?;

        // Event metrics
        let events_emitted_total = Counter::new(
            "aof_events_emitted_total",
            "Total number of coordination events emitted"
        )?;
        registry.register(Box::new(events_emitted_total.clone()))?;

        let event_broadcast_latency = Histogram::with_opts(
            HistogramOpts::new(
                "aof_event_broadcast_latency_seconds",
                "Event broadcast latency in seconds"
            ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1])
        )?;
        registry.register(Box::new(event_broadcast_latency.clone()))?;

        // WebSocket metrics
        let websocket_clients = Gauge::new(
            "aof_websocket_clients",
            "Number of connected WebSocket clients"
        )?;
        registry.register(Box::new(websocket_clients.clone()))?;

        let websocket_messages_sent_total = Counter::new(
            "aof_websocket_messages_sent_total",
            "Total WebSocket messages sent"
        )?;
        registry.register(Box::new(websocket_messages_sent_total.clone()))?;

        let websocket_messages_failed_total = Counter::new(
            "aof_websocket_messages_failed_total",
            "Total WebSocket message send failures"
        )?;
        registry.register(Box::new(websocket_messages_failed_total.clone()))?;

        // LLM metrics
        let llm_requests_total = CounterVec::new(
            Opts::new(
                "aof_llm_requests_total",
                "Total LLM API requests"
            ),
            &["provider", "model"]
        )?;
        registry.register(Box::new(llm_requests_total.clone()))?;

        let llm_tokens_total = CounterVec::new(
            Opts::new(
                "aof_llm_tokens_total",
                "Total LLM tokens consumed"
            ),
            &["provider", "type"]  // type: input, output
        )?;
        registry.register(Box::new(llm_tokens_total.clone()))?;

        let llm_latency = Histogram::with_opts(
            HistogramOpts::new(
                "aof_llm_latency_seconds",
                "LLM API call latency in seconds"
            ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
        )?;
        registry.register(Box::new(llm_latency.clone()))?;

        // Coordination metrics
        let heartbeat_checks_total = Counter::new(
            "aof_heartbeat_checks_total",
            "Total heartbeat health checks performed"
        )?;
        registry.register(Box::new(heartbeat_checks_total.clone()))?;

        let heartbeat_failures_total = Counter::new(
            "aof_heartbeat_failures_total",
            "Total heartbeat check failures"
        )?;
        registry.register(Box::new(heartbeat_failures_total.clone()))?;

        let coordination_overhead_percent = Gauge::new(
            "aof_coordination_overhead_percent",
            "Current coordination overhead as percentage of total tokens"
        )?;
        registry.register(Box::new(coordination_overhead_percent.clone()))?;

        // System metrics
        let uptime_seconds = Gauge::new(
            "aof_uptime_seconds",
            "Daemon uptime in seconds"
        )?;
        registry.register(Box::new(uptime_seconds.clone()))?;

        let session_count = Gauge::new(
            "aof_session_count",
            "Number of active sessions"
        )?;
        registry.register(Box::new(session_count.clone()))?;

        Ok(Self {
            registry,
            agent_executions_total,
            agent_execution_duration,
            agents_active,
            events_emitted_total,
            event_broadcast_latency,
            websocket_clients,
            websocket_messages_sent_total,
            websocket_messages_failed_total,
            llm_requests_total,
            llm_tokens_total,
            llm_latency,
            heartbeat_checks_total,
            heartbeat_failures_total,
            coordination_overhead_percent,
            uptime_seconds,
            session_count,
        })
    }

    /// Render metrics in Prometheus text format
    pub fn render(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(buffer)
    }
}

impl Default for AofMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default AofMetrics")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = AofMetrics::new().unwrap();
        assert!(metrics.render().is_ok());
    }

    #[test]
    fn test_metrics_render_prometheus_format() {
        let metrics = AofMetrics::new().unwrap();

        // Record some test metrics
        metrics.agent_executions_total.with_label_values(&["test-agent", "success"]).inc();
        metrics.agents_active.set(5.0);
        metrics.uptime_seconds.set(120.0);

        let output = metrics.render().unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify Prometheus text format
        assert!(output_str.contains("aof_agent_executions_total"));
        assert!(output_str.contains("aof_agents_active"));
        assert!(output_str.contains("aof_uptime_seconds"));
        assert!(output_str.contains("agent_id=\"test-agent\""));
        assert!(output_str.contains("status=\"success\""));
    }

    #[test]
    fn test_metrics_histogram_buckets() {
        let metrics = AofMetrics::new().unwrap();

        // Observe some durations
        metrics.agent_execution_duration.observe(0.7);
        metrics.agent_execution_duration.observe(3.5);
        metrics.agent_execution_duration.observe(15.0);

        let output = metrics.render().unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify histogram has correct bucket boundaries
        assert!(output_str.contains("aof_agent_execution_duration_seconds_bucket"));
        assert!(output_str.contains("le=\"0.5\""));
        assert!(output_str.contains("le=\"1\""));
        assert!(output_str.contains("le=\"5\""));
        assert!(output_str.contains("le=\"30\""));
        assert!(output_str.contains("le=\"+Inf\""));
    }

    #[test]
    fn test_llm_metrics_labels() {
        let metrics = AofMetrics::new().unwrap();

        // Record LLM activity
        metrics.llm_requests_total.with_label_values(&["anthropic", "claude-opus-4"]).inc();
        metrics.llm_tokens_total.with_label_values(&["anthropic", "input"]).inc_by(100.0);
        metrics.llm_tokens_total.with_label_values(&["anthropic", "output"]).inc_by(250.0);

        let output = metrics.render().unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("aof_llm_requests_total"));
        assert!(output_str.contains("provider=\"anthropic\""));
        assert!(output_str.contains("model=\"claude-opus-4\""));
        assert!(output_str.contains("type=\"input\""));
        assert!(output_str.contains("type=\"output\""));
    }
}
