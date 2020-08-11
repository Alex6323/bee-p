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

use crate::atomic::Error;

use bee_ternary::{T5B1Buf, TritBuf};

use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WotsAddress(Vec<i8>);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ed25519Address([u8; 32]);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Address {
    Wots(WotsAddress),
    Ed25519(Ed25519Address),
}

impl Address {
    pub fn from_ed25519_bytes(bytes: [u8; 32]) -> Self {
        Address::Ed25519(Ed25519Address(bytes))
    }

    pub fn from_wots_tritbuf(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        let trits = trits.as_i8_slice().to_vec();
        if trits.len() != 49 {
            return Err(Error::HashError);
        }
        Ok(Address::Wots(WotsAddress(trits)))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Address::Wots(WotsAddress(address)) => {
                let mut serializer = serializer.serialize_struct("Address", 3)?;
                serializer.serialize_field("Address Type", &0u8)?;
                serializer.serialize_field("Address", address)?;
                serializer.end()
            }
            Address::Ed25519(Ed25519Address(address)) => {
                let mut serializer = serializer.serialize_struct("Address", 3)?;
                serializer.serialize_field("Address Type", &1u8)?;
                serializer.serialize_field("Address", address)?;
                serializer.end()
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SigLockedSingleDeposit {
    pub address: Address,
    pub amount: u64,
}
