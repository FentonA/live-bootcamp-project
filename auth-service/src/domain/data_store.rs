// domain/data_store.rs
use super::{Email, Password};
use crate::domain::user::User;
use color_eyre::eyre::{eyre, Context, Report, Result};
use rand::Rng;
use secrecy::{ExposeSecret, Secret};
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
    async fn store_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError>;
    async fn check_token(&self, token: &Secret<String>) -> Result<(), BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Token not present")]
    TokenNotPresent,
    #[error("Token already banned")]
    TokenAlreadyBanned,
    #[error("Unexepected Error")]
    UnexpectedError(#[source] Report),
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
    UnexpectedError(#[source] Report),
    #[error("User Already had a code")]
    UserHasCode,
    #[error("User code not found")]
    CodeNotFound,
}

impl PartialEq for TwoFaCodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttempIdNotFound, Self::LoginAttempIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginAttemptId(Secret<String>);

impl LoginAttemptId {
    pub fn parse(id: Secret<String>) -> Result<Self> {
        let parsed_id =
            uuid::Uuid::parse_str(&id.expose_secret()).wrap_err("Invalid login attempt id")?;
        Ok(Self(Secret::new(parsed_id.to_string())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = Secret::new(Uuid::new_v4().to_string());
        Self(uuid)
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwoFACode(Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self> {
        let code_as_u32 = code
            .expose_secret()
            .parse::<u32>()
            .wrap_err("Invalid 2FA code")?;

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
        let code = rng.random_range(100_000..=999_999); // Generate 6-digit number
        Self(Secret::new(code.to_string()))
    }
}
