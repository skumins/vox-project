  use axum::{
    extract::{Multipart, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::{models::TranscribeResponse, prompts::lecture_prompt_with_lang, AppState};
use uuid::Uuid;

pub async fn transcribe_audio(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {

    let mut audio_data: Vec<u8> = Vec::new();
    let mut lang = String::from("en");

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "file" => {
                match field.bytes().await {
                    Ok(bytes) => {
                        audio_data = bytes.to_vec();
                        tracing::debug!("File received: {} bytes", audio_data.len());
                    }
                    Err(e) => {
                        return (StatusCode::BAD_REQUEST, format!("Error reading file: {}", e)).into_response();
                    }
                }
            }
            "lang" => {
                if let Ok(value) = field.text().await {
                    lang = value;
                }
            }
            _ => {}
        }
    }

    if audio_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No audio file uploaded. Make sure the field is named 'file'.".to_string()).into_response();
    }

    tracing::info!("Audio received: {} bytes", audio_data.len());

    let transcript = match state.deepgram.transcribe(audio_data, "audio/wav").await {
        Ok(text) => {
            tracing::info!("Transcription done: {} chars", text.len());
            text
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Deepgram Error: {}", e)).into_response();
        }
    };

    tracing::info!("Sending transcript to LLM");
    let summary = match state.llm.summarize(
        transcript.clone(), 
        lecture_prompt_with_lang(&lang),
    ).await {
        Ok(text) => {
            tracing::info!("LLM processing done");
            text
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM Error: {}", e)).into_response();
    }
    };

    let id = Uuid::new_v4();

    match sqlx::query!(
        "INSERT INTO notes (id, raw_text, processed_markdown) VALUES ($1, $2, $3)",
        id, transcript, summary,
    )
    .execute(&state.db)
    .await
    
    {
        Ok(_) => {
            tracing::info!("Note saved, ID = {}", id);
            (StatusCode::OK, Json(TranscribeResponse {
                id: id.to_string(),
                status: "success".to_string(),
            })).into_response()
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Save error in Database: {}", e), ).into_response()
        }
    }
}