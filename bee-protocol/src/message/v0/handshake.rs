//! Handshake message of the protocol version 0

use crate::message::Message;

use std::{
    convert::TryInto,
    ops::Range,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

const PORT_SIZE: usize = 2;
const TIMESTAMP_SIZE: usize = 8;
const COORDINATOR_SIZE: usize = 49;
const MINIMUM_WEIGHT_MAGNITUDE_SIZE: usize = 1;
const CONSTANT_SIZE: usize = PORT_SIZE + TIMESTAMP_SIZE + COORDINATOR_SIZE + MINIMUM_WEIGHT_MAGNITUDE_SIZE;
const VARIABLE_MIN_SIZE: usize = 1;
const VARIABLE_MAX_SIZE: usize = 32;

#[derive(Clone)]
pub(crate) struct Handshake {
    pub(crate) port: u16,
    pub(crate) timestamp: u64,
    pub(crate) coordinator: [u8; COORDINATOR_SIZE],
    pub(crate) minimum_weight_magnitude: u8,
    pub(crate) supported_versions: Vec<u8>,
}

impl Handshake {
    pub(crate) fn new(
        port: u16,
        coordinator: &[u8; COORDINATOR_SIZE],
        minimum_weight_magnitude: u8,
        supported_versions: &[u8],
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
        let mut self_coordinator = [0; COORDINATOR_SIZE];

        self_coordinator.copy_from_slice(coordinator);

        Self {
            port: port,
            timestamp: timestamp,
            coordinator: self_coordinator,
            minimum_weight_magnitude: minimum_weight_magnitude,
            supported_versions: supported_versions.to_vec(),
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
            supported_versions: Default::default(),
        }
    }
}

impl Message for Handshake {
    const ID: u8 = 0x01;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE + VARIABLE_MIN_SIZE)..(CONSTANT_SIZE + VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message = Self::default();

        let (bytes, next) = bytes.split_at(PORT_SIZE);
        message.port = u16::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(TIMESTAMP_SIZE);
        message.timestamp = u64::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(COORDINATOR_SIZE);
        message.coordinator.copy_from_slice(bytes);

        let (bytes, next) = next.split_at(MINIMUM_WEIGHT_MAGNITUDE_SIZE);
        message.minimum_weight_magnitude = u8::from_be_bytes(bytes.try_into().expect("Invalid buffer size"));

        message.supported_versions = next.to_vec();

        message
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE + self.supported_versions.len()
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        let (bytes, next) = bytes.split_at_mut(PORT_SIZE);
        bytes.copy_from_slice(&self.port.to_be_bytes());

        let (bytes, next) = next.split_at_mut(TIMESTAMP_SIZE);
        bytes.copy_from_slice(&self.timestamp.to_be_bytes());

        let (bytes, next) = next.split_at_mut(COORDINATOR_SIZE);
        bytes.copy_from_slice(&self.coordinator);

        let (bytes, next) = next.split_at_mut(MINIMUM_WEIGHT_MAGNITUDE_SIZE);
        bytes.copy_from_slice(&self.minimum_weight_magnitude.to_be_bytes());

        next.copy_from_slice(&self.supported_versions);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_test::slices::slice_eq;

    const PORT: u16 = 0xcd98;
    const COORDINATOR: [u8; COORDINATOR_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232, 31, 255, 208, 9, 126,
        21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203, 67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
    ];
    const MINIMUM_WEIGHT_MAGNITUDE: u8 = 0x6e;
    const SUPPORTED_VERSIONS: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    #[test]
    fn id() {
        assert_eq!(Handshake::ID, 1);
    }

    #[test]
    fn size_range() {
        assert_eq!(Handshake::size_range().contains(&60), false);
        assert_eq!(Handshake::size_range().contains(&61), true);
        assert_eq!(Handshake::size_range().contains(&62), true);

        assert_eq!(Handshake::size_range().contains(&91), true);
        assert_eq!(Handshake::size_range().contains(&92), true);
        assert_eq!(Handshake::size_range().contains(&93), false);
    }

    #[test]
    fn size() {
        let message = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);

        assert_eq!(message.size(), CONSTANT_SIZE + 10);
    }

    #[test]
    fn into_from() {
        let message_from = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);
        let mut bytes = vec![0u8; message_from.size()];
        message_from.into_bytes(&mut bytes);
        let message_to = Handshake::from_bytes(&bytes);

        assert_eq!(message_to.port, PORT);
        assert!(slice_eq(&message_to.coordinator, &COORDINATOR));
        assert_eq!(message_to.minimum_weight_magnitude, MINIMUM_WEIGHT_MAGNITUDE);
        assert!(slice_eq(&message_to.supported_versions, &SUPPORTED_VERSIONS));
    }
}
