use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_HANDSHAKE: u8 = 1;

pub struct Handshake {}

impl Message for Handshake {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }
}
