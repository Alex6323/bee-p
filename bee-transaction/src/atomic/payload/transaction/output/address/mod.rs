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

mod ed25519;
mod wots;

pub use ed25519::Ed25519Address;
pub use wots::WotsAddress;

use serde::{Deserialize, Serialize};

use alloc::string::String;

use super::super::super::WriteBytes;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Address {
    Wots(WotsAddress),
    Ed25519(Ed25519Address),
}

impl From<WotsAddress> for Address {
    fn from(address: WotsAddress) -> Self {
        Self::Wots(address)
    }
}

impl From<Ed25519Address> for Address {
    fn from(address: Ed25519Address) -> Self {
        Self::Ed25519(address)
    }
}

impl Address {
    pub fn to_bech32(&self) -> String {
        match self {
            Address::Wots(address) => address.to_bech32(),
            Address::Ed25519(address) => address.to_bech32(),
        }
    }
}

impl WriteBytes for Address {
    fn len_bytes(&self) -> usize {
        match self {
            Self::Wots(address) => 0u8.len_bytes() + address.len_bytes(),
            Self::Ed25519(address) => 1u8.len_bytes() + address.len_bytes(),
        }
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        match self {
            Self::Wots(address) => {
                0u8.write_bytes(buffer);
                address.write_bytes(buffer);
            }
            Self::Ed25519(address) => {
                1u8.write_bytes(buffer);
                address.write_bytes(buffer);
            }
        }
    }
}
