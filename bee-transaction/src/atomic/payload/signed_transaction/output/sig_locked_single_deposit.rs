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

use bech32::{self, ToBase32};
use blake2::{VarBlake2b, digest::{Update, VariableOutput}};
use serde::{Deserialize, Serialize};

use std::convert::TryInto;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Address {
    Wots(Vec<i8>),
    Ed25519([u8; 32]),
}

impl Address {
    pub fn from_ed25519_bytes(bytes: &[u8; 32]) -> Self {
        let mut hasher = VarBlake2b::new(32).unwrap();
        hasher.update(bytes);
        let address: [u8; 32] = hasher.finalize_boxed().as_ref().try_into().expect("Array must be 32 bytes");
        Address::Ed25519(address)
    }

    pub fn from_wots_tritbuf(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        let trits = trits.as_i8_slice().to_vec();
        if trits.len() != 49 {
            return Err(Error::HashError);
        }
        Ok(Address::Wots(trits))
    }

    pub fn to_bech32_string(&self) -> String { 
        match self {
            Address::Ed25519(a) => {
                let mut serialized = vec![1u8];
                a.iter().for_each(|b| serialized.push(*b));
                bech32::encode("iot", serialized.to_base32()).expect("Valid Ed25519 address required")
            }
            _ => todo!()
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SigLockedSingleDeposit {
    pub address: Address,
    pub amount: u64,
}
