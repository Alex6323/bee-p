use crate::message::Message;

pub struct TransactionBroadcast {}

impl Message for TransactionBroadcast {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }
}
