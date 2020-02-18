use crate::messages::errors::MessageError;

use bee_network::Message;

use std::ops::Range;

const LEGACY_GOSSIP_REQUEST_SIZE: usize = 49;
const LEGACY_GOSSIP_CONSTANT_SIZE: usize = LEGACY_GOSSIP_REQUEST_SIZE;
const LEGACY_GOSSIP_VARIABLE_MIN_SIZE: usize = 292;
const LEGACY_GOSSIP_VARIABLE_MAX_SIZE: usize = 1604;

#[derive(Clone)]
pub struct LegacyGossip {
    transaction: Vec<u8>,
    request: [u8; LEGACY_GOSSIP_REQUEST_SIZE],
}

impl LegacyGossip {
    pub fn new(transaction: &Vec<u8>, request: [u8; LEGACY_GOSSIP_REQUEST_SIZE]) -> Self {
        // TODO clone ?
        Self {
            transaction: transaction.clone(),
            request: request,
        }
    }

    pub fn transaction(&self) -> &Vec<u8> {
        &self.transaction
    }

    pub fn request(&self) -> &[u8; LEGACY_GOSSIP_REQUEST_SIZE] {
        &self.request
    }
}

impl Message for LegacyGossip {
    type Error = MessageError;

    fn size_range() -> Range<usize> {
        (LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MIN_SIZE)
            ..(LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self {
            transaction: Vec::new(),
            request: [0; LEGACY_GOSSIP_REQUEST_SIZE],
        };

        let mut offset = 0;

        message
            .transaction
            .extend_from_slice(&bytes[offset..offset + bytes.len() - LEGACY_GOSSIP_REQUEST_SIZE]);
        offset += bytes.len() - LEGACY_GOSSIP_REQUEST_SIZE;

        message
            .request
            .copy_from_slice(&bytes[offset..offset + LEGACY_GOSSIP_REQUEST_SIZE]);

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = self.transaction.clone();

        bytes.extend_from_slice(&self.request);

        bytes
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO Move to utils ?
    fn eq<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for i in 0..a.len() {
            if a[i] != b[i] {
                return false;
            }
        }

        true
    }

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

    #[test]
    fn new_into_from_test() {
        let transaction: Vec<u8> = (500..1000).map(|i| i as u8).collect();
        let request = [
            160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155,
            232, 31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203,
            67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
        ];
        let message_from = LegacyGossip::new(&transaction, request);
        let message_to = LegacyGossip::from_bytes(&message_from.into_bytes()).unwrap();

        assert_eq!(message_to.transaction(), &transaction);
        assert_eq!(eq(message_to.request(), &request), true);
    }
}
