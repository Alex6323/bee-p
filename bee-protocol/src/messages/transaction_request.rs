use crate::messages::message::Message;

use std::ops::Range;

const _TRANSACTION_REQUEST_TYPE_ID: u8 = 5;
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

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            hash: [0; TRANSACTION_REQUEST_CONSTANT_SIZE],
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
