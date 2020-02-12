use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const TRANSACTION_REQUEST_HASH_SIZE: usize = 49;
const TRANSACTION_REQUEST_CONSTANT_SIZE: usize = TRANSACTION_REQUEST_HASH_SIZE;

pub struct TransactionRequest {
    hash: [u8; TRANSACTION_REQUEST_CONSTANT_SIZE],
}

impl TransactionRequest {
    pub fn new(hash: [u8; TRANSACTION_REQUEST_CONSTANT_SIZE]) -> Self {
        Self { hash: hash }
    }

    pub fn hash(&self) -> &[u8; TRANSACTION_REQUEST_CONSTANT_SIZE] {
        &self.hash
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

        let offset = 0;
        let mut hash = [0u8; TRANSACTION_REQUEST_HASH_SIZE];

        hash.copy_from_slice(&bytes[offset..offset + TRANSACTION_REQUEST_HASH_SIZE]);

        Ok(Self { hash: hash })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.hash.to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO Move to utils ?
    fn eq<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for i in 0..a.len() {
            if a[i] != b[i] {
                return false;
            }
        }

        true
    }

    #[test]
    fn size_range_test() {
        assert_eq!(TransactionRequest::size_range().contains(&48), false);
        assert_eq!(TransactionRequest::size_range().contains(&49), true);
        assert_eq!(TransactionRequest::size_range().contains(&50), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match TransactionRequest::from_bytes(&[0; 48]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 48),
            _ => unreachable!(),
        }
        match TransactionRequest::from_bytes(&[0; 50]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 50),
            _ => unreachable!(),
        }
    }

    #[test]
    fn new_into_from_test() {
        let hash = [
            160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155,
            232, 31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203,
            67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
        ];
        let message_from = TransactionRequest::new(hash);
        let message_to = TransactionRequest::from_bytes(&message_from.into_bytes()).unwrap();

        assert_eq!(eq(message_to.hash(), &hash), true);
    }
}
