use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::convert::TryInto;
use std::ops::Range;

const HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE: usize = 8;
const HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE: usize = 8;
const HEARTBEAT_CONSTANT_SIZE: usize =
    HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE + HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE;

pub struct Heartbeat {
    first_solid_milestone_index: u64,
    last_solid_milestone_index: u64,
}

impl Heartbeat {
    pub fn new(first_solid_milestone_index: u64, last_solid_milestone_index: u64) -> Self {
        Self {
            first_solid_milestone_index: first_solid_milestone_index,
            last_solid_milestone_index: last_solid_milestone_index,
        }
    }
}

impl Message for Heartbeat {
    fn size_range() -> Range<usize> {
        (HEARTBEAT_CONSTANT_SIZE)..(HEARTBEAT_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut offset = 0;

        // Safe to unwrap since we made sure it has the right size
        let first_solid_milestone_index = u64::from_be_bytes(
            bytes[offset..offset + HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE]
                .try_into()
                .unwrap(),
        );
        offset += HEARTBEAT_FIRST_SOLID_MILESTONE_INDEX_SIZE;

        // Safe to unwrap since we made sure it has the right size
        let last_solid_milestone_index = u64::from_be_bytes(
            bytes[offset..offset + HEARTBEAT_LAST_SOLID_MILESTONE_INDEX_SIZE]
                .try_into()
                .unwrap(),
        );

        Ok(Self {
            first_solid_milestone_index: first_solid_milestone_index,
            last_solid_milestone_index: last_solid_milestone_index,
        })
    }

    fn to_bytes(self) -> Vec<u8> {
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

    #[test]
    fn size_range_test() {
        assert_eq!(Heartbeat::size_range().contains(&15), false);
        assert_eq!(Heartbeat::size_range().contains(&16), true);
        assert_eq!(Heartbeat::size_range().contains(&17), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match Heartbeat::from_bytes(&[0; 15]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 15),
            _ => unreachable!(),
        }
        match Heartbeat::from_bytes(&[0; 17]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 17),
            _ => unreachable!(),
        }
    }

    #[test]
    fn new_to_from_test() {
        let message_from = Heartbeat::new(0xe2659070221a4319, 0x3500fbdebbfdfb2c);
        let message_to = Heartbeat::from_bytes(&message_from.to_bytes()).unwrap();

        assert_eq!(message_to.first_solid_milestone_index, 0xe2659070221a4319);
        assert_eq!(message_to.last_solid_milestone_index, 0x3500fbdebbfdfb2c);
    }
}
