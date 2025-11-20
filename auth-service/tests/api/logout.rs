use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::data_store::BannedTokenStore;
use auth_service::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use reqwest::cookie::CookieStore;
use reqwest::Url;
use secrecy::Secret;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
    app.clean_up().await
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(
        signup_response.status().as_u16(),
        201,
        "Signup should succeed"
    );
    // Login
    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(
        login_response.status().as_u16(),
        200,
        "Login should succeed"
    );

    // Extract cookie from response headers
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == "jwt")
        .expect("No JWT cookie found in login response");

    let token = Secret::new(auth_cookie.value().to_string());

    // Logout
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    // Verify token is banned
    {
        let token_store = app.banned_token_store.read().await;
        let is_banned = token_store.check_token(&token).await;
        assert!(is_banned.is_ok(), "Token should be banned after logout");
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;
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
    app.clean_up().await;
}
