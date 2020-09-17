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

use crate::metadata::SnapshotMetadata;

use bee_crypto::ternary::Hash;

use std::collections::HashMap;

pub struct LocalSnapshotMetadata {
    pub(crate) inner: SnapshotMetadata,
    pub(crate) solid_entry_points: HashMap<Hash, u32>,
    pub(crate) seen_milestones: HashMap<Hash, u32>,
}

impl LocalSnapshotMetadata {
    pub fn hash(&self) -> &Hash {
        &self.inner.hash
    }

    pub fn index(&self) -> u32 {
        self.inner.snapshot_index
    }

    pub fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    pub fn solid_entry_points(&self) -> &HashMap<Hash, u32> {
        &self.solid_entry_points
    }

    pub fn seen_milestones(&self) -> &HashMap<Hash, u32> {
        &self.seen_milestones
    }
}
