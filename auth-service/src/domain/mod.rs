pub mod data_store;
pub mod email_client;
pub mod error;
pub mod user;
use color_eyre::eyre::{eyre, Result};
pub use email_client::*;

#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self> {
        if email.contains('@') && email.contains('.') {
            return Ok(Email(email));
        }
        Err(eyre!("Could not parse email".to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Self> {
        if password.len() <= 8 {
            return Err(eyre!("Could not parse password".to_string()));
        };
        Ok(Password(password))
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
