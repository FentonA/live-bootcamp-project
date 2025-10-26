// domain/data_store.rs
use super::{Email, Password};
use crate::domain::user::User;

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
