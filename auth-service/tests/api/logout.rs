use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::data_store::BannedTokenStore;
use auth_service::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use reqwest::cookie::CookieStore;
use reqwest::Url;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400)
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401)
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    app.post_login(&login_body).await;

    let cookie_str = app
        .cookie_jar
        .cookies(&app.address.parse().unwrap())
        .expect("No cookies found");

    let token = cookie_str
        .to_str()
        .expect("Invalid cookie")
        .split(';')
        .next()
        .expect("No cookie value")
        .split('=')
        .nth(1)
        .expect("No token value")
        .to_string();

    let response = app.post_logout().await;

    let token_store = app.banned_token_store.read().await;
    let is_banned = token_store.check_token(token).await;

    assert!(is_banned.is_ok(), "Token should be banned after logout");
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    app.post_login(&login_body).await;
    app.post_logout().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}
