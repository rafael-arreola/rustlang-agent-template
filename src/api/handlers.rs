use crate::{
    api::request::{ChatRequest, ChatResponse},
    infra::{
        errors::{DomainError, DomainResult},
        redis::{ChatMessage, Role},
    },
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
) -> Result<impl IntoResponse, DomainError> {
    let prompt = validate_prompt(&payload.prompt)?;

    let session_id = payload
        .session_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let history = state
        .redis
        .get_history(&session_id)
        .await
        .unwrap_or_default();

    let response_text = state.orchestrator.chat(&prompt, history).await;

    let new_messages = vec![
        ChatMessage {
            role: Role::User,
            content: prompt,
        },
        ChatMessage {
            role: Role::Assistant,
            content: response_text.clone(),
        },
    ];

    if let Err(e) = state.redis.add_messages(&session_id, new_messages).await {
        tracing::warn!("Failed to save chat history: {}", e);
    }

    Ok((
        StatusCode::OK,
        Json(ChatResponse {
            response: response_text,
            session_id,
        }),
    ))
}

fn validate_prompt(prompt: &str) -> DomainResult<String> {
    let trimmed = prompt.trim();

    if trimmed.is_empty() {
        return Err(DomainError::validation("El prompt no puede estar vacío"));
    }

    if trimmed.len() > 10_000 {
        return Err(DomainError::validation(
            "El prompt excede el límite de 10,000 caracteres",
        ));
    }

    Ok(trimmed.to_string())
}
