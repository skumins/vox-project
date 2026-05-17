use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    http::StatusCode,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use oauth2::{
    AuthorizationCode, PkceCodeVerifier, TokenResponse,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};
use time::Duration;
use rand::Rng;

use crate::{
    AppState,
    auth::google::{build_auth_url, fetch_user_info},
    auth::middleware::AuthUser,
    auth::session::delete_session,
};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}


pub async fn google_login(State(state): State<AppState>, jar: PrivateCookieJar) -> impl IntoResponse {
    let auth_data = build_auth_url(&state.oauth_client);
    
    let state_cookie = Cookie::build(("oauth_state", auth_data.state))
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(5))
        .path("/")
        .build();

    let pkce_cookie = Cookie::build(("pkce_verifier", auth_data.pkce_verifier))
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(5))
        .path("/")
        .build();

    (
        jar.add(state_cookie).add(pkce_cookie),
        Redirect::to(&auth_data.url),
    )
}

pub async fn google_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    jar: PrivateCookieJar,
) -> impl IntoResponse {

    let stored_state = match jar.get("oauth_state") {
        Some(cookie) => cookie.value().to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing state cookie").into_response(),
    };

    if stored_state != params.state {
        return (StatusCode::BAD_REQUEST, "State mismatch, possible CSRF").into_response();
    }

    let pkce_secret = match jar.get("pkce_verifier") {
        Some(cookie) => cookie.value().to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing verifier cookie").into_response(),
    };

    let pkce_verifier = PkceCodeVerifier::new(pkce_secret);

    let token = state.oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;

    let token = match token {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token exchange failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Token exchange failed").into_response();
        }
    };

    let access_token = token.access_token().secret();

    let user_info = match fetch_user_info(access_token).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to fetch user info: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user info").into_response();
        }
    };

    let user_id: uuid::Uuid = match sqlx::query_scalar!(
        r#"
        INSERT INTO users (google_id, email, name, avatar_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (google_id) DO UPDATE
            SET name = EXCLUDED.name,
                avatar_url = EXCLUDED.avatar_url
        RETURNING id as "id: uuid::Uuid"
        "#,
        user_info.sub, user_info.email, user_info.name, user_info.picture,
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Database upsert failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Datavase error").into_response();
        } 
    };

    let session_token = hex::encode(rand::thread_rng().gen::<[u8; 32]>());

    if let Err(e) = sqlx::query!(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
        session_token,
        user_id,
    )
    .execute(&state.db)
    .await
    {
        tracing::error!("Failed to save session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }


    let session_cookie = Cookie::build(("session", session_token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7))
        .path("/")
        .build();


    let remove_state = Cookie::build(("oauth_state", ""))
        .max_age(Duration::ZERO)
        .path("/")
        .build();

    let remove_pkce = Cookie::build(("pkce_verifier", ""))
        .max_age(Duration::ZERO)
        .path("/")
        .build();

    tracing::info!("User logged in: {} (id={})", user_info.email, user_id);

    // return jar from session cookie
    (
        jar.add(session_cookie)
            .remove(remove_state)
            .remove(remove_pkce),
        Redirect::to("/"),
    ).into_response()
}


pub async fn me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let user = sqlx::query_as!(
        MeResponse,
        r#"SELECT id::text as "id!", email, name, avatar_url FROM users WHERE id = $1"#,
        auth_user.user_id
    )
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(u)) => (StatusCode::OK, Json(u)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn logout(State(state): State<AppState>, jar: PrivateCookieJar) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value().to_string();
        let _ = delete_session(&token, &state.db).await;
    }

    let remove_session = Cookie::build(("session", ""))
        .max_age(Duration::ZERO)
        .path("/")
        .build();

    (
        jar.remove(remove_session),
        Redirect::to("/",)
    )
}
