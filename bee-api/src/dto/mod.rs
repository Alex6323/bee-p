use bee_message::MessageId;
use hex::FromHexError;
use bee_protocol::MilestoneIndex;
use serde::{Serialize, Deserialize};

pub mod get_info;

/// Marker traits.
pub trait DataBody {}
pub trait ErrorBody {}

/// Data response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataResponse<T: DataBody> {
    data: T,
}

/// Error response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse<T: ErrorBody> {
    error: T,
}

impl<T: DataBody> DataResponse<T> {
    /// Create a new data response.
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }
    /// Get data of the response.
    pub(crate) fn data(&self) -> &T {
        &self.data
    }
}

impl<T: ErrorBody> ErrorResponse<T> {
    /// Create a new error response.
    pub(crate) fn new(error: T) -> Self {
        Self { error }
    }
    /// Get error of the response.
    pub(crate) fn error(&self) -> &T {
        &self.error
    }
}