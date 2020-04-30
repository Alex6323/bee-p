//! Header of the type-length-value encoding.

use std::convert::TryInto;

const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
pub(crate) const HEADER_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

pub(crate) struct Header {
    pub(crate) message_type: u8,
    pub(crate) message_length: u16,
}

impl Header {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            message_type: bytes[0],
            // TODO propagate error
            message_length: u16::from_be_bytes(bytes[HEADER_TYPE_SIZE..HEADER_SIZE].try_into().unwrap()),
        }
    }
}
