use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use crate::AppState;


pub async fn ws_handler(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>,
) -> impl IntoResponse {
    // upgrade the connection and tell it what to do next
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("WebSocket connected");

    let (transcript_lang, summary_lang) = match socket.recv().await {
        Some(Ok(Message::Text(msg))) if msg.starts_with("config") => {
            #[derive(serde::Deserialize)]
            struct SessionConfig {
                lang: String,
                summary_lang: String,
            }
            let json = &msg["config:".len()..];
            match serde_json::from_str::<SessionConfig>(json) {
                Ok(cfg) => (cfg.lang, cfg.summary_lang),
                Err(_) => ("en".to_string(), "en".to_string()),
            }
        }
        _ => ("en".to_string(), "en".to_string()),
    };

    tracing::info!(transcript = %transcript_lang, summary = %summary_lang, "Session config received");
    // We start a Deepgram streaming session and get two channels
    let (audio_tx, mut transcript_rx) = match state.deepgram.start_stream(&transcript_lang).await {
        Ok(channels) => { tracing::info!("Deepgram stream started"); channels}
        Err(e) => {
            tracing::error!("Deepgram stream failed: {}", e);
            let _ = socket.send(Message::Text(format!("error:{}", e).into())).await;
            return;
        }
    };

    let mut full_transcript= String::new();
    let mut llm_rx: Option<tokio::sync::oneshot::Receiver<String>> = None;
    
    // In parallel, we read transcripts from Deepgram and send them to the client
    loop {
        tokio::select! {
            // Branch 1: Deepgram sent a text
            Some(transcript) = transcript_rx.recv() => {
                tracing::debug!("Transcript received: '{}'", transcript);
                full_transcript.push_str(
                    if transcript.starts_with("final:") {
                        &transcript["final:".len()..]
                    } else { "" }
                );
                if transcript.starts_with("final:"){
                    full_transcript.push(' ');
                }
                let _ = socket.send(Message::Text(format!("transcript:{}", transcript).into())).await;
            }

            Ok(result) = async {
                match &mut llm_rx {
                    Some(rx) => rx.await,
                    None => std::future::pending().await,
                }
            }, if llm_rx.is_some() => {
                llm_rx = None;
                let _ = socket.send(Message::Text(result.into())).await;
            }

            // Branch 2: Browser sent audio or command
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(bytes) => {
                        tracing::debug!("Audio chunk: {} bytes", bytes.len());
                        if audio_tx.send(bytes.to_vec()).await.is_err() {
                            tracing::warn!("Audio channel closed");
                            break;
                        }
                    }
                    Message::Text(cmd) => {
                        tracing::info!("Command: {}", cmd);
                        match cmd.as_str() {
                            "summarize" => {
                                if full_transcript.trim().is_empty() {
                                    let _ = socket.send(Message::Text("error:No transcript yet".into())).await;
                                    continue;
                                }

                                if llm_rx.is_some() {
                                    continue;
                                }

                                let _ = socket.send(Message::Text("status:Processing...".into())).await;
    
                                let text = full_transcript.clone();
                                let prompt = crate::prompts::lecture_prompt_with_lang(&summary_lang);
                                let llm =state.llm.clone();

                                let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                                llm_rx = Some(rx);

                                tokio::spawn(async move {
                                    let result = match llm.summarize(text, prompt).await {
                                        Ok(s) => format!("summary:{}", s),
                                        Err(e) => format!("error:LLM: {}", e),
                                    };
                                    let _ = tx.send(result);
                                });
                            }
                            "stop" => {
                                tracing::info!("Recording stopped, connection kept for summarize");
                                let _ = socket.send(Message::Text("status:Stopped".into())).await;
                            }
                            "disconnect" => break,
                            _ => {}
                        }
                    },
                    Message::Close(_) => { tracing::info!("Client closed connection"); break; } _ => {}
                }
            }
            else => {
                tracing::debug!("All channels closed, exiting loop");
                break;
            }
        }
    }
    tracing::info!("WebSocket disconnected");
}