use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_TRANSACTION_REQUEST: u8 = 5;

pub struct TransactionRequest {
    hash: [u8; 49],
}

impl TransactionRequest {
    pub fn new(hash: [u8; 49]) -> Self {
        Self { hash: hash }
    }
}

impl Message for TransactionRequest {
    fn size_range() -> (usize, usize) {
        (0, 0)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self { hash: [0; 49] }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
