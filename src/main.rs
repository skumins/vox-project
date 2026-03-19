use std::net::SocketAddr;
use axum::{
    routing::post,
    routing:: get,
    Router,
    extract::DefaultBodyLimit,
};
use sqlx::sqlite::SqlitePool;

mod services;
mod handlers;
mod models;
mod prompts;
mod stt;
mod audio_capture;
mod config;
mod api;

use services::{deepgram::DeepgramService, llm::OpenRouterService};
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub deepgram: DeepgramService,
    pub llm: OpenRouterService,
}

#[tokio::main]
async fn main() {
    // Initialize to see events in console
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();
    
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env().expect("Error configuring");

    let db_pool = SqlitePool::connect(&config.database_url).await.expect("Failed to connect to database");
    sqlx::migrate!("./migrations").run(&db_pool).await.expect("Migration failed");

    let state = AppState{
        db: db_pool,
        deepgram: DeepgramService::new(config.deepgram_key.clone()),
        llm: OpenRouterService::new(config.openrouter_key.clone(), config.model.clone()),
    };
    
    println!("VOXA backend running...");

    let app = Router::new()
        .route("/ws", get(api::ws::ws_handler))
        .route("/transcribe", post(handlers::transcribe::transcribe_audio)
            .layer(DefaultBodyLimit::max(100 * 1024 * 1024)), // 100 MB limit for audio uploads;
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}