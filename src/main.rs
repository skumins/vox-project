use std::net::SocketAddr;
use std::sync::Arc;
use axum::{routing::get, Router};
use tokio::sync::Mutex;
use sqlx::sqlite::SqlitePool;

mod stt;
mod audio_capture;
mod config;
mod api {
    pub mod ws;
}
use crate::stt::{MockStt, AudioConfig};
use crate::config::Config;

#[tokio::main]
async fn main() {
    // Initialize to see events in console
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            println!("Configuration Error: {}", e);
            return;
        }
    };
    println!("Configuration Loaded.");

    println!("Connecting to database...");
    let db_pool = SqlitePool::connect(&config.database_url).await.unwrap();
    sqlx::migrate!().run(&db_pool).await.unwrap();
    println!("Database connected and migrations applied.");

    // Create shared state
    let stt_service = Arc::new(Mutex::new(MockStt));

    println!(" System Starting ");

    // Start local capture (cpal)
    let stt_for_capture = stt_service.clone();

    tokio::spawn(async move {
        println!("Starting audio capture");

        if let Err(e) = audio_capture::start_capture(stt_for_capture).await {
            eprintln!("Microphone Error: {}", e);
        }
    });

    // STARTING THE WEB SERVER
    let app = Router::new()
        .route("/ws", get(api::ws::ws_handler))
        .with_state(stt_service);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
