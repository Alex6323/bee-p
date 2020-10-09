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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

use alloc::string::String;

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

impl Packable for Address {
    fn len_bytes(&self) -> usize {
        match self {
            Self::Wots(address) => 0u8.len_bytes() + address.len_bytes(),
            Self::Ed25519(address) => 1u8.len_bytes() + address.len_bytes(),
        }
    }

    fn pack_bytes<B: BufMut>(&self, buffer: &mut B) {
        match self {
            Self::Wots(address) => {
                0u8.pack_bytes(buffer);
                address.pack_bytes(buffer);
            }
            Self::Ed25519(address) => {
                1u8.pack_bytes(buffer);
                address.pack_bytes(buffer);
            }
        }
    }

    fn unpack_bytes<B: Buf>(buffer: &mut B) -> Self {
        match u8::unpack_bytes(buffer) {
            0 => Self::Wots(WotsAddress::unpack_bytes(buffer)),
            1 => Self::Ed25519(Ed25519Address::unpack_bytes(buffer)),
            _ => unreachable!(),
        }
    }
}
