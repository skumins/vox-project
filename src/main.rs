use std::net::SocketAddr;
use std::sync::Arc;
use axum::{routing::get, Router};
use tokio::sync::Mutex;
mod stt;
mod audio_capture;
mod api {
    pub mod ws;
}
use crate::stt::{MockStt, AudioConfig};

#[tokio::main]
async fn main() {
    // Initialize to see events in console
    tracing_subscriber::fmt::init();
    // Create shared state
    let stt_service = Arc::new(Mutex::new(MockStt));

    println!("Starting AI Audio Workspace");

    // Start local capture (cpal)
    let stt_for_capture = stt_service.clone();

    tokio::spawn(async move {
        println!("Local capture task started");

        if let Err(e) = audio_capture::audio_capture(stt_for_capture).await {
            eprintln!("Capture Error: {}", e);
        }
    });

    let app = Router::new()
        .route("/ws", get(api::ws::ws_handler))
        .with_state(stt_service);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
