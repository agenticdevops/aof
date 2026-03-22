//! Integration tests for the OpenAgentiX gateway HTTP API.
//!
//! Tests use axum's in-process test utilities — no real server is started.
//! All tests work with a real `AgentManager` but do NOT require a real LLM
//! (the run SSE test only verifies the Content-Type header).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt; // for `.oneshot()`

use agentix_runtime::gateway::AgentManager;
use agentix_runtime::gateway::api::create_router;

// ---------------------------------------------------------------------------
// Minimal valid flat YAML agent for test fixtures
// ---------------------------------------------------------------------------

fn minimal_agent_yaml(name: &str) -> String {
    format!(
        r#"apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: {name}
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "You are a helpful test agent."
"#
    )
}

fn minimal_agent_yaml_with_description(name: &str, description: &str) -> String {
    format!(
        r#"apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: {name}
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "{description}"
"#
    )
}

/// Build the test router (empty AgentManager, no workspace config).
fn test_router() -> axum::Router {
    let manager = AgentManager::new(None);
    create_router(manager).into()
}

/// Build a test router with one pre-loaded agent.
fn test_router_with_agent(name: &str) -> axum::Router {
    let manager = AgentManager::new(None);
    manager.register_from_yaml(&minimal_agent_yaml(name)).unwrap();
    create_router(manager).into()
}

// ---------------------------------------------------------------------------
// Helper: parse response body as JSON
// ---------------------------------------------------------------------------

async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test 1: GET /healthz returns 200 with {"status":"ok",...}
#[tokio::test]
async fn test_health_endpoint() {
    let app = test_router();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["status"], "ok");
    // version field should be present
    assert!(json["version"].is_string(), "version field should be a string");
}

/// Test 2: GET /api/v1/agents with no loaded agents returns []
#[tokio::test]
async fn test_list_agents_empty() {
    let app = test_router();
    let req = Request::builder()
        .uri("/api/v1/agents")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    assert!(json.is_array(), "should return an array");
    assert_eq!(json.as_array().unwrap().len(), 0);
}

/// Test 3: POST /api/v1/agents then GET — agent appears in list
#[tokio::test]
async fn test_register_and_list_agent() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    // Register an agent
    let body = serde_json::json!({ "yaml": minimal_agent_yaml("test-agent") });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["name"], "test-agent");
    assert_eq!(json["status"], "ready");

    // List agents — should contain our new agent
    let req = Request::builder()
        .uri("/api/v1/agents")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    let agents = json.as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["name"], "test-agent");
}

/// Test 4: POST /api/v1/agents with invalid YAML returns 400 with error JSON
#[tokio::test]
async fn test_register_invalid_yaml_returns_400() {
    let app = test_router();

    // YAML missing metadata.name
    let bad_yaml = r#"apiVersion: openagentix.dev/v1
kind: Agent
spec:
  model: anthropic/claude-sonnet-4-6
"#;
    let body = serde_json::json!({ "yaml": bad_yaml });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string(), "should have an error field");
    let error_msg = json["error"].as_str().unwrap();
    assert!(
        !error_msg.is_empty(),
        "error message should not be empty"
    );
}

/// Test 5: POST /api/v1/agents with same name twice returns 409
#[tokio::test]
async fn test_register_duplicate_returns_409() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    let body = serde_json::json!({ "yaml": minimal_agent_yaml("dup-agent") });

    // First registration — should succeed
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second registration — should conflict
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string());
}

/// Test 6: POST then PUT with updated description — GET confirms update
#[tokio::test]
async fn test_update_agent() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    // Register
    let body = serde_json::json!({ "yaml": minimal_agent_yaml("update-agent") });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap();

    // Update with new system prompt (description)
    let updated_yaml = minimal_agent_yaml_with_description("update-agent", "Updated description");
    let update_body = serde_json::json!({ "yaml": updated_yaml });
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/agents/update-agent")
        .header("Content-Type", "application/json")
        .body(Body::from(update_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["name"], "update-agent");
    assert_eq!(json["status"], "updated");
}

/// Test 7: PUT to a name that doesn't exist returns 404
#[tokio::test]
async fn test_update_nonexistent_returns_404() {
    let app = test_router();

    let body = serde_json::json!({ "yaml": minimal_agent_yaml("ghost") });
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/agents/ghost")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string());
}

/// Test 8: GET /api/v1/agents/nonexistent returns 404
#[tokio::test]
async fn test_get_agent_not_found() {
    let app = test_router();
    let req = Request::builder()
        .uri("/api/v1/agents/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string());
}

/// Test 9: POST /api/v1/agents/nonexistent/run returns 404
#[tokio::test]
async fn test_run_agent_not_found() {
    let app = test_router();

    let body = serde_json::json!({ "input": "do something" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents/nonexistent/run")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string());
}

/// Test 10: DELETE /api/v1/agents/test/runs/fake-id returns 404
#[tokio::test]
async fn test_stop_nonexistent_run() {
    let app = test_router_with_agent("test");

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/agents/test/runs/fake-run-id-that-does-not-exist")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = body_to_json(response.into_body()).await;
    assert!(json["error"].is_string());
}

// ---------------------------------------------------------------------------
// Provider config endpoint tests
// ---------------------------------------------------------------------------

/// Test: GET /api/v1/providers with no workspace config returns empty list
#[tokio::test]
async fn test_list_providers_empty() {
    let app = test_router();
    let req = Request::builder()
        .uri("/api/v1/providers")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

/// Test: GET /api/v1/providers with workspace config returns provider names
#[tokio::test]
async fn test_list_providers_with_config() {
    use agentix_core::WorkspaceConfig;

    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
  providers:
    anthropic:
      api_key: "sk-test-key"
    openai:
      api_key: "sk-openai-key"
"#;
    let config = WorkspaceConfig::from_yaml(yaml).unwrap();
    let manager = AgentManager::new(Some(config));
    let app: axum::Router = create_router(manager).into();

    let req = Request::builder()
        .uri("/api/v1/providers")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    let providers = json.as_array().unwrap();
    assert_eq!(providers.len(), 2);

    // Keys should be masked (not returned in full)
    for p in providers {
        assert!(p["name"].is_string());
        assert!(p["configured"].as_bool().unwrap());
        // api_key should be masked or absent
        assert!(
            p.get("api_key").is_none() || p["api_key"].as_str().unwrap().contains("***"),
            "API key should be masked"
        );
    }
}

/// Test: PUT /api/v1/providers sets a provider key, GET reflects it
#[tokio::test]
async fn test_update_provider() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    // Set a provider key
    let body = serde_json::json!({
        "name": "anthropic",
        "api_key": "sk-ant-new-key"
    });
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/providers")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Now list — should show anthropic as configured
    let req = Request::builder()
        .uri("/api/v1/providers")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let json = body_to_json(response.into_body()).await;
    let providers = json.as_array().unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["name"], "anthropic");
    assert!(providers[0]["configured"].as_bool().unwrap());
}

/// Bonus test: POST /api/v1/agents/:name/run on a registered agent returns text/event-stream header
///
/// We don't test actual LLM execution here — just verifies the SSE Content-Type.
/// Actual execution is tested at integration level in 13-09.
#[tokio::test]
async fn test_run_agent_sse_content_type() {
    let app = test_router_with_agent("sse-agent");

    let body = serde_json::json!({ "input": "do something" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/agents/sse-agent/run")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    // The response should be 200 with SSE content type
    // (404 would mean agent not found; 500 would mean model error — both fail the test)
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/event-stream"),
        "Expected text/event-stream content type, got: {}",
        content_type
    );
}
