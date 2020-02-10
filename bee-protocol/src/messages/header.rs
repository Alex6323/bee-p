use crate::messages::message::Message;

use std::ops::Range;

const _TYPE_ID_MESSAGE_HEADER: u8 = 0;

pub struct Header {
    message_type: u8,
    message_length: u16,
}

impl Header {
    pub fn new(message_type: u8, message_length: u16) -> Self {
        Self {
            message_type: message_type,
            message_length: message_length,
        }
    }
}

impl Message for Header {
    fn size_range() -> Range<usize> {
        0..0
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            message_type: 0,
            message_length: 0,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
