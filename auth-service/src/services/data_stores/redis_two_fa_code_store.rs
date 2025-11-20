use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    data_store::{LoginAttemptId, TwoFACode, TwoFaCodeStore, TwoFaCodeStoreError},
    Email,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFaCodeStore for RedisTwoFACodeStore {
    #[tracing::instrument(name = "Add two fa code - redis", skip_all)]
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFaCodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        let new_key = get_key(&email);
        // 2. Create a TwoFATuple instance
        let tuple = TwoFATuple(
            login_attempt_id.as_ref().expose_secret().to_owned(),
            code.as_ref().expose_secret().to_owned(),
        );
        // 3. Use serde_json::to_string to serialize the TwoFATuple instance into a JSON string.
        let json_string = serde_json::to_string(&tuple)
            .wrap_err("Failed to serialize 2FA tuple")
            .map_err(TwoFaCodeStoreError::UnexpectedError)?;
        // Return TwoFaCodeStoreError::UnexpectedError if serialization fails.
        // 4. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        let mut conn = self.conn.write().await;

        conn.set_ex::<_, _, ()>(new_key, json_string, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis") // New!
            .map_err(TwoFaCodeStoreError::UnexpectedError)?; // Updated!

        // The value should be the serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUTES_IN_SECONDS.
        // Return TwoFaCodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        Ok(())
    }

    #[tracing::instrument(name = "remove two fa code - redis", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFaCodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        // 2. Call the del command on the Redis connection to delete the 2FA code entry.
        // Return TwoFaCodeStoreError::UnexpectedError if the operation fails.
        let key = get_key(email);
        let mut conn = self.conn.write().await;
        conn.del::<_, ()>(key)
            .wrap_err("failed to delete 2FA code from Redis")
            .map_err(TwoFaCodeStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(name = "fetch two code - redis", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFaCodeStoreError> {
        let key = get_key(email);

        match self.conn.write().await.get::<_, String>(&key) {
            Ok(value) => {
                let data: TwoFATuple = serde_json::from_str(&value)
                    .wrap_err("failed to deserialize 2FA tuple") // New!
                    .map_err(TwoFaCodeStoreError::UnexpectedError)?; // Updated!

                let login_attempt_id = LoginAttemptId::parse(data.0.into())
                    .map_err(TwoFaCodeStoreError::UnexpectedError)?; // Updated!

                let email_code = TwoFACode::parse(data.1.into())
                    .map_err(TwoFaCodeStoreError::UnexpectedError)?; // Updated!

                Ok((login_attempt_id, email_code))
            }
            Err(_) => Err(TwoFaCodeStoreError::LoginAttempIdNotFound),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

#[tracing::instrument(name = "get key redis", skip_all)]
fn get_key(email: &Email) -> String {
    format!(
        "{}{}",
        TWO_FA_CODE_PREFIX,
        email.as_ref().expose_secret().to_owned()
    )
}
