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

use bech32::{self, ToBase32};
use serde::{Deserialize, Serialize};

use alloc::{string::String, vec};

const ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Ed25519Address([u8; ADDRESS_LENGTH]);

impl From<[u8; ADDRESS_LENGTH]> for Ed25519Address {
    fn from(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl Ed25519Address {
    pub fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    pub fn len(&self) -> usize {
        ADDRESS_LENGTH
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn to_bech32(&self) -> String {
        let mut serialized = vec![1u8];
        serialized.extend_from_slice(&self.0);
        bech32::encode("iot", serialized.to_base32()).expect("Valid Ed25519 address required")
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.to_bech32())
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self.to_string())
    }
}

impl Packable for Ed25519Address {
    fn packed_len(&self) -> usize {
        ADDRESS_LENGTH
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        buf.write_all(self.0.as_ref())?;

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        buf.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
