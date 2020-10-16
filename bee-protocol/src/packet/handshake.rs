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

//! Handshake packet of the protocol.

use crate::packet::Packet;

use std::{
    convert::TryInto,
    ops::Range,
    time::{SystemTime, UNIX_EPOCH},
};

const PORT_SIZE: usize = 2;
const TIMESTAMP_SIZE: usize = 8;
const COORDINATOR_SIZE: usize = 32;
const MINIMUM_WEIGHT_MAGNITUDE_SIZE: usize = 1;
const VERSION_SIZE: usize = 2;
const CONSTANT_SIZE: usize =
    PORT_SIZE + TIMESTAMP_SIZE + COORDINATOR_SIZE + MINIMUM_WEIGHT_MAGNITUDE_SIZE + VERSION_SIZE;

/// A packet that allows two nodes to pair.
///
/// Contains useful information to verify that the pairing node is operating on the same configuration.
/// Any difference in configuration will end up in the connection being closed and the nodes not pairing.
pub(crate) struct Handshake {
    /// Protocol port of the node.
    pub(crate) port: u16,
    /// Timestamp - in ms - when the packet was created by the node.
    pub(crate) timestamp: u64,
    /// Public key of the coordinator being tracked by the node.
    pub(crate) coordinator: [u8; COORDINATOR_SIZE],
    /// Minimum Weight Magnitude of the node.
    pub(crate) minimum_weight_magnitude: u8,
    /// Protocol version supported by the node.
    pub(crate) version: u16,
}

impl Handshake {
    pub(crate) fn new(
        port: u16,
        coordinator: &[u8; COORDINATOR_SIZE],
        minimum_weight_magnitude: u8,
        version: u16,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
        let mut self_coordinator = [0; COORDINATOR_SIZE];

        self_coordinator.copy_from_slice(coordinator);

        Self {
            port,
            timestamp,
            coordinator: self_coordinator,
            minimum_weight_magnitude,
            version,
        }
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; COORDINATOR_SIZE],
            minimum_weight_magnitude: 0,
            version: 0,
        }
    }
}

impl Packet for Handshake {
    const ID: u8 = 0x01;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut packet = Self::default();

        let (bytes, next) = bytes.split_at(PORT_SIZE);
        packet.port = u16::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(TIMESTAMP_SIZE);
        packet.timestamp = u64::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(COORDINATOR_SIZE);
        packet.coordinator.copy_from_slice(bytes);

        let (bytes, next) = next.split_at(MINIMUM_WEIGHT_MAGNITUDE_SIZE);
        packet.minimum_weight_magnitude = u8::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, _) = next.split_at(VERSION_SIZE);
        packet.version = u16::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        packet
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        let (bytes, next) = bytes.split_at_mut(PORT_SIZE);
        bytes.copy_from_slice(&self.port.to_le_bytes());

        let (bytes, next) = next.split_at_mut(TIMESTAMP_SIZE);
        bytes.copy_from_slice(&self.timestamp.to_le_bytes());

        let (bytes, next) = next.split_at_mut(COORDINATOR_SIZE);
        bytes.copy_from_slice(&self.coordinator);

        let (bytes, next) = next.split_at_mut(MINIMUM_WEIGHT_MAGNITUDE_SIZE);
        bytes.copy_from_slice(&self.minimum_weight_magnitude.to_le_bytes());

        let (bytes, _) = next.split_at_mut(VERSION_SIZE);
        bytes.copy_from_slice(&self.version.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const PORT: u16 = 0xcd98;
    const COORDINATOR: [u8; COORDINATOR_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232, 31, 255, 208, 9, 126,
        21, 82, 57, 180, 237, 182, 101,
    ];
    const MINIMUM_WEIGHT_MAGNITUDE: u8 = 0x6e;
    const VERSION: u16 = 1;

    #[test]
    fn id() {
        assert_eq!(Handshake::ID, 1);
    }

    #[test]
    fn size_range() {
        assert_eq!(Handshake::size_range().contains(&44), false);
        assert_eq!(Handshake::size_range().contains(&45), true);
        assert_eq!(Handshake::size_range().contains(&46), false);
    }

    #[test]
    fn size() {
        let packet = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, VERSION);

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, VERSION);
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = Handshake::from_bytes(&bytes);

        // TODO test timestamp
        assert_eq!(packet_to.port, PORT);
        assert!(packet_to.coordinator.eq(&COORDINATOR));
        assert_eq!(packet_to.minimum_weight_magnitude, MINIMUM_WEIGHT_MAGNITUDE);
        assert_eq!(packet_to.version, VERSION);
    }
}
