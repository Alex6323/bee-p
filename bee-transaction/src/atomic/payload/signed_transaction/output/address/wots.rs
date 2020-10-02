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

use bee_ternary::{T5B1Buf, TritBuf};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WotsAddress(Vec<i8>);

impl From<&TritBuf<T5B1Buf>> for WotsAddress {
    fn from(trits: &TritBuf<T5B1Buf>) -> Self {
        let trits = trits.as_i8_slice().to_vec();
        // TODO TRyInto
        // if trits.len() != 49 {
        //     return Err(Error::HashError);
        // }
        // Ok(Address::Wots(trits))
        Self(trits)
    }
}

impl WotsAddress {
    pub fn new(trits: &TritBuf<T5B1Buf>) -> Self {
        trits.into()
    }
}
