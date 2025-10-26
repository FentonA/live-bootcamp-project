use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::hashmap_user_store::HashmapUserStore;
use crate::services::hashset_banned_token_store::HashsetBannedTokenStore;

pub type UserStore = Arc<RwLock<HashmapUserStore>>;
pub type TokenStore = Arc<RwLock<HashsetBannedTokenStore>>;

#[derive(Clone)]
pub struct AppState {
    pub userstore: UserStore,
    pub tokenstore: TokenStore,
}

//TODO: question: why doesn't destructuring work here?
impl AppState {
    pub fn new(userstore: UserStore, tokenstore: TokenStore) -> Self {
        Self {
            userstore,
            tokenstore,
        }
    }
}
