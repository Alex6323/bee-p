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

use bech32::{self, ToBase32};

// use blake2::{
//     digest::{Update, VariableOutput},
//     VarBlake2b,
// };
//
// use core::convert::TryInto;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ed25519Address([u8; 32]);

impl From<[u8; 32]> for Ed25519Address {
    fn from(bytes: [u8; 32]) -> Self {
        // let mut hasher = VarBlake2b::new(32).unwrap();
        // hasher.update(bytes);
        // let address: [u8; 32] = hasher
        //     .finalize_boxed()
        //     .as_ref()
        //     .try_into()
        //     .expect("Array must be 32 bytes");
        Self(bytes)
    }
}

impl Ed25519Address {
    pub fn new(address: [u8; 32]) -> Self {
        address.into()
    }

    pub fn len(&self) -> usize {
        32
    }

    pub fn to_bech32(&self) -> String {
        let mut serialized = vec![1u8];
        serialized.extend_from_slice(&self.0);
        bech32::encode("iot", serialized.to_base32()).expect("Valid Ed25519 address required")
    }
}
