use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_TRANSACTION_BROADCAST: u8 = 4;

pub struct TransactionBroadcast {}

impl TransactionBroadcast {
    pub fn new() -> Self {
        Self {}
    }
}

impl Message for TransactionBroadcast {
    fn size_range() -> (usize, usize) {
        (0, 0)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
