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

use serde::Deserialize;

const DEFAULT_PATH: &str = "./snapshots/mainnet/snapshot.txt";
const DEFAULT_INDEX: u32 = 1050000;

#[derive(Default, Deserialize)]
pub struct GlobalSnapshotConfigBuilder {
    path: Option<String>,
    index: Option<u32>,
}

impl GlobalSnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: String) -> Self {
        self.path.replace(path);
        self
    }

    pub fn index(mut self, index: u32) -> Self {
        self.index.replace(index);
        self
    }

    pub fn finish(self) -> GlobalSnapshotConfig {
        GlobalSnapshotConfig {
            path: self.path.unwrap_or_else(|| DEFAULT_PATH.to_string()),
            index: self.index.unwrap_or(DEFAULT_INDEX),
        }
    }
}

#[derive(Clone)]
pub struct GlobalSnapshotConfig {
    path: String,
    index: u32,
}

impl GlobalSnapshotConfig {
    pub fn build() -> GlobalSnapshotConfigBuilder {
        GlobalSnapshotConfigBuilder::new()
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn index(&self) -> &u32 {
        &self.index
    }
}
