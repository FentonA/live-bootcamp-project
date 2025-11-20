use crate::{
    domain::data_store::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Store Token", skip_all)]
    async fn store_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token.expose_secret());

        // Lock the connection to get mutable access
        let mut conn = self.conn.write().await;

        // Use the connection with Redis commands
        conn.set_ex::<_, _, ()>(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|_| BannedTokenStoreError::TokenAlreadyBanned)?;

        Ok(())
    }

    #[tracing::instrument(name = "Check Token", skip_all)]
    async fn check_token(&self, token: &Secret<String>) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token.expose_secret());

        // Lock the connection (read access is enough for checking)
        let mut conn = self.conn.write().await;

        // Check if key exists
        let exists: bool = conn
            .exists(&key)
            .map_err(|_| BannedTokenStoreError::TokenNotPresent)?;

        if exists {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenNotPresent)
        }
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
