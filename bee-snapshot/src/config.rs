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

use crate::{
    local::{LocalSnapshotConfig, LocalSnapshotConfigBuilder},
    pruning::{PruningConfig, PruningConfigBuilder},
};

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    local: LocalSnapshotConfigBuilder,
    pruning: PruningConfigBuilder,
}

impl SnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn local_file_path(mut self, file_path: String) -> Self {
        self.local = self.local.file_path(file_path);
        self
    }

    pub fn finish(self) -> SnapshotConfig {
        SnapshotConfig {
            local: self.local.finish(),
            pruning: self.pruning.finish(),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConfig {
    local: LocalSnapshotConfig,
    pruning: PruningConfig,
}

impl SnapshotConfig {
    pub fn build() -> SnapshotConfigBuilder {
        SnapshotConfigBuilder::new()
    }

    pub fn local(&self) -> &LocalSnapshotConfig {
        &self.local
    }

    pub fn pruning(&self) -> &PruningConfig {
        &self.pruning
    }
}
