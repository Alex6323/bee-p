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

//! MessageRequest packet of the protocol version 2

use crate::packet::Packet;

use std::ops::Range;

const HASH_SIZE: usize = 32;
const CONSTANT_SIZE: usize = HASH_SIZE;

/// A packet to request a message.
pub(crate) struct MessageRequest {
    /// Hash of the requested message.
    pub(crate) hash: [u8; HASH_SIZE],
}

impl MessageRequest {
    pub(crate) fn new(hash: &[u8]) -> Self {
        let mut new = Self::default();

        new.hash.copy_from_slice(hash);

        new
    }
}

impl Default for MessageRequest {
    fn default() -> Self {
        Self { hash: [0; HASH_SIZE] }
    }
}

impl Packet for MessageRequest {
    const ID: u8 = 0x05;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut packet = Self::default();

        packet.hash.copy_from_slice(&bytes[0..HASH_SIZE]);

        packet
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.hash)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const HASH: [u8; HASH_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232, 31, 255, 208, 9, 126,
        21, 82, 57, 180, 237, 182, 101,
    ];

    #[test]
    fn id() {
        assert_eq!(MessageRequest::ID, 5);
    }

    #[test]
    fn size_range() {
        assert_eq!(MessageRequest::size_range().contains(&31), false);
        assert_eq!(MessageRequest::size_range().contains(&32), true);
        assert_eq!(MessageRequest::size_range().contains(&33), false);
    }

    #[test]
    fn size() {
        let packet = MessageRequest::new(&HASH);

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = MessageRequest::new(&HASH);
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = MessageRequest::from_bytes(&bytes);

        assert!(packet_to.hash.eq(&HASH));
    }
}
