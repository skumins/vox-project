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

    let(transcript_tx, mut transcript_rx) = tokio::sync::mpsc::channel::<(u32, String)>(32);

    let segment_counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    let mut full_transcript= String::new();
    let mut llm_rx: Option<tokio::sync::oneshot::Receiver<String>> = None;

    let mut pending: std::collections::BTreeMap<u32, String> = std::collections::BTreeMap::new();
    let mut next_expected: u32 = 0;
    

    loop {
        tokio::select! {
            Some((seq, text)) = transcript_rx.recv() => {
                pending.insert(seq, text);

                while let Some(t) = pending.remove(&next_expected) {
                    if !t.is_empty() {
                        full_transcript.push_str(&t);
                        full_transcript.push(' ');
                        let _ = socket.send(Message::Text(format!("transcript:final:{}", t).into())).await;
                    }
                    next_expected += 1;
                }
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

            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(bytes) => {
                        let deepgram = state.deepgram.clone();
                        let tx = transcript_tx.clone();
                        let lang = transcript_lang.clone();

                        let counter = segment_counter.clone();
                        let seq = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        tracing::debug!("Sending segment #{} to Deepgram ({} bytes)", seq, bytes.len());

                        tokio::spawn(async move {
                            match deepgram.transcribe_with_lang(
                                bytes.to_vec(),
                                "audio/wav",
                                &lang,
                            ).await {
                                Ok(text) if !text.trim().is_empty() => {
                                    let _ = tx.send((seq, text)).await;
                                }
                                Ok(_) => { let _ = tx.send((seq, String::new())).await; } 
                                Err(e) => { 
                                    tracing::error!("Segment #{} error: {}", seq, e);
                                    let _ = tx.send((seq, String::new())).await;
                                }
                            }
                        });
                    }

                    Message::Text(cmd) => {
                        tracing::info!("Command: {}", cmd);
                        match cmd.as_str() {
                            "summarize" => {
                                if full_transcript.trim().is_empty() {
                                    let _ = socket.send(Message::Text("error:No transcript yet".into())).await;
                                    continue;
                                }

                                if llm_rx.is_some() { continue; }

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