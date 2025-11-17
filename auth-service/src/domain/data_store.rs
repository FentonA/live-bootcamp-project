// domain/data_store.rs
use super::{Email, Password};
use crate::domain::user::User;
use color_eyre::eyre::{eyre, Context, Report, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::any::Any;
use thiserror::Error;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid user credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
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
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait TwoFaCodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFaCodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFaCodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFaCodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFaCodeStoreError {
    #[error("Error: login attempt id not not found")]
    LoginAttempIdNotFound,
    #[error("Unexpectd Error")]
    UnexpectedError(#[error] Report),
    #[error("User Already had a code")]
    UserHasCode,
    #[error("User code not found")]
    CodeNotFound,
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct LoginAttemptId(String);

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self> {
        let parsed_id = uuid::Uuid::parse_str(&id).wrap_err("Invalid login attempt id")?;
        Ok(Self(parsed_id.to_string()))
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
    pub fn parse(code: String) -> Result<Self> {
        let code_as_u32 = code.parse::<u32>().wrap_err("Invalid 2FA code")?;

        if (100_000..=999_999).contains(&code_as_u32) {
            Ok(Self(code))
        } else {
            Err(eyre!("Invalid 2FA code"))
        }
    }
}
impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code: String = (0..6)
            .map(|_| rng.random_range(b'A'..=b'Z') as char)
            .collect();
        Self(code)
    }
}
