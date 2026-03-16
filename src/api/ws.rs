use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use crate::{AppState, prompts::lecture_prompt};


pub async fn ws_handler(ws: WebSocketUpgrade, 
    State(state): State<AppState>,
) -> impl IntoResponse {
    // upgrade the connection and tell it what to do next
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("WebSocket connected");
    let mut full_transcript= String::new();
    
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