use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const LEGACY_GOSSIP_CONSTANT_SIZE: usize = 49;
const LEGACY_GOSSIP_VARIABLE_MIN_SIZE: usize = 292;
const LEGACY_GOSSIP_VARIABLE_MAX_SIZE: usize = 1604;

pub struct LegacyGossip {
    transaction: Vec<u8>,
    request: [u8; LEGACY_GOSSIP_CONSTANT_SIZE],
}

impl LegacyGossip {
    pub fn new(transaction: Vec<u8>, request: [u8; LEGACY_GOSSIP_CONSTANT_SIZE]) -> Self {
        Self {
            transaction: transaction,
            request: request,
        }
    }

    pub fn transaction(&self) -> &Vec<u8> {
        &self.transaction
    }

    pub fn request(&self) -> &[u8; LEGACY_GOSSIP_CONSTANT_SIZE] {
        &self.request
    }
}

impl Message for LegacyGossip {
    fn size_range() -> Range<usize> {
        (LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MIN_SIZE)
            ..(LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            transaction: Vec::new(),
            request: [0; LEGACY_GOSSIP_CONSTANT_SIZE],
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
        assert_eq!(LegacyGossip::size_range().contains(&340), false);
        assert_eq!(LegacyGossip::size_range().contains(&341), true);
        assert_eq!(LegacyGossip::size_range().contains(&342), true);

        assert_eq!(LegacyGossip::size_range().contains(&1652), true);
        assert_eq!(LegacyGossip::size_range().contains(&1653), true);
        assert_eq!(LegacyGossip::size_range().contains(&1654), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match LegacyGossip::from_bytes(&[0; 340]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 340),
            _ => unreachable!(),
        }
        match LegacyGossip::from_bytes(&[0; 1654]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 1654),
            _ => unreachable!(),
        }
    }
}
