use crate::domain::{Email, Password};
use color_eyre::eyre::Report;
use rand::Rng;
use thiserror::Error;

#[derive(Debug, Error, Default, Clone, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub require_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: Password, require_2fa: bool) -> Self {
        Self {
            email,
            password,
            require_2fa,
        }
    }
}
