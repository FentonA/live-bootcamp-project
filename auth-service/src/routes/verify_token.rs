use crate::app_state::app_state::AppState;
use crate::domain::data_store::*;
use crate::domain::error::AuthAPIError;
use crate::domain::user::User;
use crate::domain::{Email, Password};
use crate::utils::auth;
use crate::utils::auth::generate_auth_cookie;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<TokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if request.token.is_empty() {
        return Err(AuthAPIError::InvalidToken);
    }
    let token = Secret::new(request.token);
    if let Err(_) = auth::validate_token(token, state.tokenstore).await {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize, Debug)]
pub struct TokenRequest {
    pub token: String,
}
