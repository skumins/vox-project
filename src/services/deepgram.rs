use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use reqwest::Client;

#[derive(Clone)]
pub struct DeepgramService {
    client: Client,
    api_key: String,
}

impl DeepgramService {
    pub fn new(api_key: String) -> Self { 
        Self { 
            client: Client::new(), 
            api_key 
        }
    }

    // Start a streaming session with Deepgram. 
    // Returns 2 channels; audio_tx - send audio bytes here,   text_rx - read transcript from here.
    pub async fn start_stream( &self ) -> Result<(mpsc::Sender<Vec<u8>>, mpsc::Receiver<String>), String> {
        let url = format!(
            "wss://api.deepgram.com/v1/listen\
             ?encoding=linear16\
             &sample_rate=16000\
             &interim_results=true\
             &punctuate=true\
             &keepalive=true\
             &model=nova-2\
             &detect_language=true"
        );

        let request = {
            use tokio_tungstenite::tungstenite::client::IntoClientRequest;
            let mut req = url.into_client_request().map_err(|e| e.to_string())?;
            req.headers_mut().insert("Authorization", format!("Token {}", self.api_key).parse().unwrap(),);
            req
        };

        // Connecting to Deepgram WebSocket; ws_stream is a two-way connection
        let (ws_stream, _) = connect_async(request).await.map_err(|e| format!("Deepgram WS connect failed: {}", e))?;

        // Divide it into two parts: sink - for sending (write end);  stream - for receiving (read end)
        let (mut sink, mut stream) = ws_stream.split();

        // Buffered audio and transcript channel
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(100);
        let (text_tx, text_rx) = mpsc::channel::<String>(100);

        // Thread 1: Sending audio to Deepgram
        tokio::spawn(async move {
            while let Some(audio_bytes) = audio_rx.recv().await {
                let msg = Message::Binary(audio_bytes.into());
                if sink.send(msg).await.is_err() {
                    eprintln!("Deepgram connection dropped while sending audio.");
                    break;
                }
            }
            let close_msg = Message::Text(r#"{"type":"CloseStream"}"#.to_string().into());
            let _ = sink.send(close_msg).await;
        });

        // Thread 2: Getting transcript from Deepgram
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    if let Some(transcript) = parse_deepgram_response(&text) {
                        if !transcript.is_empty() {
                            if text_tx.send(transcript).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        Ok((audio_tx, text_rx))
    }

    pub async fn transcribe(&self, audio_data: Vec<u8>, content_type: &str) -> Result<String, Box<dyn std::error::Error>> {
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

        #[derive(serde::Deserialize)]
        struct Resp { results: Results }
        #[derive(serde::Deserialize)]
        struct Results { channels: Vec<Channel> }
        #[derive(serde::Deserialize)]
        struct Channel { alternatives: Vec<Alt> }
        #[derive(serde::Deserialize)]
        struct Alt { transcript: String }

        let parsed: Resp = response.json().await?;
        Ok(parsed.results.channels
            .first()
            .and_then(|c| c.alternatives.first())
            .map(|a| a.transcript.clone())
            .unwrap_or_default())
    }
}

fn parse_deepgram_response(json_text: &str) -> Option<String> {

    #[derive(serde::Deserialize)]
    struct StreamResult {
        #[serde(rename = "type")]
        msg_type: String,
        is_final: Option<bool>,
        channel: Option<Channel>,
    }

    #[derive(serde::Deserialize)]
    struct Channel {
        alternatives: Vec<Alternative>,
    }

    #[derive(serde::Deserialize)]
    struct Alternative {
        transcript: String,
    }

    let result: StreamResult = serde_json::from_str(json_text).ok()?;

    if result.msg_type != "Results" {
        return None;
    }

    if result.is_final != Some(true) {
        return None;
    }

    let transcript = result.channel?
        .alternatives
        .into_iter()
        .next()?
        .transcript;

    if transcript.trim().is_empty() {
        None
    } else {
        Some(transcript)
    }

}