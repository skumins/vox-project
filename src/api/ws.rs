use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use crate::{AppState, prompts::lecture_prompt};


pub async fn ws_handler(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>,
) -> impl IntoResponse {
    // upgrade the connection and tell it what to do next
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("WebSocket connected");

    // We start a Deepgram streaming session and get two channels
    let (audio_tx, mut transcript_rx) = match state.deepgram.start_stream().await {
        Ok(channels) => channels,
        Err(e) => {
            let _ = socket.send(Message::Text(format!("error:{}", e).into())).await;
            return;
        }
    };

    let mut full_transcript= String::new();
    
    // In parallel, we read transcripts from Deepgram and send them to the client
    loop {
        tokio::select! {
            // Branch 1: Deepgram sent a text
            Some(transcript) = transcript_rx.recv() => {
                full_transcript.push_str(&transcript);
                full_transcript.push(' ');
                let _ = socket.send(Message::Text(format!("transcript:{}", transcript).into())).await;
            }

            // Branch 2: Browser sent audio or command
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(bytes) => {
                        if audio_tx.send(bytes.to_vec()).await.is_err() {
                            break;
                        }
                    }
                    Message::Text(cmd) => match cmd.as_str() {
                        "sumarize" => {
                            if full_transcript.trim().is_empty() {
                                let _ = socket.send(Message::Text("error:No transcript yet". into())).await;
                                continue;
                            }

                            let _ = socket.send(Message::Text("status:Processing...".into())).await;
    
                            match state.llm.summarize(full_transcript.clone(), lecture_prompt()).await {
                                Ok(s) => {
                                    let _ = socket.send(Message::Text(format!("summary:{}", s).into())).await;
                                }
                                Err(e) => {
                                    let _ = socket.send(Message::Text(format!("error:{}", e).into())).await;
                                }
                            }
                        }
                        "stop" => break, _ => {}
                    },
                    Message::Close(_) => break, _ => {}
                }
            }
            else => break,
        }
    }
    println!("Client disconnected");
}