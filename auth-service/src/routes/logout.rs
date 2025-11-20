use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use secrecy::Secret;

use crate::{
    app_state::app_state::AppState,
    domain::{data_store::BannedTokenStore, error::AuthAPIError},
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(name = "Logout", skip_all)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = Secret::new(cookie.value().to_string());

    if let Err(_) = validate_token(token.clone(), state.tokenstore.clone()).await {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    let mut token_store = state.tokenstore.write().await;
    if let Err(e) = token_store.store_token(token).await {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    let removal_cookie = Cookie::build((JWT_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .build();

    let updated_jar = jar.remove(removal_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}
