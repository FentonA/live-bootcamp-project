pub mod data_store;
pub mod email_client;
pub mod error;
pub mod user;
use color_eyre::eyre::{eyre, Result};
pub use email_client::*;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::hash::Hash;

#[derive(Debug, Deserialize, Clone)]
pub struct Email(Secret<String>);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Email {
    pub fn parse(email: Secret<String>) -> Result<Self> {
        if validate_email(&email.expose_secret()) {
            return Ok(Self(email));
        } else {
            Err(eyre!(format!(
                "{} is not a valid email.",
                email.expose_secret()
            )))
        }
    }
}

fn validate_email(email: &String) -> bool {
    email.contains('@') && email.contains('.')
}

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(password: Secret<String>) -> Result<Self> {
        if validate_password(&password) {
            Ok(Self(password))
        } else {
            Err(eyre!("Failed to parase string to a Password"))
        }
    }
}

fn validate_password(s: &Secret<String>) -> bool {
    s.expose_secret().len() >= 8
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}
impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Email;
    use super::Password;
    use fake::faker::internet::en::Password as FakePassword;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use secrecy::Secret;

    #[test]
    fn empty_string_is_rejected() {
        let password = Secret::new("".to_string());
        assert!(Password::parse(password).is_err());
    }

    #[test]
    fn string_less_than_8_characters_is_rejected() {
        let password = Secret::new("1234567".to_string());
        assert!(Password::parse(password).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: quickcheck::Gen>(_g: &mut G) -> Self {
            let password = FakePassword(8..30).fake();
            Self(Secret::new(password))
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }

    #[test]
    fn empty_email_string_is_rejected() {
        let email = Secret::new("".to_string());
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = Secret::new("ursuladomain.com".to_string());
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = Secret::new("domain.com".to_string()); // Updated!
        assert!(Email::parse(email).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(_g: &mut G) -> Self {
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(Secret::new(valid_email.0)).is_ok() // Updated!
    }
}
