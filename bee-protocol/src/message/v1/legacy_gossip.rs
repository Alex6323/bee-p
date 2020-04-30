//! LegacyGossip message of the protocol version 1

use crate::message::Message;

use std::ops::Range;

const HASH_SIZE: usize = 49;
const CONSTANT_SIZE: usize = HASH_SIZE;
const VARIABLE_MIN_SIZE: usize = 292;
const VARIABLE_MAX_SIZE: usize = 1604;

#[derive(Clone)]
pub(crate) struct LegacyGossip {
    pub(crate) transaction: Vec<u8>,
    pub(crate) hash: [u8; HASH_SIZE],
}

impl LegacyGossip {
    pub(crate) fn new(transaction: &[u8], hash: [u8; HASH_SIZE]) -> Self {
        Self {
            transaction: transaction.to_vec(),
            hash: hash,
        }
    }
}

impl Default for LegacyGossip {
    fn default() -> Self {
        Self {
            transaction: Default::default(),
            hash: [0; HASH_SIZE],
        }
    }
}

impl Message for LegacyGossip {
    const ID: u8 = 0x02;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE + VARIABLE_MIN_SIZE)..(CONSTANT_SIZE + VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message = Self::default();

        let (bytes, next) = bytes.split_at(bytes.len() - HASH_SIZE);
        message.transaction.extend_from_slice(bytes);

        message.hash.copy_from_slice(next);

        message
    }

    fn size(&self) -> usize {
        self.transaction.len() + CONSTANT_SIZE
    }

    fn to_bytes(self, bytes: &mut [u8]) {
        bytes[0..self.transaction.len()].copy_from_slice(&self.transaction);
        bytes[self.transaction.len()..].copy_from_slice(&self.hash);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::message::{
        Header,
        MessageError,
        Tlv,
        HEADER_SIZE,
    };

    use bee_test::slices::slice_eq;

    const TRANSACTION: [u8; 500] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
        58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85,
        86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110,
        111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132,
        133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
        155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176,
        177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198,
        199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242,
        243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
        14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
        42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
        99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120,
        121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142,
        143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164,
        165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186,
        187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208,
        209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
        231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244,
    ];
    const REQUEST: [u8; HASH_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232, 31, 255, 208, 9, 126,
        21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203, 67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
    ];

    #[test]
    fn id() {
        assert_eq!(LegacyGossip::ID, 2);
    }

    #[test]
    fn size_range() {
        assert_eq!(LegacyGossip::size_range().contains(&340), false);
        assert_eq!(LegacyGossip::size_range().contains(&341), true);
        assert_eq!(LegacyGossip::size_range().contains(&342), true);

        assert_eq!(LegacyGossip::size_range().contains(&1652), true);
        assert_eq!(LegacyGossip::size_range().contains(&1653), true);
        assert_eq!(LegacyGossip::size_range().contains(&1654), false);
    }

    #[test]
    fn size() {
        let message = LegacyGossip::new(&TRANSACTION, REQUEST);

        assert_eq!(message.size(), CONSTANT_SIZE + 500);
    }

    fn to_from_eq(message: LegacyGossip) {
        assert_eq!(slice_eq(&message.transaction, &TRANSACTION), true);
        assert_eq!(slice_eq(&message.hash, &REQUEST), true);
    }

    #[test]
    fn to_from() {
        let message_from = LegacyGossip::new(&TRANSACTION, REQUEST);
        let mut bytes = vec![0u8; message_from.size()];

        message_from.to_bytes(&mut bytes);
        to_from_eq(LegacyGossip::from_bytes(&bytes));
    }

    #[test]
    fn tlv_invalid_length() {
        match Tlv::from_bytes::<LegacyGossip>(
            &Header {
                message_type: LegacyGossip::ID,
                message_length: 340,
            },
            &[0; 340],
        ) {
            Err(MessageError::InvalidPayloadLength(length)) => assert_eq!(length, 340),
            _ => unreachable!(),
        }
        match Tlv::from_bytes::<LegacyGossip>(
            &Header {
                message_type: LegacyGossip::ID,
                message_length: 1654,
            },
            &[0; 1654],
        ) {
            Err(MessageError::InvalidPayloadLength(length)) => assert_eq!(length, 1654),
            _ => unreachable!(),
        }
    }

    #[test]
    fn tlv() {
        let message_from = LegacyGossip::new(&TRANSACTION, REQUEST);
        let bytes = Tlv::into_bytes(message_from);

        to_from_eq(
            Tlv::from_bytes::<LegacyGossip>(&Header::from_bytes(&bytes[0..HEADER_SIZE]), &bytes[HEADER_SIZE..])
                .unwrap(),
        );
    }
}
