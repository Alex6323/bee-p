use crate::messages::message::Message;

use std::ops::Range;

const _TYPE_ID_MESSAGE_MILESTONE_REQUEST: u8 = 3;

pub struct MilestoneRequest {
    index: u64,
}

impl MilestoneRequest {
    pub fn new(index: u64) -> Self {
        Self { index: index }
    }
}

impl Message for MilestoneRequest {
    fn size_range() -> Range<usize> {
        0..0
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self { index: 0 }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
