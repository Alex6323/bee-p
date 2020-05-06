//! Heartbeat message of the protocol version 2

use crate::message::Message;

use std::{convert::TryInto, ops::Range};

const SOLID_MILESTONE_INDEX_SIZE: usize = 4;
const SNAPSHOT_MILESTONE_INDEX_SIZE: usize = 4;
const CONSTANT_SIZE: usize = SOLID_MILESTONE_INDEX_SIZE + SNAPSHOT_MILESTONE_INDEX_SIZE;

/// A message that informs about the part of the tangle currently being fully stored by a node.
/// This message is sent when a node:
/// * just got paired to another node;
/// * did a local snapshot and pruned away a part of the tangle;
/// * solidified a new milestone;
/// It also helps other nodes to know if they can ask it a specific transaction.
#[derive(Clone, Default)]
pub(crate) struct Heartbeat {
    /// Index of the last solid milestone.
    pub(crate) solid_milestone_index: u32,
    /// Index of the snapshotted milestone.
    pub(crate) snapshot_milestone_index: u32,
}

impl Heartbeat {
    pub(crate) fn new(solid_milestone_index: u32, snapshot_milestone_index: u32) -> Self {
        Self {
            solid_milestone_index: solid_milestone_index,
            snapshot_milestone_index: snapshot_milestone_index,
        }
    }
}

impl Message for Heartbeat {
    const ID: u8 = 0x06;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message = Self::default();

        let (bytes, next) = bytes.split_at(SOLID_MILESTONE_INDEX_SIZE);
        message.solid_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, _) = next.split_at(SNAPSHOT_MILESTONE_INDEX_SIZE);
        message.snapshot_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        message
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes[0..SOLID_MILESTONE_INDEX_SIZE].copy_from_slice(&self.solid_milestone_index.to_be_bytes());
        bytes[SOLID_MILESTONE_INDEX_SIZE..].copy_from_slice(&self.snapshot_milestone_index.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const FIRST_SOLID_MILESTONE_INDEX: u32 = 0x3dc297b4;
    const LAST_SOLID_MILESTONE_INDEX: u32 = 0x01181f9b;

    #[test]
    fn id() {
        assert_eq!(Heartbeat::ID, 6);
    }

    #[test]
    fn size_range() {
        assert_eq!(Heartbeat::size_range().contains(&7), false);
        assert_eq!(Heartbeat::size_range().contains(&8), true);
        assert_eq!(Heartbeat::size_range().contains(&9), false);
    }

    #[test]
    fn size() {
        let message = Heartbeat::new(FIRST_SOLID_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);

        assert_eq!(message.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let message_from = Heartbeat::new(FIRST_SOLID_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);
        let mut bytes = vec![0u8; message_from.size()];
        message_from.into_bytes(&mut bytes);
        let message_to = Heartbeat::from_bytes(&bytes);

        assert_eq!(message_to.solid_milestone_index, FIRST_SOLID_MILESTONE_INDEX);
        assert_eq!(message_to.snapshot_milestone_index, LAST_SOLID_MILESTONE_INDEX);
    }
}
