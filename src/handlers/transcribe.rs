  use axum::{
    extract::{Multipart, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::{models::TranscribeResponse, prompts::lecture_prompt, AppState};
use uuid::Uuid;

pub async fn transcribe_audio(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {

    let mut audio_data: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or("") == "file" {
            match field.bytes().await {
                Ok(bytes) => {
                    audio_data = bytes.to_vec();
                    println!("File received: {} bytes", audio_data.len());
                }
                Err(e) => {
                    return (StatusCode::BAD_REQUEST, format!("Error reading file: {}", e)).into_response();
                }
            }
            break; // Exit because file found
        }
    }

    if audio_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No audio file uploaded. Make sure the field is named 'file'.".to_string()).into_response();
    }

    println!("Audio received: {} bytes. Processing...", audio_data.len());

    let transcript = match state.deepgram.transcribe(audio_data, "audio/wav").await {
        Ok(text) => {
            println!("Transcription successful: {} characters", text.len());
            text
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Deepgram Error: {}", e)).into_response();
        }
    };

    println!("Sending in LLM...");
    let summary = match state.llm.summarize(transcript.clone(), lecture_prompt()).await {
        Ok(text) => {
            println!("LLM processing successful");
            text
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM Error: {}", e)).into_response();
    }
    };

    // temporary solution for debugging 
    println!("LLM response for testing:\n{}", summary);

    let id = Uuid::new_v4().to_string();

    match sqlx::query!(
        "INSERT INTO notes (id, raw_text, processed_markdown, created_at) VALUES (?, ?, ?, datetime('now'))",
        id, transcript, summary,
    )
    .execute(&state.db)
    .await
    
    {
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