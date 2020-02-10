use crate::messages::message::Message;

pub struct TransactionRequest {}

impl Message for TransactionRequest {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }
}
