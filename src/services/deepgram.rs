use reqwest::Client;
use serde::{Deserialize, Serialize}; // For transformation JSOM <=> Struct
use std::error::Error;

#[derive(Debug, Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Debug, Deserialize)]
struct Results {
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
struct Channel {
    alternatives: Vec<Alternative>,
}

// Main Structure Service
#[derive(Debug, Deserialize)]
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
    // Box<dyn Error> it is "any error"
    pub async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String, Box<dyn Error>> {
        let url = "https://api.deepgram.com/v1/listen?model=nova-3&smart_format=true";

        let response = self.client
            .post(url)
            .header("Authorization", format!("Token {}", self.api_key))
            .body(audio_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Deepgram API error: {}", response.status()).into());
        }

        let parsed: DeepgramResponse = response.json().await?;

        let transcript = parsed.results.channels
            .first().and_then(|c| c.alternatives.first())
            .map(|a| a.transcript.clone())
            .unwrap_or_default();

        Ok(transcript)
    }
}