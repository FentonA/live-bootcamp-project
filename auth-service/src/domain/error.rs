use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Unexpected Error")]
    UnexpectedError(#[source] Report),
    #[error("Incorrect Credentials:")]
    IncorrectCredentials,
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Mising Token")]
    MissingToken,
}
