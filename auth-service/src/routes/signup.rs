use crate::app_state::app_state::AppState;
use crate::domain::data_store::*;
use crate::domain::error::AuthAPIError;
use crate::domain::user::User;
use crate::domain::{Email, Password};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use secrecy::Secret;
use serde::{Deserialize, Serialize};

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email =
        Email::parse(Secret::new(request.email)).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password = Password::parse(Secret::new(request.password))
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.userstore.write().await;

    match user_store.add_user(user).await {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully".to_string(),
            });
            Ok((StatusCode::CREATED, response))
        }
        Err(UserStoreError::UserAlreadyExists) => Err(AuthAPIError::UserAlreadyExists),
        Err(e) => Err(AuthAPIError::UnexpectedError(e.into())),
    }
}

#[derive(Deserialize, Serialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Deserialize, Default, Serialize)]
pub struct SignupResponse {
    pub message: String,
}
