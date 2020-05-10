use std::{error::Error as StdError, fmt};

#[derive(Debug, Clone)]
pub enum RocksDbBackendError {
    RocksDBError(String),
    TransactionDoesNotExist,
}

impl fmt::Display for RocksDbBackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RocksDbBackendError::RocksDBError(ref reason) => write!(f, "RocksDB core error: {:?}", reason),
            RocksDbBackendError::TransactionDoesNotExist => write!(f, "Transaction does not exist"),
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
