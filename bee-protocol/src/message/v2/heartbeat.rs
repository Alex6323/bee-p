// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! Heartbeat message of the protocol version 2

// TODO comment/uncomment when Chrysalis Pt1 is released.

use crate::message::Message;

use std::{convert::TryInto, ops::Range};

const LAST_SOLID_MILESTONE_INDEX_SIZE: usize = 4;
const SNAPSHOT_MILESTONE_INDEX_SIZE: usize = 4;
// const LAST_MILESTONE_INDEX_SIZE: usize = 4;
// const CONNECTED_PEERS_SIZE: usize = 1;
// const SYNCED_PEERS_SIZE: usize = 1;
const CONSTANT_SIZE: usize = LAST_SOLID_MILESTONE_INDEX_SIZE + SNAPSHOT_MILESTONE_INDEX_SIZE;
// const CONSTANT_SIZE: usize = LAST_SOLID_MILESTONE_INDEX_SIZE
//     + SNAPSHOT_MILESTONE_INDEX_SIZE
//     + LAST_MILESTONE_INDEX_SIZE
//     + CONNECTED_PEERS_SIZE
//     + SYNCED_PEERS_SIZE;

/// A message that informs about the part of the tangle currently being fully stored by a node.
/// This message is sent when a node:
/// - just got paired to another node;
/// - did a local snapshot and pruned away a part of the tangle;
/// - solidified a new milestone;
/// It also helps other nodes to know if they can ask it a specific transaction.
#[derive(Default)]
pub(crate) struct Heartbeat {
    /// Index of the last solid milestone.
    pub(crate) last_solid_milestone_index: u32,
    /// Index of the snapshotted milestone.
    pub(crate) snapshot_milestone_index: u32,
}

// #[derive(Default)]
// pub(crate) struct Heartbeat {
//     /// Index of the last solid milestone.
//     pub(crate) last_solid_milestone_index: u32,
//     /// Index of the snapshotted milestone.
//     pub(crate) snapshot_milestone_index: u32,
//     /// Index of the last milestone.
//     pub(crate) last_milestone_index: u32,
//     /// Number of connected peers.
//     pub(crate) connected_peers: u8,
//     /// Number of synced peers.
//     pub(crate) synced_peers: u8,
// }

impl Heartbeat {
    pub(crate) fn new(last_solid_milestone_index: u32, snapshot_milestone_index: u32) -> Self {
        Self {
            last_solid_milestone_index,
            snapshot_milestone_index,
        }
    }
}

// impl Heartbeat {
//     pub(crate) fn new(
//         last_solid_milestone_index: u32,
//         snapshot_milestone_index: u32,
//         last_milestone_index: u32,
//         connected_peers: u8,
//         synced_peers: u8,
//     ) -> Self {
//         Self {
//             last_solid_milestone_index,
//             snapshot_milestone_index,
//             last_milestone_index,
//             connected_peers,
//             synced_peers,
//         }
//     }
// }

impl Message for Heartbeat {
    const ID: u8 = 0x06;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message = Self::default();

        let (bytes, next) = bytes.split_at(LAST_SOLID_MILESTONE_INDEX_SIZE);
        message.last_solid_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, _) = next.split_at(SNAPSHOT_MILESTONE_INDEX_SIZE);
        message.snapshot_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        message
    }

    // fn from_bytes(bytes: &[u8]) -> Self {
    //     let mut message = Self::default();
    //
    //     let (bytes, next) = bytes.split_at(LAST_SOLID_MILESTONE_INDEX_SIZE);
    //     message.last_solid_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));
    //
    //     let (bytes, next) = next.split_at(SNAPSHOT_MILESTONE_INDEX_SIZE);
    //     message.snapshot_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));
    //
    //     let (bytes, next) = next.split_at(LAST_MILESTONE_INDEX_SIZE);
    //     message.last_milestone_index = u32::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));
    //
    //     let (bytes, next) = next.split_at(CONNECTED_PEERS_SIZE);
    //     message.connected_peers = u8::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));
    //
    //     let (bytes, _) = next.split_at(SYNCED_PEERS_SIZE);
    //     message.synced_peers = u8::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));
    //
    //     message
    // }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes[0..LAST_SOLID_MILESTONE_INDEX_SIZE].copy_from_slice(&self.last_solid_milestone_index.to_be_bytes());
        bytes[LAST_SOLID_MILESTONE_INDEX_SIZE..].copy_from_slice(&self.snapshot_milestone_index.to_be_bytes());
    }

    // fn into_bytes(self, bytes: &mut [u8]) {
    //     let (bytes, next) = bytes.split_at_mut(LAST_SOLID_MILESTONE_INDEX_SIZE);
    //     bytes.copy_from_slice(&self.last_solid_milestone_index.to_be_bytes());
    //     let (bytes, next) = next.split_at_mut(SNAPSHOT_MILESTONE_INDEX_SIZE);
    //     bytes.copy_from_slice(&self.snapshot_milestone_index.to_be_bytes());
    //     let (bytes, next) = next.split_at_mut(LAST_MILESTONE_INDEX_SIZE);
    //     bytes.copy_from_slice(&self.last_milestone_index.to_be_bytes());
    //     let (bytes, next) = next.split_at_mut(CONNECTED_PEERS_SIZE);
    //     bytes.copy_from_slice(&self.connected_peers.to_be_bytes());
    //     let (bytes, _) = next.split_at_mut(SYNCED_PEERS_SIZE);
    //     bytes.copy_from_slice(&self.synced_peers.to_be_bytes());
    // }
}

#[cfg(test)]
mod tests {

    use super::*;

    const LAST_SOLID_MILESTONE_INDEX: u32 = 0x0118_1f9b;
    const SNAPSHOT_MILESTONE_INDEX: u32 = 0x3dc2_97b4;
    // const LAST_MILESTONE_INDEX: u32 = 0x60be_20c2;
    // const CONNECTED_PEERS: u8 = 12;
    // const SYNCED_PEERS: u8 = 5;

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

    // #[test]
    // fn size_range() {
    //     assert_eq!(Heartbeat::size_range().contains(&13), false);
    //     assert_eq!(Heartbeat::size_range().contains(&14), true);
    //     assert_eq!(Heartbeat::size_range().contains(&15), false);
    // }

    // #[test]
    // fn size() {
    //     let message = Heartbeat::new(
    //         SNAPSHOT_MILESTONE_INDEX,
    //         LAST_SOLID_MILESTONE_INDEX,
    //         LAST_MILESTONE_INDEX,
    //         CONNECTED_PEERS,
    //         SYNCED_PEERS,
    //     );
    //
    //     assert_eq!(message.size(), CONSTANT_SIZE);
    // }

    #[test]
    fn size() {
        let message = Heartbeat::new(SNAPSHOT_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);

        assert_eq!(message.size(), CONSTANT_SIZE);
    }

    // #[test]
    // fn into_from() {
    //     let message_from = Heartbeat::new(
    //         SNAPSHOT_MILESTONE_INDEX,
    //         LAST_SOLID_MILESTONE_INDEX,
    //         LAST_MILESTONE_INDEX,
    //         CONNECTED_PEERS,
    //         SYNCED_PEERS,
    //     );
    //     let mut bytes = vec![0u8; message_from.size()];
    //     message_from.into_bytes(&mut bytes);
    //     let message_to = Heartbeat::from_bytes(&bytes);
    //
    //     assert_eq!(message_to.last_solid_milestone_index, SNAPSHOT_MILESTONE_INDEX);
    //     assert_eq!(message_to.snapshot_milestone_index, LAST_SOLID_MILESTONE_INDEX);
    //     assert_eq!(message_to.last_milestone_index, LAST_MILESTONE_INDEX);
    //     assert_eq!(message_to.connected_peers, CONNECTED_PEERS);
    //     assert_eq!(message_to.synced_peers, SYNCED_PEERS);
    // }

    #[test]
    fn into_from() {
        let message_from = Heartbeat::new(SNAPSHOT_MILESTONE_INDEX, LAST_SOLID_MILESTONE_INDEX);
        let mut bytes = vec![0u8; message_from.size()];
        message_from.into_bytes(&mut bytes);
        let message_to = Heartbeat::from_bytes(&bytes);

        assert_eq!(message_to.last_solid_milestone_index, SNAPSHOT_MILESTONE_INDEX);
        assert_eq!(message_to.snapshot_milestone_index, LAST_SOLID_MILESTONE_INDEX);
    }
}
