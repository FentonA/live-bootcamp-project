use crate::domain::data_store::{LoginAttemptId, TwoFACode, TwoFaCodeStore};
use crate::domain::Email;
use crate::routes::LoginResponse;
use crate::utils::auth::generate_auth_cookie;
use crate::{AppState, AuthAPIError};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[tracing::instrument(name = "Verify 2FA", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };
    let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id) {
        Ok(id) => id,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let two_fa_code = match TwoFACode::parse(request.two_fa_code) {
        Ok(code) => code,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(tuple) => tuple,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    if code_tuple.0 != login_attempt_id || code_tuple.1 != two_fa_code {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }

    two_fa_code_store.remove_code(&email);

    let cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
    };
    let updated_jar = jar.add(cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth)).into_response()),
    )
}

#[derive(Deserialize, Default, Debug)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}
