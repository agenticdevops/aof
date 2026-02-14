//! Conversation API endpoints for natural language agent creation
//!
//! This module provides REST endpoints for the conversational interface:
//! - POST /api/conversation/session - Create new conversation session
//! - GET /api/conversation/session/:id - Get session details
//! - POST /api/conversation/message - Send message and get response
//! - POST /api/conversation/confirm - Confirm and persist generated files
//! - POST /api/conversation/cancel - Cancel pending file generation

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use aof_conversational::{
    Orchestrator, WorkspacePersistence, OrchestratorResponse,
    ConversationMessage,
};

/// Shared state for conversation API
#[derive(Clone)]
pub struct ConversationState {
    pub orchestrator: Arc<RwLock<Orchestrator>>,
    pub persistence: Arc<WorkspacePersistence>,
}

/// Request to create a new conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {}

/// Response from session creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

/// Request to send a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessageRequest {
    pub session_id: String,
    pub message: String,
}

/// Response from message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessageResponse {
    pub session_id: String,
    pub response: OrchestratorResponse,
    pub messages: Vec<ConversationMessage>,
}

/// Request to confirm generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfirmRequest {
    pub session_id: String,
}

/// Response from file confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfirmResponse {
    pub files_written: Vec<String>,
    pub message: String,
}

/// Request to cancel pending files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationCancelRequest {
    pub session_id: String,
}

/// Session details response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub messages: Vec<ConversationMessage>,
    pub created_at: String,
}

/// API errors
#[derive(Debug)]
pub enum ApiError {
    SessionNotFound,
    InvalidInput(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::SessionNotFound => (StatusCode::NOT_FOUND, "Session not found".to_string()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": message,
        });

        (status, Json(body)).into_response()
    }
}

/// POST /api/conversation/session - Create new conversation session
pub async fn create_session(
    State(state): State<ConversationState>,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    let mut orchestrator = state.orchestrator.write().await;

    let session_id = orchestrator.create_session()
        .map_err(|e| ApiError::InternalError(format!("Failed to create session: {}", e)))?;

    info!(session_id = %session_id, "Created new conversation session");

    Ok(Json(CreateSessionResponse { session_id }))
}

/// GET /api/conversation/session/:id - Get session details
pub async fn get_session(
    State(state): State<ConversationState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, ApiError> {
    let orchestrator = state.orchestrator.read().await;

    let session = orchestrator.get_session(&session_id)
        .ok_or(ApiError::SessionNotFound)?;

    Ok(Json(SessionResponse {
        session_id: session.id.clone(),
        messages: session.messages.clone(),
        created_at: session.created_at.to_rfc3339(),
    }))
}

/// POST /api/conversation/message - Send message and get response
pub async fn conversation_message(
    State(state): State<ConversationState>,
    Json(req): Json<ConversationMessageRequest>,
) -> Result<Json<ConversationMessageResponse>, ApiError> {
    info!(
        session_id = %req.session_id,
        message_len = req.message.len(),
        "Processing conversation message"
    );

    let mut orchestrator = state.orchestrator.write().await;

    // Handle message through orchestrator
    let response = orchestrator
        .handle_message(&req.session_id, &req.message)
        .await
        .map_err(|e| ApiError::InternalError(format!("Orchestrator error: {}", e)))?;

    // Get updated message history
    let session = orchestrator.get_session(&req.session_id)
        .ok_or(ApiError::SessionNotFound)?;

    info!(
        session_id = %req.session_id,
        response_type = ?response,
        "Conversation message processed"
    );

    Ok(Json(ConversationMessageResponse {
        session_id: req.session_id,
        response,
        messages: session.messages.clone(),
    }))
}

/// POST /api/conversation/confirm - Confirm and persist generated files
pub async fn conversation_confirm(
    State(state): State<ConversationState>,
    Json(req): Json<ConversationConfirmRequest>,
) -> Result<Json<ConversationConfirmResponse>, ApiError> {
    info!(session_id = %req.session_id, "Confirming generated files");

    let mut orchestrator = state.orchestrator.write().await;

    // Get pending files from session
    let session = orchestrator.get_session(&req.session_id)
        .ok_or(ApiError::SessionNotFound)?;

    let pending_files = session.pending_files.clone()
        .ok_or_else(|| ApiError::InvalidInput("No pending files to confirm".to_string()))?;

    // Persist files to workspace
    let result = state.persistence
        .persist_files(&pending_files)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to persist files: {}", e)))?;

    // Clear pending files in session
    orchestrator.clear_pending_files(&req.session_id)
        .map_err(|e| ApiError::InternalError(format!("Failed to clear pending files: {}", e)))?;

    info!(
        session_id = %req.session_id,
        files_written = result.files_written.len(),
        files_modified = result.files_modified.len(),
        "Files persisted successfully"
    );

    let mut all_files = result.files_written.clone();
    all_files.extend(result.files_modified.clone());

    Ok(Json(ConversationConfirmResponse {
        files_written: all_files,
        message: format!(
            "Successfully created {} files and modified {} files",
            result.files_written.len(),
            result.files_modified.len()
        ),
    }))
}

/// POST /api/conversation/cancel - Cancel pending file generation
pub async fn conversation_cancel(
    State(state): State<ConversationState>,
    Json(req): Json<ConversationCancelRequest>,
) -> Result<Json<()>, ApiError> {
    info!(session_id = %req.session_id, "Cancelling pending files");

    let mut orchestrator = state.orchestrator.write().await;

    orchestrator.clear_pending_files(&req.session_id)
        .map_err(|e| ApiError::InternalError(format!("Failed to clear pending files: {}", e)))?;

    info!(session_id = %req.session_id, "Pending files cancelled");

    Ok(Json(()))
}
