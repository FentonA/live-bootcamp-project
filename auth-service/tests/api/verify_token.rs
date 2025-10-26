use crate::helpers::{get_random_email, TestApp};
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use reqwest::cookie::CookieStore;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let body = serde_json::json!({"tokn": "".to_string()});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 422)
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Sign up
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    app.post_signup(&signup_body).await;

    // Login to get a valid JWT token
    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 200);

    // Extract the JWT token from the cookie jar (not response headers)
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

    let verify_body = serde_json::json!({ "token": token });
    let response = app.post_verify_token(&verify_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
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
    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 200);

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

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status().as_u16(), 200);

    let verify_body = serde_json::json!({ "token": token });
    let response = app.post_verify_token(&verify_body).await;

    assert_eq!(response.status().as_u16(), 401);
}
