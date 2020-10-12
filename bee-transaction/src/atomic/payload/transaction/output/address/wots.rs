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

use crate::atomic::{
    packable::{Buf, BufMut, Packable},
    Error,
};

use bee_ternary::{T5B1Buf, TritBuf};

use bytemuck::cast_slice;

use serde::{Deserialize, Serialize};

use alloc::string::String;
use core::convert::{TryFrom, TryInto};

// TODO length is 243, change to array when std::array::LengthAtMost32 disappears.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct WotsAddress(Box<[u8]>);

impl TryFrom<&TritBuf<T5B1Buf>> for WotsAddress {
    type Error = Error;

    fn try_from(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        // TODO const
        if trits.len() != 243 {
            return Err(Error::InvalidAddress);
        }

        Ok(Self(cast_slice(trits.as_i8_slice()).to_vec().into_boxed_slice()))
    }
}

// TODO builder ?
impl WotsAddress {
    pub fn new(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        trits.try_into()
    }

    pub fn to_bech32(&self) -> String {
        // TODO
        String::from("")
    }
}

impl core::fmt::Display for WotsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.to_bech32())
    }
}

impl core::fmt::Debug for WotsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "WotsAddress({})", self.to_string())
    }
}

impl Packable for WotsAddress {
    fn packed_len(&self) -> usize {
        243
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        Self::pack_bytes(self.0.as_ref(), buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        let bytes = Self::unpack_bytes(buffer, 243).into_boxed_slice();
        Self(bytes)
    }
}
