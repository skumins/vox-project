use std::net::SocketAddr;
use std::sync::Arc;
use axum::{routing::post, Router};
use tokio::sync::Mutex;
use sqlx::sqlite::SqlitePool;

mod services;
mod handlers;
mod models;
mod prompts;
mod stt;
mod audio_capture;
mod config;

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
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env().expect("Error configuring");

    let db_pool = SqlitePool::connect(&config.database_url).await.unwrap();
    sqlx::migrate!().run(&db_pool).await.unwrap();

    let deepgram = DeepgramService::new(config.deepgram_key);
    let llm = OpenRouterService::new(config.openrouter_key, config.model);

    let state = AppState{
        db: db_pool,
        deepgram,
        llm,
    };
    
    println!("VOXA backend running...");

    // STARTING THE WEB SERVER
    let app = Router::new()
        .route("/transcribe", post(handlers::transcribe::transcribe_audio))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
