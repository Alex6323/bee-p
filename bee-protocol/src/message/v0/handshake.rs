use crate::message::{
    Message,
    MessageError,
};

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

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            return Err(MessageError::InvalidPayloadLength(bytes.len()));
        }

        let mut message = Self::default();
        let mut offset = 0;

        message.port = u16::from_be_bytes(
            bytes[offset..offset + PORT_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidPayloadField)?,
        );
        offset += PORT_SIZE;

        message.timestamp = u64::from_be_bytes(
            bytes[offset..offset + TIMESTAMP_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidPayloadField)?,
        );
        offset += TIMESTAMP_SIZE;

        message
            .coordinator
            .copy_from_slice(&bytes[offset..offset + COORDINATOR_SIZE]);
        offset += COORDINATOR_SIZE;

        message.minimum_weight_magnitude = u8::from_be_bytes(
            bytes[offset..offset + MINIMUM_WEIGHT_MAGNITUDE_SIZE]
                .try_into()
                .map_err(|_| MessageError::InvalidPayloadField)?,
        );
        offset += MINIMUM_WEIGHT_MAGNITUDE_SIZE;

        message.supported_versions = bytes[offset..].to_vec();

        Ok(message)
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE + self.supported_versions.len()
    }

    fn to_bytes(self, bytes: &mut [u8]) {
        let mut offset = 0;

        bytes[offset..offset + PORT_SIZE].copy_from_slice(&self.port.to_be_bytes());
        offset += PORT_SIZE;
        bytes[offset..offset + TIMESTAMP_SIZE].copy_from_slice(&self.timestamp.to_be_bytes());
        offset += TIMESTAMP_SIZE;
        bytes[offset..offset + COORDINATOR_SIZE].copy_from_slice(&self.coordinator);
        offset += COORDINATOR_SIZE;
        bytes[offset..offset + MINIMUM_WEIGHT_MAGNITUDE_SIZE]
            .copy_from_slice(&self.minimum_weight_magnitude.to_be_bytes());
        offset += MINIMUM_WEIGHT_MAGNITUDE_SIZE;
        bytes[offset..].copy_from_slice(&self.supported_versions);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::message::{
        Header,
        Tlv,
        HEADER_SIZE,
    };

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
    fn from_bytes_invalid_length() {
        match Handshake::from_bytes(&[0; 60]) {
            Err(MessageError::InvalidPayloadLength(length)) => assert_eq!(length, 60),
            _ => unreachable!(),
        }
        match Handshake::from_bytes(&[0; 93]) {
            Err(MessageError::InvalidPayloadLength(length)) => assert_eq!(length, 93),
            _ => unreachable!(),
        }
    }

    #[test]
    fn size() {
        let message = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);

        assert_eq!(message.size(), CONSTANT_SIZE + 10);
    }

    fn to_from_eq(message: Handshake) {
        assert_eq!(message.port, PORT);
        assert_eq!(slice_eq(&message.coordinator, &COORDINATOR), true);
        assert_eq!(message.minimum_weight_magnitude, MINIMUM_WEIGHT_MAGNITUDE);
        assert_eq!(slice_eq(&message.supported_versions, &SUPPORTED_VERSIONS), true);
    }

    #[test]
    fn to_from() {
        let message_from = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);
        let mut bytes = vec![0u8; message_from.size()];

        message_from.to_bytes(&mut bytes);
        to_from_eq(Handshake::from_bytes(&bytes).unwrap());
    }

    #[test]
    fn tlv() {
        let message_from = Handshake::new(PORT, &COORDINATOR, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);
        let bytes = Tlv::into_bytes(message_from);

        to_from_eq(
            Tlv::from_bytes::<Handshake>(&Header::from_bytes(&bytes[0..HEADER_SIZE]), &bytes[HEADER_SIZE..]).unwrap(),
        );
    }
}
