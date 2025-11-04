// domain/data_store.rs
use super::{Email, Password};
use crate::domain::user::User;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::any::Any;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user<'a>(&'a self, email: &Email) -> Result<&'a User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn check_token<'a>(&'a self, token: String) -> Result<(), BannedTokenStoreError>;
}

#[derive(Debug)]
pub enum BannedTokenStoreError {
    TokenNotPresent,
    TokenAlreadyBanned,
}

#[async_trait::async_trait]
pub trait TwoFaCodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFaCodeStoreError>;

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFaCodeStoreError>;
}

#[derive(Debug)]
pub enum TwoFaCodeStoreError {
    LoginAttempIdNotFound,
    UnexpectedError,
    UserHasCode,
    CodeNotFound,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct LoginAttemptId(String);

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        match Uuid::parse_str(&id) {
            Ok(_) => Ok(Self(id)),
            Err(e) => {
                println!(
                    "this is the error with the login attemp {:?}",
                    e.to_string()
                );
                Err("Could not parse uudi".to_string())
            }
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = Uuid::new_v4().to_string();
        Self(uuid)
    }
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct TwoFACode(String);

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        if code.len() < 6 {
            return Err("Not a valid Two FA Code".to_string());
        }
        Ok(Self(code))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code: String = (0..6).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
        Self(code)
    }
}
