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

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::string::ToString;

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
    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        buf.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; MESSAGE_ID_LENGTH];
        buf.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
