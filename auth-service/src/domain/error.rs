#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
    IncorrectCredentials,
    InvalidToken,
    MissingToken,
}
