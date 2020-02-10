use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_TRANSACTION_REQUEST: u8 = 5;

pub struct TransactionRequest {}

impl TransactionRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl Message for TransactionRequest {
    fn size() -> (usize, usize) {
        (0, 0)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
