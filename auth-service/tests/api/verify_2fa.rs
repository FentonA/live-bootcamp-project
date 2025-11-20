use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::data_store::TwoFaCodeStore;
use auth_service::domain::Email;
use auth_service::routes::login::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use secrecy::{ExposeSecret, Secret};

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    let code_store = app.two_fa_code_store.read().await;
    let email_obj = Email::parse(Secret::new(random_email.clone())).unwrap();
    let (_, code) = code_store
        .get_code(&email_obj)
        .await
        .expect("Failed to get code");

    let verify_request = serde_json::json!({
        "email": email_obj.as_ref().expose_secret(),
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": code.as_ref().expose_secret()
    });

    drop(code_store);

    let response = app.post_verify_2fa(&verify_request).await;
    println!("this is the response {:?}", response);
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let response = app
        .http_client
        .post(format!("{}/verify-2fa", &app.address))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 422);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let invalid_email_request = serde_json::json!({
        "email": "invalid-email",
        "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
        "2FACode": "123456"
    });

    let response = app.post_verify_2fa(&invalid_email_request).await;
    assert_eq!(response.status().as_u16(), 400);

    let invalid_id_request = serde_json::json!({
        "email": "test@example.com",
        "loginAttemptId": "invalid-uuid",
        "2FACode": "123456"
    });

    let response = app.post_verify_2fa(&invalid_id_request).await;
    assert_eq!(response.status().as_u16(), 400);

    let invalid_code_request = serde_json::json!({
        "email": "test@example.com",
        "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
        "2FACode": "123"
    });

    let response = app.post_verify_2fa(&invalid_code_request).await;
    assert_eq!(response.status().as_u16(), 400);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    let code_store = app.two_fa_code_store.read().await;
    let email_obj = Email::parse(Secret::new(random_email.clone())).unwrap();
    let (_, code) = code_store
        .get_code(&email_obj)
        .await
        .expect("Failed to get code");

    let code_str = code.as_ref().to_owned();
    drop(code_store);

    let verify_request = serde_json::json!({
        "email": email_obj.as_ref().expose_secret(),
        "loginAttemptId": json_body.login_attempt_id.clone(),
        "2FACode": code_str.expose_secret()
    });

    let response = app.post_verify_2fa(&verify_request).await;
    assert_eq!(response.status().as_u16(), 200);

    let verify_request = serde_json::json!({
        "email": email_obj.as_ref().expose_secret(),
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": code.as_ref().expose_secret()
    });

    let response = app.post_verify_2fa(&verify_request).await;
    assert_eq!(response.status().as_u16(), 401);
    app.clean_up().await;
}
