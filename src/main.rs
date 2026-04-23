use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
    extract::DefaultBodyLimit,
};
use sqlx::postgres::PgPool;

mod services;
mod handlers;
mod models;
mod prompts;
mod config;
mod api;

use services::{deepgram::DeepgramService, llm::OpenRouterService};
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub deepgram: DeepgramService,
    pub llm: OpenRouterService,
    pub encryption_key: String,
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

    let db_pool = PgPool::connect(&config.database_url).await.expect("Failed to connect to database");
    sqlx::migrate!("./migrations").run(&db_pool).await.expect("Migration failed");

    let state = AppState{
        db: db_pool,
        deepgram: DeepgramService::new(config.deepgram_key.clone()),
        llm: OpenRouterService::new(config.openrouter_key.clone(), config.model.clone()),
        encryption_key: config.encryption_key.clone(),
    };
    
    tracing::info!("VOXA backend running...");

    let app = Router::new()
        .route("/ws", get(api::ws::ws_handler))
        .route("/transcribe", post(handlers::transcribe::transcribe_audio)
            .layer(DefaultBodyLimit::max(100 * 1024 * 1024)), // 100 MB limit for audio uploads;
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .expect("Failed to bind address");
    axum::serve(listener, app).await
        .expect("Server crashed");
}