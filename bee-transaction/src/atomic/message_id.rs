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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

pub const MESSAGE_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MessageId([u8; MESSAGE_ID_LENGTH]);

impl From<[u8; MESSAGE_ID_LENGTH]> for MessageId {
    fn from(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl MessageId {
    pub fn new(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        bytes.into()
    }
}

impl core::fmt::Display for MessageId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for MessageId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MessageId({})", self.to_string())
    }
}

impl Packable for MessageId {
    fn len_bytes(&self) -> usize {
        MESSAGE_ID_LENGTH
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        Self::pack_bytes(&self.0, buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        let vec = Self::unpack_bytes(buffer, MESSAGE_ID_LENGTH);
        let bytes = unsafe { *(vec.as_slice() as *const [u8] as *const [u8; MESSAGE_ID_LENGTH]) };
        Self(bytes)
    }
}
