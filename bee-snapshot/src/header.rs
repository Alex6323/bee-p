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

use bee_crypto::ternary::Hash;

pub struct SnapshotHeader {
    pub(crate) coordinator: Hash,
    pub(crate) hash: Hash,
    pub(crate) snapshot_index: u32,
    pub(crate) entry_point_index: u32,
    pub(crate) pruning_index: u32,
    pub(crate) timestamp: u64,
}

impl SnapshotHeader {
    pub fn coordinator(&self) -> &Hash {
        &self.coordinator
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn snapshot_index(&self) -> u32 {
        self.snapshot_index
    }

    pub fn entry_point_index(&self) -> u32 {
        self.entry_point_index
    }

    pub fn pruning_index(&self) -> u32 {
        self.pruning_index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
