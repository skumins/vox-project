use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
};

use axum_extra::extract::cookie::PrivateCookieJar;

use crate::{AppState, auth::session::validate_session};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
}

pub async fn auth_guard(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    mut request: Request,
    next: Next,
) -> Response {

    let session_token = match jar.get("session") {
        Some(cookie) => cookie.value().to_string(),
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id = match validate_session(&session_token, &state.db).await {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    request.extensions_mut().insert(AuthUser { user_id });
    next.run(request).await
}