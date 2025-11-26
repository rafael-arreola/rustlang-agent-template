use crate::{
    api::request::{ChatRequest, ChatResponse},
    infra::redis::{ChatMessage, Role},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use uuid::Uuid;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // 1. Session Management
    let session_id = payload
        .session_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 2. Load History
    let history = state
        .redis
        .get_history(&session_id)
        .await
        .unwrap_or_default();

    // 3. Process with Orchestrator (with history)
    // The chat method currently handles the prompt + history interaction
    let response_text = state.orchestrator.chat(&payload.prompt, history).await;

    // 4. Save History (User + Assistant)
    let new_messages = vec![
        ChatMessage {
            role: Role::User,
            content: payload.prompt.clone(),
        },
        ChatMessage {
            role: Role::Assistant,
            content: response_text.clone(),
        },
    ];

    if let Err(e) = state.redis.add_messages(&session_id, new_messages).await {
        tracing::warn!("Failed to save chat history: {}", e);
    }

    // 5. Response
    (
        StatusCode::OK,
        Json(ChatResponse {
            response: response_text,
            session_id,
        }),
    )
        .into_response()
}
