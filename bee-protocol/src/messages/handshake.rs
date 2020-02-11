use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const HANDSHAKE_COORDINATOR_SIZE: usize = 49;
const HANDSHAKE_CONSTANT_SIZE: usize = 2 + 8 + HANDSHAKE_COORDINATOR_SIZE + 1;
const HANDSHAKE_VARIABLE_MIN_SIZE: usize = 1;
const HANDSHAKE_VARIABLE_MAX_SIZE: usize = 32;

pub struct Handshake {
    port: u16,
    timestamp: u64,
    coordinator: [u8; HANDSHAKE_COORDINATOR_SIZE],
    minimum_weight_magnitude: u8,
    supported_messages: [u8; HANDSHAKE_VARIABLE_MAX_SIZE],
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; HANDSHAKE_COORDINATOR_SIZE],
            minimum_weight_magnitude: 0,
            supported_messages: [0; HANDSHAKE_VARIABLE_MAX_SIZE],
        }
    }
}

impl Message for Handshake {
    fn size_range() -> Range<usize> {
        (HANDSHAKE_CONSTANT_SIZE + HANDSHAKE_VARIABLE_MIN_SIZE)
            ..(HANDSHAKE_CONSTANT_SIZE + HANDSHAKE_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; HANDSHAKE_COORDINATOR_SIZE],
            minimum_weight_magnitude: 0,
            supported_messages: [0; HANDSHAKE_VARIABLE_MAX_SIZE],
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
        assert_eq!(Handshake::size_range().contains(&60), false);
        assert_eq!(Handshake::size_range().contains(&61), true);
        assert_eq!(Handshake::size_range().contains(&62), true);

        assert_eq!(Handshake::size_range().contains(&91), true);
        assert_eq!(Handshake::size_range().contains(&92), true);
        assert_eq!(Handshake::size_range().contains(&93), false);
    }

    #[test]
    fn test() {
        Handshake::from_bytes(&[0; 40]);
    }
}
