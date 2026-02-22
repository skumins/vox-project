use axum::{
    extract::{Multipart, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::models::TranscribeResponse;
use crate::AppState;
use crate::prompts::lecture_prompt;
use uuid::Uuid;

pub async fn transcribe_audio(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {

    let mut audio_data = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            if let Ok(bytes) = field.bytes().await {
                audio_data = bytes.to_vec();
            }
        }
    }

    if audio_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No audio file uploaded").into_response();
    }

    println!("Audio received: {} bytes. Processing...", audio_data.len());

    let raw_text = match state.deepgram.transcribe(audio_data).await {
        Ok(text) => text,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Deepgram Error: {}", e)).into_response(),
    };

    println!("Transcription completed: {} characters", raw_text.len());

    let prompt = lecture_prompt();
    let processed_markdown = match state.llm.summarize(raw_text.clone(), prompt).await {
        Ok(text) => text,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM Error: {}", e)).into_response(),
    };

    println!("LLM processed completion:");

    let id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO notes (id, raw_text, processed_markdown, created_at) VALUES (?, ?, ?, datetime('now'))",
        id, raw_text, processed_markdown
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            println!("Saved to Database with ID: {}", id);
            Json(TranscribeResponse {
                id,
                status: "success".to_string(),
            }).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database Error: {}", e)).into_response(),
    }
}