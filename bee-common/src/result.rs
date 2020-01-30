#[derive(Debug)]
pub enum Error {
    ConfigError {
        key: &'static str,
        msg: &'static str,
    },
    NetworkError,
    TransactionError,
}

pub type Result<T> = std::result::Result<T, Error>;
