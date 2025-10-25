use crate::app_state::app_state::AppState;
use crate::domain::data_store::*;
use crate::domain::error::AuthAPIError;
use crate::domain::user::User;
use crate::domain::{Email, Password};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.userstore.write().await;

    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|_| (AuthAPIError::IncorrectCredentials))?;

    let response = Json(LoginMessage {
        message: "Login successful".to_string(),
    });

    Ok((StatusCode::OK, response))
}

#[derive(Deserialize, Default)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Deserialize, Default, Serialize)]
pub struct LoginMessage {
    pub message: String,
}
