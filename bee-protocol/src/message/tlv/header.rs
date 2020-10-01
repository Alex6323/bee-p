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

//! Header of the type-length-value encoding.

use std::convert::TryInto;

const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
pub(crate) const HEADER_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

/// A header for the type-length-value encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Header {
    /// Type of the message.
    pub(crate) message_type: u8,
    /// Length of the message.
    pub(crate) message_length: u16,
}

impl Header {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            message_type: bytes[0],
            // TODO propagate error
            message_length: u16::from_le_bytes(bytes[HEADER_TYPE_SIZE..HEADER_SIZE].try_into().unwrap()),
        }
    }

    pub(crate) fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.message_type;
        bytes[1..].copy_from_slice(&self.message_length.to_le_bytes());
    }
}
