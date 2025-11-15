use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
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
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        // 3. Use serde_json::to_string to serialize the TwoFATuple instance into a JSON string.
        let json_string =
            serde_json::to_string(&tuple).map_err(|_| TwoFaCodeStoreError::UnexpectedError)?;
        // Return TwoFACodeStoreError::UnexpectedError if serialization fails.
        // 4. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        let mut conn = self.conn.write().await;

        conn.set_ex::<_, _, ()>(new_key, json_string, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFaCodeStoreError::UnexpectedError)?;

        // The value should be the serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUTES_IN_SECONDS.
        // Return TwoFACodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFaCodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        // 2. Call the del command on the Redis connection to delete the 2FA code entry.
        // Return TwoFACodeStoreError::UnexpectedError if the operation fails.
        let key = get_key(email);
        let mut conn = self.conn.write().await;
        conn.del::<_, ()>(key)
            .map_err(|_| TwoFaCodeStoreError::UnexpectedError);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFaCodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        let key = get_key(email);
        // 2. Call the get command on the Redis connection to get the value stored for the key.
        let mut conn = self.conn.write().await;
        // Return TwoFACodeStoreError::LoginAttemptIdNotFound if the operation fails.
        let json_string: String = conn
            .get(key)
            .map_err(|_| TwoFaCodeStoreError::LoginAttempIdNotFound)?;

        let tuple: TwoFATuple =
            serde_json::from_str(&json_string).map_err(|_| TwoFaCodeStoreError::UnexpectedError)?;

        let login_attempt_id =
            LoginAttemptId::parse(tuple.0).map_err(|_| TwoFaCodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse(tuple.1).map_err(|_| TwoFaCodeStoreError::UnexpectedError)?;
        // If the operation succeeds, call serde_json::from_str to parse the JSON string into a TwoFATuple.
        // Then, parse the login attempt ID string and 2FA code string into a LoginAttemptId and TwoFACode type respectively.
        // Return TwoFACodeStoreError::UnexpectedError if parsing fails.
        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
