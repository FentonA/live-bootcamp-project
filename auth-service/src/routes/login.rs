use crate::app_state::app_state::AppState;
use crate::domain::data_store::*;
use crate::domain::error::AuthAPIError;
use crate::domain::user::User;
use crate::domain::{Email, Password};
use crate::utils::auth::generate_auth_cookie;
use axum::response::Response;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let password = match Password::parse(request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = &state.userstore.read().await;

    if let Err(_) = user_store.validate_user(&email, &password).await {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let updated_jar = jar.add(auth_cookie);

    match user.require_2fa {
        true => handle_2fa(&state, &user.email, updated_jar).await,
        false => handle_no_2fa(&email, updated_jar).await,
    }
}

async fn handle_2fa(
    state: &AppState,
    email: &Email,
    jar: CookieJar,
) -> (CookieJar, Result<Response, AuthAPIError>) {
    let login_attempt = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let response = TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: login_attempt.clone(),
    };

    let mut code_store = state.two_fa_code_store.write().await;
    if let Err(_) = code_store.add_code(email, login_attempt, two_fa_code).await {
        let _ = state.email_client.send_email(
            &email,
            "Could not create 2FA code",
            "There was an error creating the 2fa code for your account",
        );
        return (jar, Err(AuthAPIError::UnexpectedError));
    }
    (jar, Ok((StatusCode::OK, Json(response)).into_response()))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (CookieJar, Result<Response, AuthAPIError>) {
    let response = RegularAuth {
        message: "You have successfully logged in!".to_string(),
    };
    (jar, Ok((StatusCode::OK, Json(response)).into_response()))
}

#[derive(Deserialize, Default)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Deserialize, Default, Serialize)]
pub struct RegularAuth {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: LoginAttemptId,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}
