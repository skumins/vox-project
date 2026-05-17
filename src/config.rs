use std::env;
#[derive(Debug, Clone)]

pub struct Config {
    pub database_url: String,
    pub deepgram_key: String,
    pub openrouter_key: String,
    pub model: String,
    pub encryption_key: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_url: String,
    pub cookie_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not found in .env")?;
        let deepgram_key = env::var("DEEPGRAM_API_KEY").map_err(|_| "DEEPGRAM_API_KEY not found in .env")?;
        let openrouter_key = env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY not found in .env")?;
        let model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "openrouter/free".to_string());
        let encryption_key = env::var("ENCRYPTION_KEY").map_err(|_| "ENCRYPTION_KEY not found in .env")?;
        let google_client_id = env::var("GOOGLE_CLIENT_ID").map_err(|_| "GOOGLE_CLIENT_ID not found in .env")?;
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").map_err(|_| "GOOGLE_CLIENT_SECRET not found in .env")?;
        let google_redirect_url = env::var("GOOGLE_REDIRECT_URL").map_err(|_| "GOOGLE_REDIRECT_URL not found in .env")?;
        let cookie_secret = env::var("COOKIE_SECRET").map_err(|_| "COOKIE_SECRET not found in .env")?;

        Ok(Self {
            database_url,
            deepgram_key,
            openrouter_key,
            model,
            encryption_key,
            google_client_id,
            google_client_secret,
            google_redirect_url,
            cookie_secret,
        })
    }
}