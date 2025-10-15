use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::hashmap_user_store::HashmapUserStore;

pub type UserStore = Arc<RwLock<HashmapUserStore>>;

#[derive(Clone)]
pub struct AppState {
    pub userstore: UserStore,
}

//TODO: question: why doesn't destructuring work here?
impl AppState {
    pub fn new(userstore: UserStore) -> Self {
        Self { userstore }
    }
}
