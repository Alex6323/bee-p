use crate::messages::errors::MessageError;
use crate::messages::message::Message;
use crate::messages::MessageType;

use std::convert::TryInto;
use std::ops::Range;

const HEADER_CONSTANT_SIZE: usize = 1 + 2;

pub struct Header {
    message_type: MessageType,
    message_length: u16,
}

impl Header {
    pub fn new(message_type: MessageType, message_length: u16) -> Self {
        Self {
            message_type: message_type,
            message_length: message_length,
        }
    }
}

impl Message for Header {
    fn size_range() -> Range<usize> {
        (HEADER_CONSTANT_SIZE)..(HEADER_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            message_type: bytes[0].try_into()?,
            message_length: 0,
        })
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size_range_test() {
        assert_eq!(Header::size_range().contains(&2), false);
        assert_eq!(Header::size_range().contains(&3), true);
        assert_eq!(Header::size_range().contains(&4), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match Header::from_bytes(&[0; 2]) {
            Err(MessageError::InvalidMessageLength(l)) => assert_eq!(l, 2),
            _ => unreachable!(),
        }
        match Header::from_bytes(&[0; 4]) {
            Err(MessageError::InvalidMessageLength(l)) => assert_eq!(l, 4),
            _ => unreachable!(),
        }
    }
}
