use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
    extract::{DefaultBodyLimit, FromRef},
    middleware,
};
use sqlx::postgres::PgPool;

use oauth2::basic::BasicClient;
use axum_extra::extract::cookie::Key;


mod services;
mod handlers;
mod models;
mod prompts;
mod config;
mod api;
mod auth;


use services::{deepgram::DeepgramService, llm::OpenRouterService};
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub deepgram: DeepgramService,
    pub llm: OpenRouterService,
    pub encryption_key: String,
    pub oauth_client: std::sync::Arc<BasicClient>,
    pub cookie_key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
    
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

    let oauth_client = build_oauth_client(&config);

    let cookie_key = Key::from(
        hex::decode(&config.cookie_secret)
        .expect("COOKIE_SECRET must be valid hex")
        .as_slice()
    );

    let state = AppState{
        db: db_pool,
        deepgram: DeepgramService::new(config.deepgram_key.clone()),
        llm: OpenRouterService::new(config.openrouter_key.clone(), config.model.clone()),
        encryption_key: config.encryption_key.clone(),
        oauth_client: std::sync::Arc::new(oauth_client),
        cookie_key,
    };
    
    tracing::info!("VOXA backend running...");

    let protected = Router::new()
        .route("/api/me", get(handlers::auth::me))
        //  /api/notes, /api/keys, ...
        .layer(middleware::from_fn_with_state(
            state.clone(), 
            auth::middleware::auth_guard,
        ));

    let app = Router::new()
        .route("/ws", get(api::ws::ws_handler))
        .route("/transcribe", post(handlers::transcribe::transcribe_audio)
            .layer(DefaultBodyLimit::max(100 * 1024 * 1024)), // 100 MB limit for audio uploads;
        )
        .route("/auth/google", get(handlers::auth::google_login))
        .route("/auth/google/callback", get(handlers::auth::google_callback))
        .route("/auth/logout", post(handlers::auth::logout))
        .merge(protected)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .expect("Failed to bind address");
    axum::serve(listener, app).await
        .expect("Server crashed");
}

fn build_oauth_client(config: &Config) -> BasicClient {

    use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid Google auth URL");

    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid Google token URL");

    BasicClient::new(
        ClientId::new(config.google_client_id.clone()),
        Some(ClientSecret::new(config.google_client_secret.clone())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(config.google_redirect_url.clone())
            .expect("Invalid redirect URL"),
    )
}