use secrecy::{ExposeSecret, Secret};
use std::{collections::HashSet, thread::yield_now};

use crate::domain::data_store::{BannedTokenStore, BannedTokenStoreError};

pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

impl HashsetBannedTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: HashSet::new(),
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn store_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(token.expose_secret()) {
            return Err(BannedTokenStoreError::TokenAlreadyBanned);
        }
        self.tokens.insert(token.expose_secret().to_owned());
        Ok(())
    }

    async fn check_token(&self, token: &Secret<String>) -> Result<(), BannedTokenStoreError> {
        if !self.tokens.contains(token.expose_secret()) {
            return Err(BannedTokenStoreError::TokenNotPresent);
        }
        Ok(())
    }
}
