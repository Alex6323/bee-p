use crate::message::errors::MessageError;
use crate::message::Message;
//
use std::convert::TryInto;
use std::ops::Range;

const HEADER_ID: u8 = 0x00;

const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
const HEADER_CONSTANT_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

#[derive(Clone, Default)]
pub(crate) struct Header {
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

    pub fn message_type(&self) -> u8 {
        self.message_type
    }

    pub fn message_length(&self) -> u16 {
        self.message_length
    }
}

impl Message for Header {
    fn id() -> u8 {
        HEADER_ID
    }

    fn size_range() -> Range<usize> {
        HEADER_CONSTANT_SIZE..(HEADER_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self::default();
        let mut offset = 0;

        message.message_type = u8::from_be_bytes(
            bytes[offset..offset + HEADER_TYPE_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HEADER_TYPE_SIZE;

        message.message_length = u16::from_be_bytes(
            bytes[offset..offset + HEADER_LENGTH_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HEADER_LENGTH_SIZE;

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = self.message_type.to_be_bytes().to_vec();

        bytes.extend_from_slice(&self.message_length.to_be_bytes());

        bytes
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const MESSAGE_TYPE: u8 = 4;
    const MESSAGE_LENGTH: u16 = 42;

    #[test]
    fn id_test() {
        assert_eq!(Header::id(), HEADER_ID);
    }

    #[test]
    fn size_range_test() {
        assert_eq!(Header::size_range().contains(&2), false);
        assert_eq!(Header::size_range().contains(&3), true);
        assert_eq!(Header::size_range().contains(&4), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match Header::from_bytes(&[0; 60]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 60),
            _ => unreachable!(),
        }
        match Header::from_bytes(&[0; 93]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 93),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: Header) {
        assert_eq!(message.message_type(), MESSAGE_TYPE);
        assert_eq!(message.message_length(), MESSAGE_LENGTH);
    }

    #[test]
    fn into_from_test() {
        let message_from = Header::new(MESSAGE_TYPE, MESSAGE_LENGTH);

        into_from_eq(Header::from_bytes(&message_from.into_bytes()).unwrap());
    }
}
