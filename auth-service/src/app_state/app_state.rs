use crate::services::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
use crate::{
    data_stores::redis_two_fa_code_store::RedisTwoFACodeStore,
    domain::{
        data_store::{BannedTokenStore, UserStore},
        email_client, EmailClient,
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type TokenStore = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type EmailClientType = Arc<dyn EmailClient + Send + Sync>;
pub type CodeStore = Arc<RwLock<RedisTwoFACodeStore>>;

#[derive(Clone)]
pub struct AppState {
    pub userstore: UserStoreType,
    pub tokenstore: TokenStore,
    pub two_fa_code_store: CodeStore,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        userstore: UserStoreType,
        tokenstore: TokenStore,
        two_fa_code_store: CodeStore,
        email_client: EmailClientType,
    ) -> Self {
        Self {
            userstore,
            tokenstore,
            two_fa_code_store,
            email_client,
        }
    }
}
