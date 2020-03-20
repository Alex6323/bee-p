use std::fmt;
use std::io;

use std::error::Error as StdError;

type MissingItem = &'static str;

pub enum Error {
    ConfigLoadError(io::Error),
    ConfigSaveError(io::Error),
    ConfigToJsonError(serde_json::Error),
    ConfigFromJsonError(serde_json::Error),
    ConfigIncompleteError(MissingItem),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigLoadError(e) => write!(f, "Config could not be loaded from file due to: {:?}\n{}", e.kind(), e),
            Error::ConfigSaveError(e) => write!(f, "Config could not be saved to file due to {:?}", e.kind()),
            Error::ConfigToJsonError(e) => write!(f, "Config could not be serialized to JSON due to {:?}", e),
            Error::ConfigFromJsonError(e) => write!(f, "Config could not be deserialized from JSON due to {:?}", e),
            Error::ConfigIncompleteError(e) => write!(f, "Config misses the following item: {}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;