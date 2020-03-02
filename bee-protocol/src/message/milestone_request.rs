use crate::message::errors::MessageError;
use crate::message::Message;

use std::convert::TryInto;
use std::ops::Range;

const MILESTONE_REQUEST_ID: u8 = 0x03;

const MILESTONE_REQUEST_INDEX_SIZE: usize = 8;
const MILESTONE_REQUEST_CONSTANT_SIZE: usize = MILESTONE_REQUEST_INDEX_SIZE;

#[derive(Clone, Default)]
pub(crate) struct MilestoneRequest {
    pub(crate) index: u64,
}

impl MilestoneRequest {
    pub fn new(index: u64) -> Self {
        Self { index: index }
    }
}

impl Message for MilestoneRequest {
    fn id() -> u8 {
        MILESTONE_REQUEST_ID
    }

    fn size_range() -> Range<usize> {
        (MILESTONE_REQUEST_CONSTANT_SIZE)..(MILESTONE_REQUEST_CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            index: u64::from_be_bytes(
                bytes[0..MILESTONE_REQUEST_INDEX_SIZE]
                    .try_into()
                    .map_err(|_| MessageError::InvalidMessageField)?,
            ),
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.index.to_be_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const INDEX: u64 = 0x3cd44cef7195aa20;

    #[test]
    fn id_test() {
        assert_eq!(MilestoneRequest::id(), MILESTONE_REQUEST_ID);
    }

    #[test]
    fn size_range_test() {
        assert_eq!(MilestoneRequest::size_range().contains(&7), false);
        assert_eq!(MilestoneRequest::size_range().contains(&8), true);
        assert_eq!(MilestoneRequest::size_range().contains(&9), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match MilestoneRequest::from_bytes(&[0; 7]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 7),
            _ => unreachable!(),
        }
        match MilestoneRequest::from_bytes(&[0; 9]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 9),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: MilestoneRequest) {
        assert_eq!(message.index, INDEX);
    }

    #[test]
    fn into_from_test() {
        let message_from = MilestoneRequest::new(INDEX);

        into_from_eq(MilestoneRequest::from_bytes(&message_from.into_bytes()).unwrap());
    }

    #[test]
    fn full_into_from_test() {
        let message_from = MilestoneRequest::new(INDEX);
        let bytes = message_from.into_full_bytes();

        into_from_eq(MilestoneRequest::from_full_bytes(&bytes[0..3], &bytes[3..]).unwrap());
    }
}
