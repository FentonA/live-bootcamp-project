use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

// TODO: Implement tests for all other routes (signup, login, logout, verify-2fa, and verify-token)
// For now, simply assert that each route returns a 200 HTTP status code.
#[tokio::test]
async fn signup_returns_ok() {
    let app = TestApp::new().await;

    let response = app.signup().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_returns_ok() {
    let app = TestApp::new().await;

    let response = app.login().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_returns_ok() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_2fa_ok() {
    let app = TestApp::new().await;

    let response = app.verify_2fa().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_ok() {
    let app = TestApp::new().await;

    let response = app.verify_token().await;

    assert_eq!(response.status().as_u16(), 200);
}
