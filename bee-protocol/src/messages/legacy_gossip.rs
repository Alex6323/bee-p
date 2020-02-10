use crate::messages::message::Message;

use std::ops::Range;

const _LEGACY_GOSSIP_TYPE_ID: u8 = 2;
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
}

impl Message for LegacyGossip {
    fn size_range() -> Range<usize> {
        (LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MIN_SIZE)
            ..(LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            transaction: Vec::new(),
            request: [0; LEGACY_GOSSIP_CONSTANT_SIZE],
        }
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
