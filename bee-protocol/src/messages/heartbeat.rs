use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const HEARTBEAT_CONSTANT_SIZE: usize = 8 + 8;

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

    fn from_bytes(_bytes: &[u8]) -> Result<Self, MessageError> {
        Ok(Self {
            first_solid_milestone_index: 0,
            last_solid_milestone_index: 0,
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
    fn empty() {}
}
