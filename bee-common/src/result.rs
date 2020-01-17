#[derive(Debug)]
pub enum Error {
    ConfigError,
    NetworkError,
    TransactionError,
}

pub type Result<T> = std::result::Result<T, Error>;
