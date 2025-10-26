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
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(&token) {
            return Err(BannedTokenStoreError::TokenAlreadyBanned);
        }
        self.tokens.insert(token);
        Ok(())
    }

    async fn check_token(&self, token: String) -> Result<(), BannedTokenStoreError> {
        if !self.tokens.contains(&token) {
            return Err(BannedTokenStoreError::TokenNotPresent);
        }
        Ok(())
    }
}
