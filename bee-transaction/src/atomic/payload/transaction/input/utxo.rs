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
    payload::transaction::{constants::INPUT_OUTPUT_INDEX_RANGE, TransactionId},
    Error,
};

use serde::{Deserialize, Serialize};

use super::WriteBytes;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct UTXOInput {
    id: TransactionId,
    index: u16,
}

// TODO builder ?
impl UTXOInput {
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidIndex);
        }

        Ok(Self { id, index })
    }

    pub fn id(&self) -> &TransactionId {
        &self.id
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl core::fmt::Display for UTXOInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.id.to_string(), hex::encode(self.index.to_le_bytes()))
    }
}

impl WriteBytes for UTXOInput {
    fn len_bytes(&self) -> usize {
        self.id.len_bytes() + self.index.len_bytes()
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.id.write_bytes(buffer);
        self.index.write_bytes(buffer);
    }
}
