use reqwest::Client;
use serde::{Deserialize}; // For transformation JSON
use std::error::Error;

#[derive(Debug, Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Debug, Deserialize)]
struct DeepgramResults {
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Debug, Deserialize)]
struct Alternative {
    transcript: String,
}

// Main Structure Service
#[derive(Debug, Clone)]
pub struct DeepgramService {
    client: Client,
    api_key: String,
}

impl DeepgramService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    // Accept audio bytes and return text
    pub async fn transcribe(&self, audio_data: Vec<u8>, content_type: &str) -> Result<String, Box<dyn Error>> {
        let url = "https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&punctuate=true&detect_language=true";

        let response = self.client
            .post(url)
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Content-Type", content_type)
            .body(audio_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Deepgram {}: {}", status, body).into());
        }

        let parsed: DeepgramResponse = response.json().await?;

        let transcript = parsed.results.channels
            .first()
            .and_then(|c| c.alternatives.first())
            .map(|a| a.transcript.clone())
            .unwrap_or_default();

        Ok(transcript)
    }
}