use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_TRANSACTION_BROADCAST: u8 = 4;

pub struct TransactionBroadcast {}

impl Message for TransactionBroadcast {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }

    fn size() -> (usize, usize) {
        (0, 0)
    }
}
