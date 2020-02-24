use crate::message::errors::ProtocolMessageError;
use crate::message::Message;

use std::ops::Range;

const TRANSACTION_REQUEST_ID: u8 = 0x05;

const TRANSACTION_REQUEST_HASH_SIZE: usize = 49;
const TRANSACTION_REQUEST_CONSTANT_SIZE: usize = TRANSACTION_REQUEST_HASH_SIZE;

#[derive(Clone)]
pub(crate) struct TransactionRequest {
    hash: [u8; TRANSACTION_REQUEST_HASH_SIZE],
}

impl TransactionRequest {
    pub fn new(hash: [u8; TRANSACTION_REQUEST_HASH_SIZE]) -> Self {
        Self { hash: hash }
    }

    pub fn hash(&self) -> &[u8; TRANSACTION_REQUEST_HASH_SIZE] {
        &self.hash
    }
}

impl Message for TransactionRequest {
    fn id() -> u8 {
        TRANSACTION_REQUEST_ID
    }

    fn size_range() -> Range<usize> {
        (TRANSACTION_REQUEST_CONSTANT_SIZE)..(TRANSACTION_REQUEST_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolMessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(ProtocolMessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self {
            hash: [0u8; TRANSACTION_REQUEST_HASH_SIZE],
        };

        message
            .hash
            .copy_from_slice(&bytes[0..TRANSACTION_REQUEST_HASH_SIZE]);

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.hash.to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bee_test::slices::slice_eq;

    const HASH: [u8; TRANSACTION_REQUEST_HASH_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232,
        31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203, 67, 93,
        74, 238, 57, 39, 51, 169, 193, 124, 254,
    ];

    #[test]
    fn id_test() {
        assert_eq!(TransactionRequest::id(), TRANSACTION_REQUEST_ID);
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
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 48),
            _ => unreachable!(),
        }
        match TransactionRequest::from_bytes(&[0; 50]) {
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 50),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: TransactionRequest) {
        assert_eq!(slice_eq(message.hash(), &HASH), true);
    }

    #[test]
    fn into_from_test() {
        let message_from = TransactionRequest::new(HASH);

        into_from_eq(TransactionRequest::from_bytes(&message_from.into_bytes()).unwrap());
    }

    #[test]
    fn full_into_from_test() {
        let message_from = TransactionRequest::new(HASH);

        into_from_eq(TransactionRequest::from_full_bytes(&message_from.into_full_bytes()).unwrap());
    }
}
