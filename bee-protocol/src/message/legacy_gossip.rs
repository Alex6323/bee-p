use crate::message::errors::ProtocolMessageError;
use crate::message::Message;

use std::ops::Range;

const LEGACY_GOSSIP_ID: u8 = 0x02;

const LEGACY_GOSSIP_REQUEST_SIZE: usize = 49;
const LEGACY_GOSSIP_CONSTANT_SIZE: usize = LEGACY_GOSSIP_REQUEST_SIZE;
const LEGACY_GOSSIP_VARIABLE_MIN_SIZE: usize = 292;
const LEGACY_GOSSIP_VARIABLE_MAX_SIZE: usize = 1604;

#[derive(Clone)]
pub struct LegacyGossip {
    transaction: Vec<u8>,
    request: [u8; LEGACY_GOSSIP_REQUEST_SIZE],
}

impl LegacyGossip {
    pub fn new(transaction: &Vec<u8>, request: [u8; LEGACY_GOSSIP_REQUEST_SIZE]) -> Self {
        // TODO clone ?
        Self {
            transaction: transaction.clone(),
            request: request,
        }
    }

    pub fn transaction(&self) -> &Vec<u8> {
        &self.transaction
    }

    pub fn request(&self) -> &[u8; LEGACY_GOSSIP_REQUEST_SIZE] {
        &self.request
    }
}

impl Message for LegacyGossip {
    fn id() -> u8 {
        LEGACY_GOSSIP_ID
    }

    fn size_range() -> Range<usize> {
        (LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MIN_SIZE)
            ..(LEGACY_GOSSIP_CONSTANT_SIZE + LEGACY_GOSSIP_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolMessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(ProtocolMessageError::InvalidMessageLength(bytes.len()))?;
        }

        let mut message = Self {
            transaction: Vec::new(),
            request: [0; LEGACY_GOSSIP_REQUEST_SIZE],
        };

        let mut offset = 0;

        message
            .transaction
            .extend_from_slice(&bytes[offset..offset + bytes.len() - LEGACY_GOSSIP_REQUEST_SIZE]);
        offset += bytes.len() - LEGACY_GOSSIP_REQUEST_SIZE;

        message
            .request
            .copy_from_slice(&bytes[offset..offset + LEGACY_GOSSIP_REQUEST_SIZE]);

        Ok(message)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = self.transaction.clone();

        bytes.extend_from_slice(&self.request);

        bytes
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bee_test::slices::slice_eq;

    const TRANSACTION: [u8; 500] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 0, 1,
        2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
        50, 51, 52, 53, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
        74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,
        97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
        116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133,
        134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
        152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169,
        170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187,
        188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205,
        206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223,
        224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241,
        242, 243, 244,
    ];
    const REQUEST: [u8; LEGACY_GOSSIP_REQUEST_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232,
        31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203, 67, 93,
        74, 238, 57, 39, 51, 169, 193, 124, 254,
    ];

    #[test]
    fn id_test() {
        assert_eq!(LegacyGossip::id(), LEGACY_GOSSIP_ID);
    }

    #[test]
    fn size_range_test() {
        assert_eq!(LegacyGossip::size_range().contains(&340), false);
        assert_eq!(LegacyGossip::size_range().contains(&341), true);
        assert_eq!(LegacyGossip::size_range().contains(&342), true);

        assert_eq!(LegacyGossip::size_range().contains(&1652), true);
        assert_eq!(LegacyGossip::size_range().contains(&1653), true);
        assert_eq!(LegacyGossip::size_range().contains(&1654), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match LegacyGossip::from_bytes(&[0; 340]) {
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 340),
            _ => unreachable!(),
        }
        match LegacyGossip::from_bytes(&[0; 1654]) {
            Err(ProtocolMessageError::InvalidMessageLength(length)) => assert_eq!(length, 1654),
            _ => unreachable!(),
        }
    }

    fn into_from_eq(message: LegacyGossip) {
        assert_eq!(slice_eq(message.transaction(), &TRANSACTION), true);
        assert_eq!(slice_eq(message.request(), &REQUEST), true);
    }

    #[test]
    fn into_from_test() {
        let message_from = LegacyGossip::new(&TRANSACTION.to_vec(), REQUEST);

        into_from_eq(LegacyGossip::from_bytes(&message_from.into_bytes()).unwrap());
    }

    #[test]
    fn full_into_from_test() {
        let message_from = LegacyGossip::new(&TRANSACTION.to_vec(), REQUEST);

        into_from_eq(LegacyGossip::from_bytes(&message_from.into_bytes()).unwrap());
    }
}
