use auth_service::{
    app_state::app_state::AppState,
    domain::{email_client, EmailClient},
    hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    hashmap_user_store::HashmapUserStore,
    hashset_banned_token_store::HashsetBannedTokenStore,
    mock_email_client::MockEmailClient,
    utils::constants::prod,
    Application,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let userstore = HashmapUserStore::new();
    let tokenstore = HashsetBannedTokenStore::new();
    let two_fa_code_store = HashMapTwoFACodeStore::default();
    let email_client = Arc::new(MockEmailClient);
    let app_state = AppState::new(
        Arc::new(RwLock::new(userstore)),
        Arc::new(RwLock::new(tokenstore)),
        Arc::new(RwLock::new(two_fa_code_store)),
        email_client,
    );
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Could not build the app");

    app.run().await.expect("Could not run app");
}
