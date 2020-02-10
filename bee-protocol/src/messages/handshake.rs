use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_HANDSHAKE: u8 = 1;

pub struct Handshake {
    port: u16,
    timestamp: u64,
    coordinator: [u8; 49],
    minimum_weight_magnitude: u8,
    supported_messages: [u8; 32],
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; 49],
            minimum_weight_magnitude: 0,
            supported_messages: [0; 32],
        }
    }
}

impl Message for Handshake {
    fn size_range() -> (usize, usize) {
        (0, 0)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; 49],
            minimum_weight_magnitude: 0,
            supported_messages: [0; 32],
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
