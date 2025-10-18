pub mod data_store;
pub mod error;
pub mod user;

#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, String> {
        if email.contains('@') && email.contains('.') {
            return Ok(Email(email));
        }
        Err("Could not parse email".to_string())
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
    pub fn parse(password: String) -> Result<Self, String> {
        if password.len() <= 8 {
            return Err("Could not parse password".to_string());
        };
        Ok(Password(password))
    }
}
