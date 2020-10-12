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
    payload::transaction::constants::INPUT_OUTPUT_INDEX_RANGE,
    Error,
};

use serde::{Deserialize, Serialize};

use core::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ReferenceUnlock(u16);

impl TryFrom<u16> for ReferenceUnlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Self::Error::InvalidIndex);
        }

        Ok(Self(index))
    }
}

impl ReferenceUnlock {
    pub fn new(index: u16) -> Result<Self, Error> {
        index.try_into()
    }

    pub fn index(&self) -> u16 {
        self.0
    }
}

impl Packable for ReferenceUnlock {
    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        self.0.pack(buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        let index = u16::unpack(buffer);
        Self(index)
    }
}
