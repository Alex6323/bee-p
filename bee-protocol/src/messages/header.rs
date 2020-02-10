use crate::messages::message::Message;

const _TYPE_ID_MESSAGE_HEADER: u8 = 0;

pub struct Header {}

impl Header {
    pub fn new() -> Self {
        Self {}
    }
}

impl Message for Header {
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
