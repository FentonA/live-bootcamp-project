use crate::helpers::{get_random_email, TestApp};
use auth_service::ErrorResponse;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let body = [serde_json::json!({
        "email": get_random_email(),
        "password": "passw",
        "requires2FA": false
    })];
    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 422)
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;

    let login = serde_json::json!({
        "email": "existing_user@mail.com",
        "password": "correct_password",
        "requires2FA": true
    });

    let response = app.post_signup(&login).await;

    assert_eq!(response.status().as_u16(), 201);
    let user = serde_json::json!({
        "email": "existing_user@mail.com",
        "password": "dfjas:dlfkjasd:fljkad",
        "requires2FA": true
    });
    let response = app.post_login(&user).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "nouser@mail.com",
        "password": "pass",
        "requires2FA": true
    });

    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 400);
}
