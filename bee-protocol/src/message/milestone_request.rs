use crate::message::errors::ProtocolMessageError;

use bee_network::Message;

use std::convert::TryInto;
use std::ops::Range;

const MILESTONE_REQUEST_INDEX_SIZE: usize = 8;
const MILESTONE_REQUEST_CONSTANT_SIZE: usize = MILESTONE_REQUEST_INDEX_SIZE;

#[derive(Clone)]
pub struct MilestoneRequest {
    index: u64,
}

impl MilestoneRequest {
    pub fn new(index: u64) -> Self {
        Self { index: index }
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

impl Message for MilestoneRequest {
    type Error = ProtocolMessageError;

    fn size_range() -> Range<usize> {
        (MILESTONE_REQUEST_CONSTANT_SIZE)..(MILESTONE_REQUEST_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolMessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(ProtocolMessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            // Safe to unwrap since we made sure it has the right size
            index: u64::from_be_bytes(bytes[0..MILESTONE_REQUEST_INDEX_SIZE].try_into().unwrap()),
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.index.to_be_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size_range_test() {
        assert_eq!(MilestoneRequest::size_range().contains(&7), false);
        assert_eq!(MilestoneRequest::size_range().contains(&8), true);
        assert_eq!(MilestoneRequest::size_range().contains(&9), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match MilestoneRequest::from_bytes(&[0; 7]) {
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 7),
            _ => unreachable!(),
        }
        match MilestoneRequest::from_bytes(&[0; 9]) {
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 9),
            _ => unreachable!(),
        }
    }

    #[test]
    fn new_into_from_test() {
        let message_from = MilestoneRequest::new(0x3cd44cef7195aa20);
        let message_to = MilestoneRequest::from_bytes(&message_from.into_bytes()).unwrap();

        assert_eq!(message_to.index(), 0x3cd44cef7195aa20);
    }
}
