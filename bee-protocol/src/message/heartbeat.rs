use crate::message::errors::MessageError;
use crate::message::Message;

use std::convert::TryInto;
use std::ops::Range;

const HEARTBEAT_ID: u8 = 0x06;

const HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE: usize = 4;
const HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE: usize = 4;
const HEARTBEAT_CONSTANT_SIZE: usize =
    HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE + HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE;

#[derive(Clone, Default)]
pub(crate) struct Heartbeat {
    pub(crate) first_solid_milestone_index: u32,
    pub(crate) last_solid_milestone_index: u32,
}

impl Heartbeat {
    pub(crate) fn new(first_solid_milestone_index: u32, last_solid_milestone_index: u32) -> Self {
        Self {
            first_solid_milestone_index: first_solid_milestone_index,
            last_solid_milestone_index: last_solid_milestone_index,
        }
    }
}

impl Message for Heartbeat {
    fn id() -> u8 {
        HEARTBEAT_ID
    }

    fn size_range() -> Range<usize> {
        (HEARTBEAT_CONSTANT_SIZE)..(HEARTBEAT_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self::default();
        let mut offset = 0;

        message.first_solid_milestone_index = u32::from_be_bytes(
            bytes[offset..offset + HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE;

        message.last_solid_milestone_index = u32::from_be_bytes(
            bytes[offset..offset + HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        [
            self.first_solid_milestone_index.to_be_bytes(),
            self.last_solid_milestone_index.to_be_bytes(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const FIRST_SOLID_MILESTONE_INDEX: u32 = 0x3dc297b4;
    const LAST_SOLID_MILESTONE_INDEX: u32 = 0x01181f9b;

    #[test]
    fn id_test() {
        assert_eq!(Heartbeat::id(), HEARTBEAT_ID);
    }

    #[test]
    fn size_range_test() {
        assert_eq!(Heartbeat::size_range().contains(&7), false);
        assert_eq!(Heartbeat::size_range().contains(&8), true);
        assert_eq!(Heartbeat::size_range().contains(&9), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match Heartbeat::from_bytes(&[0; 7]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 7),
            _ => unreachable!(),
        }
        match Heartbeat::from_bytes(&[0; 9]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 9),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: Heartbeat) {
        assert_eq!(
            message.first_solid_milestone_index,
            FIRST_SOLID_MILESTONE_INDEX
        );
        assert_eq!(
            message.last_solid_milestone_index,
            LAST_SOLID_MILESTONE_INDEX
        );
    }

    #[test]
    fn into_from_test() {
        let message_from = Heartbeat::new(FIRST_SOLID_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);

        into_from_eq(Heartbeat::from_bytes(&message_from.into_bytes()).unwrap());
    }

    #[test]
    fn full_into_from_test() {
        let message_from = Heartbeat::new(FIRST_SOLID_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);
        let bytes = message_from.into_full_bytes();

        into_from_eq(Heartbeat::from_full_bytes(&bytes[0..3], &bytes[3..]).unwrap());
    }
}
