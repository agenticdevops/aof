//! Gateway HTTP client for `agentix` CLI commands.
//!
//! Wraps all HTTP calls to the running gateway. On connection errors, returns
//! a typed `GatewayUnreachable` error that command handlers display with a
//! helpful "start the gateway" message.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures_util::Stream;
use reqwest::StatusCode;

/// Sentinel error message used to detect unreachable gateway.
const UNREACHABLE_PREFIX: &str = "GATEWAY_UNREACHABLE";

/// HTTP client for the OpenAgentiX gateway REST API.
pub struct GatewayClient {
    base_url: String,
    client: reqwest::Client,
}

impl GatewayClient {
    /// Create a new client pointed at `base_url` (e.g. `http://127.0.0.1:7777`).
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Returns `true` if the gateway responds to `/healthz`.
    pub async fn health(&self) -> Result<bool> {
        match self.client.get(self.url("/healthz")).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(e) if is_connection_error(&e) => Ok(false),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    /// `GET /api/v1/agents` — list all registered agents.
    pub async fn list_agents(&self) -> Result<Vec<serde_json::Value>> {
        let resp = self
            .client
            .get(self.url("/api/v1/agents"))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `GET /api/v1/agents/:name` — get a single agent.
    pub async fn get_agent(&self, name: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url(&format!("/api/v1/agents/{}", name)))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `GET /api/v1/runs` — run history across all agents (or filtered by agent name).
    ///
    /// Uses the global runs endpoint backed by SQLite for persistent history.
    pub async fn list_runs(
        &self,
        agent: Option<&str>,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let mut query = vec![("limit", limit.to_string())];
        if let Some(name) = agent {
            query.push(("agent", name.to_string()));
        }
        let resp = self
            .client
            .get(self.url("/api/v1/runs"))
            .query(&query)
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `GET /api/v1/agents/:name/runs/:run_id/logs` — get stored run events.
    pub async fn get_run_logs(
        &self,
        agent: &str,
        run_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let resp = self
            .client
            .get(self.url(&format!(
                "/api/v1/agents/{}/runs/{}/logs",
                agent, run_id
            )))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `DELETE /api/v1/agents/:name/runs/:run_id` — stop a run.
    pub async fn stop_run(&self, agent: &str, run_id: &str) -> Result<()> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/agents/{}/runs/{}", agent, run_id)))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?;
        Ok(())
    }

    /// `POST /api/v1/agents` — register a new agent from YAML.
    ///
    /// Returns `(status_code, response_body)` so callers can detect 409 Conflict.
    pub async fn register_agent(
        &self,
        yaml: &str,
    ) -> Result<(u16, serde_json::Value)> {
        let resp = self
            .client
            .post(self.url("/api/v1/agents"))
            .json(&serde_json::json!({ "yaml": yaml }))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
        Ok((status, body))
    }

    /// `PUT /api/v1/agents/:name` — update an existing agent from YAML.
    pub async fn update_agent(&self, name: &str, yaml: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .put(self.url(&format!("/api/v1/agents/{}", name)))
            .json(&serde_json::json!({ "yaml": yaml }))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `POST /api/v1/agents/:name/trigger` — fire an agent via CLI or agent-to-agent trigger.
    ///
    /// Returns the accepted run_id.
    pub async fn trigger_agent(
        &self,
        agent: &str,
        input: &str,
        caller: Option<&str>,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "payload": { "input": input },
            "caller": caller.unwrap_or("cli"),
        });
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/agents/{}/trigger", agent)))
            .json(&body)
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        check_status(resp).await?.json().await.map_err(Into::into)
    }

    /// `POST /api/v1/agents/:name/run?format=json` — run an agent and stream NDJSON.
    pub async fn stream_run(
        &self,
        agent: &str,
        input: &str,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/agents/{}/run?format=json", agent)))
            .json(&serde_json::json!({ "input": input }))
            .send()
            .await
            .map_err(|e| gateway_err(&self.base_url, e))?;
        let resp = check_status(resp).await?;
        Ok(resp.bytes_stream())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_connection_error(e: &reqwest::Error) -> bool {
    e.is_connect() || e.is_timeout()
}

/// Convert a reqwest connection error into a human-friendly "gateway unreachable" error.
fn gateway_err(base_url: &str, e: reqwest::Error) -> anyhow::Error {
    if is_connection_error(&e) {
        anyhow!(
            "{UNREACHABLE_PREFIX}:{base_url}"
        )
    } else {
        anyhow!("{}", e)
    }
}

/// Returns `true` if the error was produced by `gateway_err` (i.e. gateway is down).
pub fn is_gateway_unreachable(e: &anyhow::Error) -> Option<String> {
    let msg = e.to_string();
    if let Some(rest) = msg.strip_prefix(&format!("{UNREACHABLE_PREFIX}:")) {
        Some(rest.to_string())
    } else {
        None
    }
}

/// Assert that the response status is 2xx, otherwise convert to an error.
async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let url = resp.url().to_string();
    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    let message = body["error"]
        .as_str()
        .unwrap_or("unknown error")
        .to_string();
    match status {
        StatusCode::NOT_FOUND => Err(anyhow!("Not found: {}", url)),
        StatusCode::CONFLICT => Err(anyhow!("Conflict: {}", message)),
        StatusCode::BAD_REQUEST => Err(anyhow!("Bad request: {}", message)),
        _ => Err(anyhow!("Gateway returned {}: {}", status, message)),
    }
}
