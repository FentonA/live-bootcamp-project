use auth_service::{
    app_state::app_state::AppState, hashmap_user_store::HashmapUserStore,
    hashset_banned_token_store::HashsetBannedTokenStore, utils::constants::prod, Application,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let userstore = HashmapUserStore::new();
    let tokenstore = HashsetBannedTokenStore::new();
    let app_state = AppState::new(
        Arc::new(RwLock::new(userstore)),
        Arc::new(RwLock::new(tokenstore)),
    );
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Could not build the app");

    app.run().await.expect("Could not run app");
}
