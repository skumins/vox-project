use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::stt::Transcriber;

pub async fn ws_handler(ws: WebSocketUpgrade, 
    State(state): State<Arc<Mutex<dyn Transcriber>>>,
) -> impl IntoResponse {
    // upgrade the connection and tell it what to do next
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<dyn Transcriber>>) {
    println!("WebSocket connected");
    // loop to listen for messages from the browser
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            // here we can process audio sent from the website
            println!("Received WebSocket message: {:?}", msg);
        } else {
            // connection closed by user
            break;
        }
    }
}