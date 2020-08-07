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

use bee_ternary::{TritBuf, T5B1Buf};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WotsAddress(Vec<i8>);

impl WotsAddress {
    pub fn from_tritbuf(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        let trits = trits.as_i8_slice().to_vec();
        if trits.len() != 49 {
            return Err(Error::HashError)
        }
        Ok(WotsAddress(trits))
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ed25519Address([u8; 32]);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Address {
    Wots(WotsAddress),
    Ed25519(Ed25519Address),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SigLockedSingleDeposit {
    pub address: Address,
    pub amount: u64,
}
