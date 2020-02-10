use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_LEGACY_GOSSIP: u8 = 2;

pub struct LegacyGossip {}

impl Message for LegacyGossip {
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
