//! Integration tests for the OAuth auth REST endpoints (Phase 23, Plan 04).
//!
//! Tests verify:
//! - `GET /api/v1/auth/:provider/status` returns unauthenticated when no tokens stored
//! - Provider name normalization in auth endpoints ("google" and "gemini" map to same provider)
//! - `DELETE /api/v1/auth/:provider` removes a stored token (status goes to false)
//! - `AgentManager` with subscription-mode provider returns clear error when no token stored

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use agentix_runtime::gateway::AgentManager;
use agentix_runtime::gateway::api::create_router;

/// Build a test router backed by an AgentManager with no workspace config.
fn test_router() -> axum::Router {
    let manager = AgentManager::new(None);
    create_router(manager).into()
}

// ---------------------------------------------------------------------------
// Helper: parse JSON response body
// ---------------------------------------------------------------------------

async fn parse_json(body: axum::body::Body) -> serde_json::Value {
    let bytes = body.collect().await.expect("collect body").to_bytes();
    serde_json::from_slice(&bytes).expect("parse JSON")
}

// ---------------------------------------------------------------------------
// Test: auth status returns not-authenticated when no tokens are stored
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_status_returns_not_authenticated() {
    let app = test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/openai/status")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = parse_json(resp.into_body()).await;
    assert_eq!(json["provider"], "openai");
    assert_eq!(json["authenticated"], false);
}

// ---------------------------------------------------------------------------
// Test: provider name normalization — "google" and "gemini" both work
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provider_normalization_google_and_gemini() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    // "google" should return status for the gemini provider
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/google/status")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = parse_json(resp.into_body()).await;
    // provider field should be normalized to "gemini"
    assert_eq!(json["provider"], "gemini");
    assert_eq!(json["authenticated"], false);
}

// ---------------------------------------------------------------------------
// Test: POST /api/v1/auth/:provider/token then DELETE clears the token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_disconnect_removes_profile() {
    let manager = AgentManager::new(None);
    let app: axum::Router = create_router(manager).into();

    // Store an Anthropic token
    let store_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/anthropic/token")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"token":"sk-ant-test-1234"}"#))
        .unwrap();

    let store_resp = app.clone().oneshot(store_req).await.unwrap();
    assert!(
        store_resp.status().is_success(),
        "Expected 2xx from token store, got {}",
        store_resp.status()
    );

    // Verify it's now authenticated
    let status_req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/anthropic/status")
        .body(Body::empty())
        .unwrap();

    let status_resp = app.clone().oneshot(status_req).await.unwrap();
    assert_eq!(status_resp.status(), StatusCode::OK);
    let json = parse_json(status_resp.into_body()).await;
    assert_eq!(json["authenticated"], true, "Expected authenticated after storing token");

    // Delete the profile
    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/auth/anthropic")
        .body(Body::empty())
        .unwrap();

    let delete_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(
        delete_resp.status(),
        StatusCode::NO_CONTENT,
        "Expected 204 from disconnect"
    );

    // Verify it's now unauthenticated
    let status_req2 = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/anthropic/status")
        .body(Body::empty())
        .unwrap();

    let status_resp2 = app.clone().oneshot(status_req2).await.unwrap();
    let json2 = parse_json(status_resp2.into_body()).await;
    assert_eq!(
        json2["authenticated"],
        false,
        "Expected unauthenticated after disconnect"
    );
}

// ---------------------------------------------------------------------------
// Test: auth start endpoint returns auth_url for openai
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_start_openai_returns_auth_url() {
    let app = test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/openai/start")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = parse_json(resp.into_body()).await;
    assert_eq!(json["provider"], "openai");
    // auth_url should be a non-null string starting with https://
    let auth_url = json["auth_url"].as_str().expect("auth_url should be a string");
    assert!(
        auth_url.starts_with("https://auth.openai.com"),
        "Expected OpenAI auth URL, got: {}",
        auth_url
    );
}

// ---------------------------------------------------------------------------
// Test: Anthropic start returns token-paste instructions (no auth_url)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_start_anthropic_returns_token_instructions() {
    let app = test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/anthropic/start")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = parse_json(resp.into_body()).await;
    assert_eq!(json["provider"], "anthropic");
    assert_eq!(json["method"], "token");
    // auth_url should be null for Anthropic (token paste flow)
    assert!(json["auth_url"].is_null(), "Anthropic should not have auth_url");
}

// ---------------------------------------------------------------------------
// Test: subscription mode — AuthService returns None when no token stored
//
// This tests the precondition that triggers the subscription error in
// `create_provider_from_definition`: when ProviderMode::Subscription is
// configured but no token is in the AuthService, the function returns an error.
//
// We test the precondition (no token → get_valid_openai_access_token = None)
// rather than testing through the private execute_run pathway.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscription_mode_no_token_in_auth_service() {
    use agentix_core::AuthService;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let auth_service = AuthService::new(tmp.path(), false);

    // No profiles stored — should return None for OpenAI
    let result = auth_service.get_valid_openai_access_token(None).await;
    assert!(result.is_ok(), "Should not error on missing profile");
    assert!(result.unwrap().is_none(), "Should return None when no OpenAI profile stored");

    // Same for Gemini
    let result = auth_service.get_valid_gemini_access_token(None).await;
    assert!(result.is_ok(), "Should not error on missing Gemini profile");
    assert!(result.unwrap().is_none(), "Should return None when no Gemini profile stored");

    // Same for Anthropic bearer token
    let result = auth_service.get_provider_bearer_token("anthropic", None).await;
    assert!(result.is_ok(), "Should not error on missing Anthropic profile");
    assert!(result.unwrap().is_none(), "Should return None when no Anthropic profile stored");
}

// ---------------------------------------------------------------------------
// Test: AgentManager has auth_service accessible
// ---------------------------------------------------------------------------

#[tokio::test]
async fn agent_manager_has_auth_service() {
    let manager = agentix_runtime::gateway::AgentManager::new(None);
    // Verify auth_service is accessible and functional
    let result = manager.auth_service.load_profiles().await;
    assert!(result.is_ok(), "AuthService should load profiles without error");
    let profiles = result.unwrap();
    assert_eq!(profiles.profiles.len(), 0, "Fresh AuthService should have no profiles");
}
