use crate::domain::data_store::{UserStore, UserStoreError};
use crate::domain::user::*;
use crate::domain::{Email, Password};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    async fn get_user<'a>(&'a self, email: &Email) -> Result<&'a User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) if &user.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user(email: &str, password: &str) -> User {
        let email = Email::parse(email.to_string()).expect("Valid email");
        let password = Password::parse(password.to_string()).expect("Valid password");
        User::new(email, password, true)
    }

    #[tokio::test]
    async fn test_add_user() {
        let test_user = create_test_user("test@mail.com", "password123");
        let mut user_store = HashmapUserStore::new();

        let result = user_store.add_user(test_user.clone()).await;
        assert!(result.is_ok());

        let result_duplicate = user_store.add_user(test_user).await;
        assert_eq!(result_duplicate, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = Email::parse("test@mail.com".to_string()).expect("Valid email");
        let test_user = create_test_user("test@mail.com", "password123");
        let test_user_two = create_test_user("test2@mail.com", "password123");
        let mut user_store = HashmapUserStore::new();

        user_store.add_user(test_user.clone()).await.unwrap();

        let result = user_store.get_user(&email).await;
        assert_eq!(result.expect("User should exist"), &test_user);

        let email_two = Email::parse("test2@mail.com".to_string()).expect("Valid email");
        let result_not_found = user_store.get_user(&email_two).await;
        assert_eq!(result_not_found, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let email = Email::parse("test@mail.com".to_string()).expect("Valid email");
        let password = Password::parse("password123".to_string()).expect("Valid password");
        let wrong_password =
            Password::parse("wrongpassword123".to_string()).expect("Valid password");

        let test_user = create_test_user("test@mail.com", "password123");
        let mut user_store = HashmapUserStore::new();

        user_store.add_user(test_user.clone()).await.unwrap();

        let result = user_store.validate_user(&email, &password).await;
        assert!(result.is_ok());

        let result_invalid = user_store.validate_user(&email, &wrong_password).await;
        assert_eq!(result_invalid, Err(UserStoreError::InvalidCredentials));

        let nonexistent_email =
            Email::parse("nonexistent@mail.com".to_string()).expect("Valid email");
        let result_not_found = user_store
            .validate_user(&nonexistent_email, &password)
            .await;
        assert_eq!(result_not_found, Err(UserStoreError::UserNotFound));
    }
}
