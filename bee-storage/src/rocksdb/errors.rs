use std::{
    error::Error as StdError,
    fmt,
};

use rocksdb::Error;

#[derive(Debug, Clone)]
pub enum RocksDbBackendError {
    ConnectionBackendError(String),
    RocksDBError(String),
    TransactionDoesNotExist,
    UnknownError,
    //...
}

impl fmt::Display for RocksDbBackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RocksDbBackendError::ConnectionBackendError(ref reason) => write!(f, "Connection error: {:?}", reason),
            RocksDbBackendError::RocksDBError(ref reason) => write!(f, "RocksDB core error: {:?}", reason),
            RocksDbBackendError::TransactionDoesNotExist => write!(f, "Transaction does not exist"),
            RocksDbBackendError::UnknownError => write!(f, "Unknown error"),
        }
    }
}

// Allow this type to be treated like an error
impl StdError for RocksDbBackendError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            _ => None,
        }
    }
}

impl From<rocksdb::Error> for RocksDbBackendError {
    fn from(err: rocksdb::Error) -> Self {
        RocksDbBackendError::RocksDBError(String::from(err.to_string()))
    }
}