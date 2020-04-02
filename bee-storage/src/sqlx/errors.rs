use sqlx::Error as SqlxError;

use std::{
    error::Error as StdError,
    fmt,
};

#[derive(Clone, Debug)]
pub enum SqlxBackendError {
    ConnectionBackendError(String),
    EnvError(std::env::VarError),
    SqlxError(String),
    Bincode(String),
    UnknownError,
    //...
}

impl fmt::Display for SqlxBackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlxBackendError::ConnectionBackendError(ref reason) => write!(f, "Connection error: {:?}", reason),
            SqlxBackendError::EnvError(ref reason) => write!(f, "Connection error: {:?}", reason),
            SqlxBackendError::SqlxError(ref reason) => write!(f, "Sqlx core error: {:?}", reason),
            SqlxBackendError::Bincode(ref reason) => write!(f, "Bincode error: {:?}", reason),
            SqlxBackendError::UnknownError => write!(f, "Unknown error"),
        }
    }
}

// Allow this type to be treated like an error
impl StdError for SqlxBackendError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            _ => None,
        }
    }
}

impl From<std::env::VarError> for SqlxBackendError {
    #[inline]
    fn from(err: std::env::VarError) -> Self {
        SqlxBackendError::EnvError(err)
    }
}

impl From<SqlxError> for SqlxBackendError {
    #[inline]
    fn from(err: SqlxError) -> Self {
        SqlxBackendError::SqlxError(String::from(err.to_string()))
    }
}

impl From<std::boxed::Box<bincode::ErrorKind>> for SqlxBackendError {
    #[inline]
    fn from(err: std::boxed::Box<bincode::ErrorKind>) -> Self {
        SqlxBackendError::Bincode(String::from(err.as_ref().to_string()))
    }
}
