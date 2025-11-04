use std::collections::HashMap;

use crate::domain::{
    data_store::{LoginAttemptId, TwoFACode, TwoFaCodeStore, TwoFaCodeStoreError},
    Email,
};

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    pub codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashMapTwoFACodeStore {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl TwoFaCodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFaCodeStoreError> {
        if self.codes.contains_key(&email) {
            return Err(TwoFaCodeStoreError::UserHasCode);
        }
        self.codes.insert(email.clone(), (login_attempt_id, code));
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFaCodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFaCodeStoreError::CodeNotFound)
    }
}
