use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
mod stt;
use crate::stt::{Transcriber, MockStt};

#[tokio::main]


async fn main() {
    let app = Router::new()
        .route("/ws", get(ws_handler));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoRensponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut stt_service = MockStt;
    let _ = stt_service.connect().await;

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Binary(data) = msg {
            let _ = stt_service.handle_audio(data).await;
        }
    }
}
