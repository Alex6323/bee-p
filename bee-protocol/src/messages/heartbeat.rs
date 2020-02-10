use crate::messages::message::Message;

use std::ops::Range;

const _TYPE_ID_MESSAGE_HEARTBEAT: u8 = 6;

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
        0..0
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            first_solid_milestone_index: 0,
            last_solid_milestone_index: 0,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}
