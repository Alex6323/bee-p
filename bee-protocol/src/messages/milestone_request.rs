use crate::messages::message::Message;

use std::ops::Range;

const _MILESTONE_REQUEST_TYPE_ID: u8 = 3;
const MILESTONE_REQUEST_CONSTANT_SIZE: usize = 8;

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
        (MILESTONE_REQUEST_CONSTANT_SIZE)..(MILESTONE_REQUEST_CONSTANT_SIZE + 1)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self { index: 0 }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
