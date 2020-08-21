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

const DEFAULT_FILE_PATH: &str = "./snapshots/mainnet/export.bin";

#[derive(Default, Deserialize)]
pub struct LocalSnapshotConfigBuilder {
    file_path: Option<String>,
}

impl LocalSnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_path(mut self, file_path: String) -> Self {
        self.file_path.replace(file_path);
        self
    }

    pub fn finish(self) -> LocalSnapshotConfig {
        LocalSnapshotConfig {
            file_path: self.file_path.unwrap_or_else(|| DEFAULT_FILE_PATH.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct LocalSnapshotConfig {
    file_path: String,
}

impl LocalSnapshotConfig {
    pub fn build() -> LocalSnapshotConfigBuilder {
        LocalSnapshotConfigBuilder::new()
    }

    pub fn file_path(&self) -> &String {
        &self.file_path
    }
}
