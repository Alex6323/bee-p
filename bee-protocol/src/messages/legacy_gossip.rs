use crate::messages::message::Message;

use std::ops::Range;

const _TYPE_ID_MESSAGE_LEGACY_GOSSIP: u8 = 2;

pub struct LegacyGossip {
    transaction: Vec<u8>,
    request: [u8; 49],
}

impl LegacyGossip {
    pub fn new(transaction: Vec<u8>, request: [u8; 49]) -> Self {
        Self {
            transaction: transaction,
            request: request,
        }
    }
}

impl Message for LegacyGossip {
    fn size_range() -> Range<usize> {
        0..0
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            transaction: Vec::new(),
            request: [0; 49],
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
