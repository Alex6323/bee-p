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

use crate::atomic::{payload::transaction::constants::INPUT_OUTPUT_INDEX_RANGE, Error, Hash};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct UTXOInput {
    id: Hash,
    index: u8,
}

// TODO builder ?
impl UTXOInput {
    pub fn new(id: Hash, index: u8) -> Result<Self, Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidIndex);
        }

        Ok(Self { id, index })
    }

    pub fn id(&self) -> &Hash {
        &self.id
    }

    pub fn index(&self) -> u8 {
        self.index
    }
}
