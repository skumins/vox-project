use oauth2::{
    basic::BasicClient,
    PkceCodeChallenge,
    CsrfToken, Scope, 
};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

pub struct AuthUrlData {
    pub url: String,
    pub state: String,
    pub pkce_verifier: String,
}

pub fn build_auth_url(client: &BasicClient) -> AuthUrlData {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    AuthUrlData { 
        url: auth_url.to_string(), 
        state: csrf_token.secret().to_string(), 
        pkce_verifier: pkce_verifier.secret().to_string(), 
    }
}

pub async fn fetch_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = Client::new();
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "unreadable body".to_string());
        return Err(format!("Google userinfo error {}: {}", status, body));
    }

    response
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse userinfo JSON: {}", e))
}