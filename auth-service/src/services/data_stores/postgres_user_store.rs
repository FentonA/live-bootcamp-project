use crate::domain::user::User;
use crate::domain::{
    data_store::{UserStore, UserStoreError},
    Email, Password,
};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use color_eyre::eyre::{eyre, Context, Result};
use rand_core::OsRng;
use secrecy::{ExposeSecret, Secret};

use sqlx::PgPool;
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let existing_user = sqlx::query!(
            "SELECT email FROM users WHERE email = $1",
            user.email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        if existing_user.is_some() {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let password_hash = compute_password_hash(user.password.as_ref().to_owned())
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref().expose_secret(),
            &password_hash.expose_secret(),
            user.require_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let row = sqlx::query!(
            r#"
                SELECT email, password_hash, requires_2fa
                FROM users
                WHERE email = $1
            "#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .ok_or(UserStoreError::UserNotFound)?;

        let user = User {
            email: Email::parse(Secret::new(row.email)).map_err(UserStoreError::UnexpectedError)?,
            password: Password::parse(Secret::new(row.password_hash))
                .map_err(UserStoreError::UnexpectedError)?,
            require_2fa: row.requires_2fa,
        };

        Ok(user)
    }

    #[tracing::instrument(name = "Validating user from PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = sqlx::query!(
            r#"
                SELECT password_hash
                FROM users
                WHERE email = $1
            "#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .ok_or(UserStoreError::InvalidCredentials)?;

        verify_password_hash(
            Secret::new(user.password_hash),
            password.as_ref().to_owned(),
        )
        .await
        .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(expected_password_hash.expose_secret())?;

            Argon2::default()
                .verify_password(
                    password_candidate.expose_secret().as_bytes(),
                    &expected_password_hash,
                )
                .wrap_err("failed to verify password")
        })
    })
    .await;

    result?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)?
            .to_string();
            Ok(Secret::new(password_hash))
        })
    })
    .await;

    result?
}
