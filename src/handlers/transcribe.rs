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

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            match field.bytes().await {
                Ok(bytes) => {
                    audio_data = bytes.to_vec();
                    println!("File received: {} bytes", audio_data.len());
                }
                Err(e) => {
                    return (StatusCode::BAD_REQUEST, format!("Error reading file: {}", e), ).into_response();
                }
            }
            break; // Exit because file found
        }
    }

    if audio_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No audio file uploaded. Make sure the field is named 'file'.".to_string(), ).into_response();
    }

    println!("Audio received: {} bytes. Processing...", audio_data.len());

    let raw_text = match state.deepgram.transcribe(audio_data).await {
        Ok(text) => {
            println!("Transcription successful: {} characters", text.len());
            text
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Deepgram Error: {}", e)).into_response(),
        }
    };

    println!("Sending in LLM...");
    let prompt = lecture_prompt();
    let processed_markdown = match state.llm.summarize(raw_text.clone(), prompt).await {
        Ok(text) => {
            println!("LLM processing successful");
            text
        },
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM Error: {}", e)).into_response();
    }
    };

    let id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO notes (id, raw_text, processed_markdown, created_at) VALUES (?, ?, ?, datetime('now'))",
        id, raw_text, processed_markdown,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            println!("Saved to Database with ID: {}", id);
            (StatusCode::OK, Json(TranscribeResponse {
                id,
                status: "success".to_string(),
            })).into_response()
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Save error in Database: {}", e), ).into_response()
        }
    }
}