use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct LlmResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Clone)]
pub struct OpenRouterService {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenRouterService {
    pub fn new(api_key: String, model: String) -> Self {
        // Build client with timeout because free LLM models can be slow
        let client = Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            model,
        }
    }

    pub async fn summarize(&self, text: String, system_prompt: String) -> Result<String, Box<dyn Error>> {
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ]
        });

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            // OpenRouter recommends setting your site url
            .header("HTTP-Referer", "http://localhost:3000")
            .header("X-Title", "VOXA")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            // For debugging
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("LLM API Error {}: {} ", status, error_text).into());
        }

        let parsed: LlmResponse = response.json().await?;

        let content = parsed.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or("No summary generated".to_string());

        Ok(content)
    }
}