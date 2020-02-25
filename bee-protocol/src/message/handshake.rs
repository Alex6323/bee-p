use crate::message::errors::MessageError;
use crate::message::Message;

use std::convert::TryInto;
use std::ops::Range;

const HANDSHAKE_ID: u8 = 0x01;

const HANDSHAKE_PORT_SIZE: usize = 2;
const HANDSHAKE_TIMESTAMP_SIZE: usize = 8;
const HANDSHAKE_COORDINATOR_SIZE: usize = 49;
const HANDSHAKE_MINIMUM_WEIGHT_MAGNITUDE: usize = 1;
const HANDSHAKE_CONSTANT_SIZE: usize = HANDSHAKE_PORT_SIZE
    + HANDSHAKE_TIMESTAMP_SIZE
    + HANDSHAKE_COORDINATOR_SIZE
    + HANDSHAKE_MINIMUM_WEIGHT_MAGNITUDE;
const HANDSHAKE_VARIABLE_MIN_SIZE: usize = 1;
const HANDSHAKE_VARIABLE_MAX_SIZE: usize = 32;

#[derive(Clone)]
pub(crate) struct Handshake {
    port: u16,
    timestamp: u64,
    coordinator: [u8; HANDSHAKE_COORDINATOR_SIZE],
    minimum_weight_magnitude: u8,
    supported_messages: [u8; HANDSHAKE_VARIABLE_MAX_SIZE],
}

impl Handshake {
    // TODO supported_messages as slice ?
    pub fn new(
        port: u16,
        timestamp: u64,
        coordinator: &[u8; HANDSHAKE_COORDINATOR_SIZE],
        minimum_weight_magnitude: u8,
        supported_messages: &[u8; HANDSHAKE_VARIABLE_MAX_SIZE],
    ) -> Self {
        let mut self_coordinator = [0; HANDSHAKE_COORDINATOR_SIZE];
        let mut self_supported_messages = [0; HANDSHAKE_VARIABLE_MAX_SIZE];

        self_coordinator.copy_from_slice(coordinator);
        self_supported_messages.copy_from_slice(supported_messages);

        Self {
            port: port,
            timestamp: timestamp,
            coordinator: self_coordinator,
            minimum_weight_magnitude: minimum_weight_magnitude,
            supported_messages: self_supported_messages,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn coordinator(&self) -> &[u8; HANDSHAKE_COORDINATOR_SIZE] {
        &self.coordinator
    }

    pub fn minimum_weight_magnitude(&self) -> u8 {
        self.minimum_weight_magnitude
    }

    pub fn supported_messages(&self) -> &[u8; HANDSHAKE_VARIABLE_MAX_SIZE] {
        &self.supported_messages
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; HANDSHAKE_COORDINATOR_SIZE],
            minimum_weight_magnitude: 0,
            supported_messages: [0; HANDSHAKE_VARIABLE_MAX_SIZE],
        }
    }
}

impl Message for Handshake {
    fn id() -> u8 {
        HANDSHAKE_ID
    }

    fn size_range() -> Range<usize> {
        (HANDSHAKE_CONSTANT_SIZE + HANDSHAKE_VARIABLE_MIN_SIZE)
            ..(HANDSHAKE_CONSTANT_SIZE + HANDSHAKE_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self {
            port: 0,
            timestamp: 0,
            coordinator: [0; HANDSHAKE_COORDINATOR_SIZE],
            minimum_weight_magnitude: 0,
            supported_messages: [0; HANDSHAKE_VARIABLE_MAX_SIZE],
        };

        let mut offset = 0;

        message.port = u16::from_be_bytes(
            bytes[offset..offset + HANDSHAKE_PORT_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HANDSHAKE_PORT_SIZE;

        message.timestamp = u64::from_be_bytes(
            bytes[offset..offset + HANDSHAKE_TIMESTAMP_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HANDSHAKE_TIMESTAMP_SIZE;

        message
            .coordinator
            .copy_from_slice(&bytes[offset..offset + HANDSHAKE_COORDINATOR_SIZE]);
        offset += HANDSHAKE_COORDINATOR_SIZE;

        message.minimum_weight_magnitude = u8::from_be_bytes(
            bytes[offset..offset + HANDSHAKE_MINIMUM_WEIGHT_MAGNITUDE]
                .try_into()
                .map_err(|_| MessageError::InvalidMessageField)?,
        );
        offset += HANDSHAKE_MINIMUM_WEIGHT_MAGNITUDE;

        message.supported_messages[0..bytes[offset..].len()].copy_from_slice(&bytes[offset..]);

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.port.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.coordinator);
        bytes.extend_from_slice(&self.minimum_weight_magnitude.to_be_bytes());
        bytes.extend_from_slice(&self.supported_messages);

        bytes
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bee_test::slices::slice_eq;

    const PORT: u16 = 0xcd98;
    const TIMESTAMP: u64 = 0xb2a1d7546a470ed8;
    const COORDINATOR: [u8; HANDSHAKE_COORDINATOR_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232,
        31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203, 67, 93,
        74, 238, 57, 39, 51, 169, 193, 124, 254,
    ];
    const MINIMUM_WEIGHT_MAGNITUDE: u8 = 0x6e;
    const SUPPORTED_MESSAGES: [u8; HANDSHAKE_VARIABLE_MAX_SIZE] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    #[test]
    fn id_test() {
        assert_eq!(Handshake::id(), HANDSHAKE_ID);
    }

    #[test]
    fn size_range_test() {
        assert_eq!(Handshake::size_range().contains(&60), false);
        assert_eq!(Handshake::size_range().contains(&61), true);
        assert_eq!(Handshake::size_range().contains(&62), true);

        assert_eq!(Handshake::size_range().contains(&91), true);
        assert_eq!(Handshake::size_range().contains(&92), true);
        assert_eq!(Handshake::size_range().contains(&93), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match Handshake::from_bytes(&[0; 60]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 60),
            _ => unreachable!(),
        }
        match Handshake::from_bytes(&[0; 93]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 93),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: Handshake) {
        assert_eq!(message.port(), PORT);
        assert_eq!(message.timestamp(), TIMESTAMP);
        assert_eq!(slice_eq(message.coordinator(), &COORDINATOR), true);
        assert_eq!(message.minimum_weight_magnitude(), MINIMUM_WEIGHT_MAGNITUDE);
        assert_eq!(
            slice_eq(message.supported_messages(), &SUPPORTED_MESSAGES),
            true
        );
    }

    #[test]
    fn into_from_test() {
        let message_from = Handshake::new(
            PORT,
            TIMESTAMP,
            &COORDINATOR,
            MINIMUM_WEIGHT_MAGNITUDE,
            &SUPPORTED_MESSAGES,
        );

        into_from_eq(Handshake::from_bytes(&message_from.into_bytes()).unwrap());
    }

    #[test]
    fn full_into_from_test() {
        let message_from = Handshake::new(
            PORT,
            TIMESTAMP,
            &COORDINATOR,
            MINIMUM_WEIGHT_MAGNITUDE,
            &SUPPORTED_MESSAGES,
        );

        into_from_eq(Handshake::from_full_bytes(&message_from.into_full_bytes()).unwrap());
    }
}
