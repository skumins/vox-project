use std::env;

#[derive(Debug, Clone)]

pub struct Config {
    pub database_url: String,
    pub deepgram_key: String,
    pub openrouter_key: String,
    pub model: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not found in .env".to_string)?;
        let deepgram_key = env::var("DEEPGRAM_API_KEY").map_err(|_| "DEEPGRAM_API_KEY not found in .env".to_string)?;
        let openrouter_key = env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY not found in .env")?;
        let model = env::var("OPPENROUTER_MODEL").map_err(|_| "deepseek/deepseek-chat".to_string());

        Ok(Self {
            database_url,
            deepgram_key,
            openrouter_key,
            model,
        })
    }
}