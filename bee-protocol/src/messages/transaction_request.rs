use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const TRANSACTION_REQUEST_CONSTANT_SIZE: usize = 49;

pub struct TransactionRequest {
    hash: [u8; TRANSACTION_REQUEST_CONSTANT_SIZE],
}

impl TransactionRequest {
    pub fn new(hash: [u8; TRANSACTION_REQUEST_CONSTANT_SIZE]) -> Self {
        Self { hash: hash }
    }
}

impl Message for TransactionRequest {
    fn size_range() -> Range<usize> {
        (TRANSACTION_REQUEST_CONSTANT_SIZE)..(TRANSACTION_REQUEST_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            hash: [0; TRANSACTION_REQUEST_CONSTANT_SIZE],
        })
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size_range_test() {
        assert_eq!(TransactionRequest::size_range().contains(&48), false);
        assert_eq!(TransactionRequest::size_range().contains(&49), true);
        assert_eq!(TransactionRequest::size_range().contains(&50), false);
    }
}
