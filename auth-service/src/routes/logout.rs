use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    domain::error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie - return error if missing
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value();

    if let Err(_) = validate_token(token).await {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    let removal_cookie = Cookie::build((JWT_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .build();

    let updated_jar = jar.clone().remove(removal_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}
